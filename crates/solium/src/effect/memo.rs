#![cfg_attr(not(test), expect(dead_code, reason = "Task 44 runs the memos"))]

//! The per-monitor memo (Ruling F30): one result per monitor, cut and
//! prefix, shared by every slot whose chain's memoised prefix is the same,
//! re-run only when the xray under it was redrawn or its key moved.
//! `tests::two_slots_with_one_prefix_share_one_memo`,
//! `tests::a_memo_runs_once_per_xray_generation`.

use std::collections::HashMap;

use smithay::backend::renderer::gles::GlesTexture;
use smithay::utils::{Physical, Rectangle};

use super::run::Held;
use super::tree::Tree;
use super::xray::Cut;

/// Which memo: a monitor, the cut of xray it reads, and the chained content
/// key of the prefix steps up to the one it holds (`upto`, the prefix's
/// `stage::Plan::keys()[export.step]`), so a prefix result two chains both
/// read is one memo, and another monitor's or cut's is another.
/// `tests::two_slots_with_one_prefix_share_one_memo`,
/// `tests::a_memo_is_one_per_monitor_and_cut`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MemoKey {
    pub(crate) monitor: String,
    pub(crate) cut: Cut,
    pub(crate) upto: u64,
}

/// One memo's targets and its result, the xray generation it was made
/// from, its own generation (moved on each run), whether a run made it,
/// the last pass a slot wanted it, the plan its run is handed (the prefix
/// up to the step it holds), and the bytes it is counted at against
/// `effects.memo.budget_mib`. `tests::a_memos_generation_moves_on_each_run`,
/// `tests::a_memo_keeps_the_plan_it_was_first_wanted_with`,
/// `tests::a_memo_whose_run_made_nothing_runs_again`,
/// `tests::a_held_memo_is_counted_at_its_latest_size`.
#[derive(Debug, Default)]
pub(crate) struct Memo {
    pub(crate) held: Held,
    pub(crate) generation: u64,
    pub(crate) xray: Option<u64>,
    pub(crate) made: bool,
    pub(crate) seen: u64,
    pub(crate) plan: Option<solium_effects::stage::Plan>,
    pub(crate) bytes: u64,
}

/// Every memo, by key, and whether a memo over the budget was logged.
/// Outside the pool's free-list budget: a memo holds its targets while it
/// is wanted, and [`Memos::sweep`] gives them back.
/// `tests::a_memo_no_slot_wanted_this_pass_is_dropped`.
#[derive(Debug, Default)]
pub(crate) struct Memos {
    by_key: HashMap<MemoKey, Memo>,
    refused_logged: bool,
}

impl Memos {
    /// A slot reads this memo this pass.
    /// `tests::two_slots_with_one_prefix_share_one_memo`.
    pub(crate) fn want(&mut self, key: &MemoKey, pass: u64) {
        self.by_key.entry(key.clone()).or_default().seen = pass;
    }

    /// Held, and never run, run with nothing made, or run on another xray
    /// generation; a memo not held is never run.
    /// `tests::a_memo_runs_once_per_xray_generation`,
    /// `tests::a_memo_whose_run_made_nothing_runs_again`.
    pub(crate) fn needs_run(&self, key: &MemoKey, xray_generation: u64) -> bool {
        self.by_key
            .get(key)
            .is_some_and(|memo| !memo.made || memo.xray != Some(xray_generation))
    }

    /// A memo, to run into its `held` or to clear `made` after a run that
    /// made nothing. `tests::a_memo_whose_run_made_nothing_runs_again`.
    pub(crate) fn get_mut(&mut self, key: &MemoKey) -> Option<&mut Memo> {
        self.by_key.get_mut(key)
    }

    /// The plan a wanted memo runs, kept from the first slot that wanted
    /// it: the key is that plan's content, so every later one is the same.
    /// `tests::a_memo_keeps_the_plan_it_was_first_wanted_with`.
    pub(crate) fn set_plan(
        &mut self,
        key: &MemoKey,
        plan: impl FnOnce() -> solium_effects::stage::Plan,
    ) {
        if let Some(memo) = self.by_key.get_mut(key)
            && memo.plan.is_none()
        {
            memo.plan = Some(plan());
        }
    }

    /// `tests::a_memo_keeps_the_plan_it_was_first_wanted_with`.
    pub(crate) fn plan(&self, key: &MemoKey) -> Option<&solium_effects::stage::Plan> {
        self.by_key.get(key)?.plan.as_ref()
    }

    /// The memo ran over this xray generation: its own generation moves.
    /// `tests::a_memos_generation_moves_on_each_run`.
    pub(crate) fn ran(&mut self, key: &MemoKey, xray_generation: u64) {
        if let Some(memo) = self.by_key.get_mut(key) {
            memo.xray = Some(xray_generation);
            memo.made = true;
            memo.generation = memo.generation.wrapping_add(1);
        }
    }

    /// `tests::a_memos_generation_moves_on_each_run`.
    pub(crate) fn generation(&self, key: &MemoKey) -> Option<u64> {
        self.by_key.get(key).map(|memo| memo.generation)
    }

    /// The memo's result, while it stands.
    #[expect(
        dead_code,
        reason = "Task 44 reads a memo's result, which only a run on the GPU makes"
    )]
    pub(crate) fn output(&self, key: &MemoKey) -> Option<&GlesTexture> {
        self.by_key.get(key)?.held.output()
    }

    /// Give back every memo no slot wanted for `keep_passes` passes
    /// (`effects.memo.keep_passes`; 0 gives back what this pass did not
    /// want). `tests::a_memo_no_slot_wanted_this_pass_is_dropped`,
    /// `tests::a_memo_is_kept_for_keep_passes`.
    pub(crate) fn sweep(&mut self, pass: u64, keep_passes: u64, pool: &mut crate::pool::Pool) {
        self.by_key.retain(|_, memo| {
            let keep = memo.seen.saturating_add(keep_passes) >= pass;
            if !keep {
                memo.held.release(pool);
            }
            keep
        });
    }

    /// `tests::two_slots_with_one_prefix_share_one_memo`.
    pub(crate) fn len(&self) -> usize {
        self.by_key.len()
    }

    /// Whether the memo `key` may be held: it already is, or the memos held
    /// and its `bytes` fit `budget_mib` (`effects.memo.budget_mib`, 0: no
    /// cap). An admitted memo is counted at the `bytes` it was last admitted
    /// at, so a held memo whose xray grew is counted at its new size. Over
    /// the budget nothing is held and the caller runs the whole plan per
    /// part (Task 44); the first refusal is logged.
    /// `tests::a_new_memo_over_the_budget_is_not_admitted`,
    /// `tests::a_held_memo_is_counted_at_its_latest_size`.
    pub(crate) fn admit(&mut self, key: &MemoKey, bytes: u64, budget_mib: u64) -> bool {
        if let Some(memo) = self.by_key.get_mut(key) {
            memo.bytes = bytes;
            return true;
        }
        let fits = budget_mib == 0
            || self.held_bytes().saturating_add(bytes) <= budget_mib.saturating_mul(1024 * 1024);
        if fits {
            self.by_key.insert(
                key.clone(),
                Memo {
                    bytes,
                    ..Memo::default()
                },
            );
        } else if !self.refused_logged {
            self.refused_logged = true;
            tracing::warn!(
                budget_mib,
                held = self.held_bytes(),
                "effects.memo.budget_mib reached: a new memo is not held, its parts run their whole chain"
            );
        }
        fits
    }

    /// The bytes every held memo is counted at.
    /// `tests::a_new_memo_over_the_budget_is_not_admitted`.
    pub(crate) fn held_bytes(&self) -> u64 {
        self.by_key
            .values()
            .fold(0, |held, memo| held.saturating_add(memo.bytes))
    }
}

/// `effects.memo` (Ruling F30; amended 2026-10-08, configurable): how far a
/// memo's prefix runs, how many passes an unwanted memo is kept, and the
/// most the held memos may take, in MiB (0: no cap). The default is
/// `Split::Resize`, 0 and 0. `tests::effects_memo_is_read_and_checked`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MemoConfig {
    pub(crate) split: solium_effects::stage::Split,
    pub(crate) keep_passes: u64,
    pub(crate) budget_mib: u64,
}

/// The keys `effects.memo` takes.
const KEYS: [&str; 3] = ["split", "keep_passes", "budget_mib"];

/// `effects.memo`, read and checked through `Tree`'s own shape: a table of
/// named keys, each refused by name outside its words or bounds, an unknown
/// key named with the nearest known one (as Ruling 28's are).
/// `tests::effects_memo_is_read_and_checked`,
/// `tests::effects_memo_refuses_what_is_not_its_shape`.
pub(crate) fn parse(tree: &Tree) -> Result<MemoConfig, String> {
    use solium_effects::stage::Split;
    let Tree::Table { list, fields } = tree else {
        return Err(
            "`effects.memo` is a table: { split = \"resize\", keep_passes = 0, budget_mib = 0 }"
                .to_owned(),
        );
    };
    if !list.is_empty() {
        return Err("`effects.memo` takes named keys: split, keep_passes or budget_mib".to_owned());
    }
    let mut config = MemoConfig::default();
    for (key, value) in fields {
        match key.as_str() {
            "split" => {
                config.split = match value {
                    Tree::Text(word) if word == "resize" => Split::Resize,
                    Tree::Text(word) if word == "longest" => Split::Longest,
                    Tree::Text(word) if word == "off" => Split::Off,
                    _ => return Err("`effects.memo.split` is resize, longest or off".to_owned()),
                };
            }
            "keep_passes" => config.keep_passes = whole("keep_passes", value, 3600)?,
            "budget_mib" => config.budget_mib = whole("budget_mib", value, 4096)?,
            other => {
                let meant = solium_effects::spec::nearest(other, &KEYS)
                    .map(|meant| format!("; did you mean `{meant}`?"))
                    .unwrap_or_default();
                return Err(format!(
                    "`{other}` is not a memo key: split, keep_passes or budget_mib{meant}"
                ));
            }
        }
    }
    Ok(config)
}

/// `effects.memo.<name>`, a whole number from 0 to `max`: an integer, or a
/// float with no fraction, as Lua holds `2^6`.
/// `tests::effects_memo_refuses_what_is_not_its_shape`.
fn whole(name: &str, value: &Tree, max: u32) -> Result<u64, String> {
    let number = match value {
        Tree::Int(int) => *int,
        #[expect(
            clippy::cast_possible_truncation,
            reason = "finite, whole and far inside an i64, so the cast is exact"
        )]
        Tree::Number(float) if float.fract() == 0.0 && float.abs() < 1e15 => *float as i64,
        _ => {
            return Err(format!(
                "`effects.memo.{name}` is a whole number, 0 to {max}"
            ));
        }
    };
    u64::try_from(number)
        .ok()
        .filter(|read| *read <= u64::from(max))
        .ok_or_else(|| format!("`effects.memo.{name}` is 0 to {max}, not {number}"))
}

/// What the memo of `plan` holds at the xray's `size`: each step's target
/// at its size, rounded up as the run sizes it, 4 bytes a pixel and 8 for
/// `rgba16f`. An upper bound of the run's ping-pong targets, which is what
/// a budget should count.
/// `tests::a_memos_bytes_are_each_steps_target_at_its_size`.
pub(crate) fn plan_bytes(plan: &solium_effects::stage::Plan, size: (u32, u32)) -> u64 {
    use solium_effects::stage::Format;
    let (sizes, _) = plan.sizes(size);
    plan.steps
        .iter()
        .zip(sizes)
        .map(|(step, (w, h))| {
            let pixel = match step.format {
                Format::Rgba8 => 4,
                Format::Rgba16f => 8,
            };
            u64::from(w) * u64::from(h) * pixel
        })
        .fold(0, u64::saturating_add)
}

/// What a T2 suffix's result was made from: the generations of the memos
/// it reads (hashed; 0 when it reads none), the generation of the xray cut
/// its place reads, the part's padded box in the monitor's pixels, its
/// params, its shape, and its self capture's commit when it reads one. A
/// struct with named fields, so FX6's `animate` adds a field (Ruling F52).
/// `xray` is its own field because every T2 run binds the cut as
/// `backdrop`, and a run that reads no memo (`memo = false`,
/// `effects.memo.split = "off"`, a plan with no resizing step, a place over
/// `effects.memo.budget_mib`, `glass-rect` with `soften_passes = 0`) would
/// otherwise keep a stale backdrop after the wallpaper redraws.
/// `tests::a_t2_key_moves_with_each_of_its_fields`. No `Hash`: nothing
/// hashes a `T2Key` (`needs_run` compares it), and smithay 0.7.0's
/// `Rectangle` implements no `Hash`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct T2Key {
    pub(crate) memo: u64,
    pub(crate) xray: u64,
    pub(crate) padded: Rectangle<i32, Physical>,
    pub(crate) params: u64,
    pub(crate) shape: u64,
    pub(crate) own: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::{MemoKey, Memos, T2Key};
    use crate::effect::xray::Cut;

    fn key(upto: u64) -> MemoKey {
        MemoKey {
            monitor: "DP-1".to_owned(),
            cut: Cut::Background,
            upto,
        }
    }

    /// A plan of `frags` in a chain, built as `stage::tests::plan` builds
    /// one: each step reads the one before it (the first reads `backdrop`),
    /// a `down*` frag at scale 0.5, an `up*` frag sized like the input of the
    /// down step it undoes, any other at 1, each keyed by its frag's hash.
    fn chain(frags: &[&str]) -> solium_effects::stage::Plan {
        use solium_effects::glsl::{Host, Signature, content_hash};
        use solium_effects::stage::{Feed, Format, Plan, Reads, Size, Step};
        let backdrop = || Feed::Input("backdrop".to_owned());
        let mut downs: Vec<Feed> = Vec::new();
        let steps = frags
            .iter()
            .enumerate()
            .map(|(index, frag)| {
                let first = index.checked_sub(1).map_or_else(backdrop, Feed::Step);
                let size = if frag.starts_with("down") {
                    downs.push(first.clone());
                    Size::Scaled {
                        of: first.clone(),
                        scale: 0.5,
                    }
                } else if frag.starts_with("up") {
                    Size::Like(downs.pop().unwrap_or_else(backdrop))
                } else {
                    Size::Scaled {
                        of: first.clone(),
                        scale: 1.0,
                    }
                };
                Step {
                    effect: "test".to_owned(),
                    frag: (*frag).to_owned(),
                    signature: Signature {
                        host: Host::Pass,
                        params: Vec::new(),
                        uses: Vec::new(),
                        known: Vec::new(),
                    },
                    size,
                    format: Format::Rgba8,
                    first,
                    uses: Vec::new(),
                    uniforms: Vec::new(),
                    key: content_hash(&[frag.as_bytes()]),
                    per_part: false,
                }
            })
            .collect();
        Plan {
            steps,
            states: Vec::new(),
            reads: Reads {
                backdrop: true,
                ..Reads::default()
            },
            first_input: "backdrop".to_owned(),
        }
    }

    /// A one-step plan of `frag`, built as `stage::tests::plan` builds one.
    pub(super) fn one_step(frag: &str) -> solium_effects::stage::Plan {
        chain(&[frag])
    }

    /// **A memo runs once per xray generation**: wanted and never run, it
    /// runs; run on generation 3, it waits for 4.
    #[test]
    fn a_memo_runs_once_per_xray_generation() {
        let mut memos = Memos::default();
        memos.want(&key(7), 1);
        assert!(memos.needs_run(&key(7), 3));
        memos.ran(&key(7), 3);
        assert!(!memos.needs_run(&key(7), 3));
        assert!(memos.needs_run(&key(7), 4));
    }

    /// **A memo whose run made nothing runs again**: Task 44 clears `made`
    /// when a prefix run comes back `Pending` or `Failed`, and `needs_run`
    /// reads the flag, so a memo with no result runs again on the same
    /// xray generation.
    #[test]
    fn a_memo_whose_run_made_nothing_runs_again() {
        let mut memos = Memos::default();
        memos.want(&key(7), 1);
        memos.ran(&key(7), 3);
        memos.get_mut(&key(7)).expect("wanted").made = false;
        assert!(memos.needs_run(&key(7), 3));
    }

    /// **A memo wanted again keeps its result**: the next pass's want marks
    /// it seen and leaves its run, generation and plan as they were, so it
    /// is not run again over the same xray.
    #[test]
    fn a_memo_wanted_again_keeps_its_result() {
        let mut memos = Memos::default();
        memos.want(&key(7), 1);
        memos.set_plan(&key(7), || one_step("down1"));
        memos.ran(&key(7), 3);
        let generation = memos.generation(&key(7));
        memos.want(&key(7), 2);
        assert!(!memos.needs_run(&key(7), 3), "run again over the same xray");
        assert_eq!(memos.generation(&key(7)), generation);
        assert!(memos.plan(&key(7)).is_some());
    }

    /// **Two slots with one prefix share one memo**: wanted twice, it is one
    /// entry, run once.
    #[test]
    fn two_slots_with_one_prefix_share_one_memo() {
        let mut memos = Memos::default();
        memos.want(&key(7), 1);
        memos.want(&key(7), 1);
        memos.want(&key(9), 1);
        assert_eq!(memos.len(), 2);
    }

    /// **A memo is one per monitor and cut** (Ruling F30's key): the same
    /// prefix over another monitor's xray, or over the other cut of the same
    /// monitor, is a memo of its own.
    #[test]
    fn a_memo_is_one_per_monitor_and_cut() {
        let mut memos = Memos::default();
        memos.want(&key(7), 1);
        memos.want(
            &MemoKey {
                monitor: "DP-2".to_owned(),
                ..key(7)
            },
            1,
        );
        memos.want(
            &MemoKey {
                cut: Cut::Bottom,
                ..key(7)
            },
            1,
        );
        assert_eq!(memos.len(), 3);
    }

    /// **A memo's generation moves on each run**, which is what a T2
    /// suffix keys on.
    #[test]
    fn a_memos_generation_moves_on_each_run() {
        let mut memos = Memos::default();
        memos.want(&key(7), 1);
        memos.ran(&key(7), 1);
        let first = memos.generation(&key(7));
        memos.ran(&key(7), 2);
        assert_ne!(memos.generation(&key(7)), first);
    }

    /// **A memo keeps the plan it was first wanted with**: one key, one
    /// plan, since the key is the plan's content.
    #[test]
    fn a_memo_keeps_the_plan_it_was_first_wanted_with() {
        let mut memos = Memos::default();
        memos.want(&key(7), 1);
        assert!(memos.plan(&key(7)).is_none());
        memos.set_plan(&key(7), || crate::effect::memo::tests::one_step("down1"));
        memos.set_plan(&key(7), || unreachable!("built twice"));
        assert_eq!(memos.plan(&key(7)).map(|plan| plan.steps.len()), Some(1));
    }

    /// **A memo nothing wants this pass is given back** at the default
    /// `keep_passes = 0`.
    #[test]
    fn a_memo_no_slot_wanted_this_pass_is_dropped() {
        let (mut memos, mut pool) = (Memos::default(), crate::pool::Pool::new(0));
        memos.want(&key(7), 1);
        memos.want(&key(9), 2);
        memos.sweep(2, 0, &mut pool);
        assert_eq!(memos.len(), 1);
        assert!(memos.generation(&key(9)).is_some());
    }

    /// **`effects.memo.keep_passes` keeps an unwanted memo that long**
    /// (amended 2026-10-08, configurable): wanted at pass 1 and kept for 60
    /// passes, it is still held at pass 61 and given back at 62, so a part
    /// unwanted for a few passes (a covered `auto` bar) reads it again.
    #[test]
    fn a_memo_is_kept_for_keep_passes() {
        let (mut memos, mut pool) = (Memos::default(), crate::pool::Pool::new(0));
        memos.want(&key(7), 1);
        memos.sweep(61, 60, &mut pool);
        assert_eq!(memos.len(), 1, "given back before keep_passes ran out");
        memos.sweep(62, 60, &mut pool);
        assert_eq!(memos.len(), 0);
    }

    /// **`effects.memo.budget_mib` caps what memos hold** (amended
    /// 2026-10-08, configurable): a memo already held is always admitted,
    /// a new one only while the held bytes and its own fit; 0 is no cap.
    #[test]
    fn a_new_memo_over_the_budget_is_not_admitted() {
        let mib = 1024 * 1024;
        let mut memos = Memos::default();
        assert!(memos.admit(&key(7), 3 * mib / 4, 1));
        memos.want(&key(7), 1);
        assert!(!memos.admit(&key(9), mib / 2, 1), "over a 1 MiB budget");
        assert!(
            memos.admit(&key(7), 3 * mib / 4, 1),
            "a held memo was refused"
        );
        assert!(memos.admit(&key(9), mib / 2, 0), "0 is no cap");
        assert_eq!(memos.held_bytes(), 3 * mib / 4 + mib / 2);
    }

    /// **A held memo is counted at its latest size**: admitted again at
    /// another size (a new mode, another `effects.xray.scale`), its bytes
    /// follow, so a new memo is judged against what is held now.
    #[test]
    fn a_held_memo_is_counted_at_its_latest_size() {
        let mib = 1024 * 1024;
        let mut memos = Memos::default();
        assert!(memos.admit(&key(7), mib / 4, 1));
        assert!(memos.admit(&key(7), 3 * mib / 4, 1));
        assert_eq!(memos.held_bytes(), 3 * mib / 4);
        assert!(
            !memos.admit(&key(9), mib / 2, 1),
            "judged against the memo's first size"
        );
    }

    /// **What a memo holds is each step's target at its size**, rounded up
    /// as the run sizes it, 4 bytes a pixel and 8 in `rgba16f`: what
    /// `effects.memo.budget_mib` counts.
    #[test]
    fn a_memos_bytes_are_each_steps_target_at_its_size() {
        let mut plan = chain(&["down1", "up1"]);
        assert_eq!(
            super::plan_bytes(&plan, (101, 61)),
            (51 * 31 + 101 * 61) * 4
        );
        if let Some(up) = plan.steps.get_mut(1) {
            up.format = solium_effects::stage::Format::Rgba16f;
        }
        assert_eq!(
            super::plan_bytes(&plan, (101, 61)),
            51 * 31 * 4 + 101 * 61 * 8
        );
    }

    fn tree(lua: &str) -> crate::effect::tree::Tree {
        crate::effect::tree::tests::tree(lua).expect("a table")
    }

    /// **`effects.memo` is read and checked by name.**
    #[test]
    fn effects_memo_is_read_and_checked() {
        use solium_effects::stage::Split;
        assert_eq!(
            super::parse(&tree(
                r#"{ split = "longest", keep_passes = 60, budget_mib = 512 }"#
            )),
            Ok(super::MemoConfig {
                split: Split::Longest,
                keep_passes: 60,
                budget_mib: 512
            })
        );
        assert_eq!(super::parse(&tree("{}")), Ok(super::MemoConfig::default()));
        assert!(
            super::parse(&tree(r#"{ split = "short" }"#))
                .expect_err("refused")
                .contains("resize, longest or off")
        );
        assert!(
            super::parse(&tree("{ keep_passes = 4000 }"))
                .expect_err("refused")
                .contains("0 to 3600")
        );
        assert!(
            super::parse(&tree("{ budget_mib = 5000 }"))
                .expect_err("refused")
                .contains("0 to 4096")
        );
        assert!(
            super::parse(&tree("{ budgt_mb = 1 }"))
                .expect_err("refused")
                .contains("split, keep_passes or budget_mib")
        );
    }

    /// **`effects.memo` is a table of named keys and whole numbers**:
    /// anything else is refused naming the key (a number that is not finite
    /// included, as Ruling 28's `tree_of` keeps it), a whole number Lua
    /// holds as a float (`2^6`) is read, and a misspelt key is named with
    /// the nearest one.
    #[test]
    fn effects_memo_refuses_what_is_not_its_shape() {
        use crate::effect::tree::Tree;
        assert_eq!(
            super::parse(&tree("{ keep_passes = 2^6 }")).map(|config| config.keep_passes),
            Ok(64)
        );
        let not_finite = Tree::Table {
            list: Vec::new(),
            fields: [("budget_mib".to_owned(), Tree::Number(f64::INFINITY))].into(),
        };
        assert!(
            super::parse(&not_finite)
                .expect_err("refused")
                .contains("`effects.memo.budget_mib` is a whole number")
        );
        for (lua, says) in [
            ("true", "`effects.memo` is a table"),
            (r#"{ "resize" }"#, "takes named keys"),
            (
                "{ keep_passes = 1.5 }",
                "`effects.memo.keep_passes` is a whole number",
            ),
            (
                "{ keep_passes = -1 }",
                "`effects.memo.keep_passes` is 0 to 3600",
            ),
            (
                r#"{ budget_mib = "lots" }"#,
                "`effects.memo.budget_mib` is a whole number",
            ),
            (
                r#"{ split = 1 }"#,
                "`effects.memo.split` is resize, longest or off",
            ),
            ("{ budgt_mb = 1 }", "did you mean `budget_mib`?"),
        ] {
            let refused = super::parse(&tree(lua)).expect_err(lua);
            assert!(refused.contains(says), "{lua}: {refused}");
        }
    }

    /// **The T2 key is a struct with named fields** (Ruling F52): a suffix
    /// re-runs when any one of them moves, and two keys equal in every
    /// field are equal.
    #[test]
    fn a_t2_key_moves_with_each_of_its_fields() {
        let base = T2Key {
            memo: 1,
            xray: 4,
            padded: smithay::utils::Rectangle::new((10, 10).into(), (100, 40).into()),
            params: 2,
            shape: 3,
            own: None,
        };
        assert_eq!(base, T2Key { ..base });
        for moved in [
            T2Key { memo: 9, ..base },
            T2Key { xray: 9, ..base },
            T2Key {
                padded: smithay::utils::Rectangle::new((11, 10).into(), (100, 40).into()),
                ..base
            },
            T2Key { params: 9, ..base },
            T2Key { shape: 9, ..base },
            T2Key {
                own: Some(1),
                ..base
            },
        ] {
            assert_ne!(base, moved);
        }
    }
}
