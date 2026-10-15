//! A rule's slot fading in and out as the rule comes and goes: what a rule's
//! `fade` asks for (amended 2026-10-15, the glass shade, which is a `front`
//! slot on a window without the keyboard that comes and goes with the focus).
//!
//! **The fade is the compositor's and draws nothing new.** The slot's result is
//! composited at the opacity this works out, as a decoration layer's picture is
//! at its `layerOpacity`: the chain is not run again for it, and a slot that
//! has faded out is wanted no more, so it costs nothing until its rule comes
//! back. Pure: one pass's answer from the last, the rule the slot resolved to
//! now, and the pass's instant. `tests`.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use super::plan::Owner;
use super::rules::{Fade, RuleKey, Slot};

/// One slot on its way in or out: the rule whose result it shows, and the
/// opacity, `from` at `since` toward `to`, on `fade`'s curve.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Fading {
    key: RuleKey,
    from: f32,
    to: f32,
    since: Duration,
    fade: Fade,
}

impl Fading {
    /// The share of `fade.duration` this leg takes: the whole of it from 0 to
    /// 1, and as much of it as the way left when it turns round part of the
    /// way.
    fn span(&self) -> Duration {
        let share = f64::from((self.to - self.from).abs());
        Duration::try_from_secs_f64(self.fade.duration.as_secs_f64() * share)
            .unwrap_or(Duration::ZERO)
    }

    /// How opaque the slot is at `now`, clamped to 0..1 (a curve that
    /// overshoots, `outBack` or a spring, is held at the ends).
    fn opacity(&self, now: Duration) -> f32 {
        let span = self.span();
        if span.is_zero() {
            return self.to;
        }
        let progress = now.saturating_sub(self.since).as_secs_f64() / span.as_secs_f64();
        let eased = self.fade.curve.at(progress.clamp(0.0, 1.0));
        let at = f64::from(self.from) + f64::from(self.to - self.from) * eased;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "an opacity, clamped to 0..1"
        )]
        let at = at.clamp(0.0, 1.0) as f32;
        at
    }

    /// Whether this leg has arrived by `now`.
    fn done(&self, now: Duration) -> bool {
        now.saturating_sub(self.since) >= self.span()
    }

    /// Turn toward `to` from wherever the slot is at `now`.
    fn turn(&mut self, to: f32, now: Duration) {
        if (self.to - to).abs() > f32::EPSILON {
            self.from = self.opacity(now);
            self.to = to;
            self.since = now;
        }
    }
}

/// What a slot draws this pass: the rule whose result it is, and how opaque.
/// `tests::a_slot_fades_in_as_its_rule_comes_and_out_as_it_goes`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Shown {
    pub(crate) key: RuleKey,
    pub(crate) opacity: f32,
}

/// Every slot that is fading, or showing a rule that fades, by its owner and
/// slot. `tests`.
#[derive(Debug, Default)]
pub(crate) struct Fades {
    by_slot: HashMap<(Owner, Slot), Fading>,
    /// The slots stepped since the last [`Fades::sweep`].
    stepped: HashSet<(Owner, Slot)>,
}

impl Fades {
    /// What `(owner, slot)` draws this pass at `now`, given the rule it
    /// resolved to (`None` for none) with that rule's fade, and whether a
    /// rule is still bound (`bound`: a reload may have dropped it).
    ///
    /// A rule with no fade is shown whole, at once, and one that stops is
    /// gone at once, as before fades. A rule with one comes in from 0, and
    /// when it stops the slot goes on showing its result, fading to 0, and
    /// is wanted no more once there. Turned round part of the way, it goes
    /// back from where it got to. Another rule taking the slot, or the same
    /// one bound again by a reload, takes it over where the fade had got to.
    /// `tests::a_slot_fades_in_as_its_rule_comes_and_out_as_it_goes`,
    /// `tests::a_fade_turned_round_goes_back_from_where_it_got_to`,
    /// `tests::a_rule_without_a_fade_comes_and_goes_at_once`,
    /// `tests::a_reloaded_rule_takes_its_slot_over_where_the_fade_had_got`.
    pub(crate) fn step(
        &mut self,
        owner: &Owner,
        slot: Slot,
        resolved: Option<(RuleKey, Option<Fade>)>,
        bound: impl Fn(RuleKey) -> bool,
        now: Duration,
    ) -> Option<Shown> {
        // Nothing held and nothing to fade: the slot is its rule, as before
        // fades, and nothing is cloned or kept for it.
        if self.by_slot.is_empty() && !matches!(resolved, Some((_, Some(_)))) {
            return resolved.map(|(key, _)| Shown { key, opacity: 1.0 });
        }
        let at = (owner.clone(), slot);
        match resolved {
            Some((key, None)) => {
                self.by_slot.remove(&at);
                Some(Shown { key, opacity: 1.0 })
            }
            Some((key, Some(fade))) => {
                self.stepped.insert(at.clone());
                let fading = self.by_slot.entry(at).or_insert(Fading {
                    key,
                    from: 0.0,
                    to: 1.0,
                    since: now,
                    fade,
                });
                fading.key = key;
                fading.fade = fade;
                fading.turn(1.0, now);
                Some(Shown {
                    key,
                    opacity: fading.opacity(now),
                })
            }
            None => {
                let fading = self.by_slot.get_mut(&at)?;
                self.stepped.insert(at.clone());
                if !bound(fading.key) {
                    self.by_slot.remove(&at);
                    return None;
                }
                fading.turn(0.0, now);
                if fading.done(now) {
                    self.by_slot.remove(&at);
                    return None;
                }
                Some(Shown {
                    key: fading.key,
                    opacity: fading.opacity(now),
                })
            }
        }
    }

    /// Whether a slot is still on its way in or out at `now`: the frames that
    /// show it are owed, since nothing else on screen may change to draw them.
    /// `tests::a_slot_fades_in_as_its_rule_comes_and_out_as_it_goes`.
    pub(crate) fn moving(&self, now: Duration) -> bool {
        self.by_slot.values().any(|fading| !fading.done(now))
    }

    /// At the end of a pass: keep the slots stepped in it, and of the rest
    /// only those whose owner `keep` says is still there, though not drawn
    /// this pass (a window on a workspace out of sight).
    /// `tests::a_gone_owners_fades_are_dropped`.
    pub(crate) fn sweep(&mut self, mut keep: impl FnMut(&Owner) -> bool) {
        self.by_slot
            .retain(|at, _| self.stepped.contains(at) || keep(&at.0));
        self.stepped.clear();
    }

    /// Forget every slot: no rule is left to fade.
    pub(crate) fn clear(&mut self) {
        self.by_slot.clear();
        self.stepped.clear();
    }

    /// How many slots are held. Nothing in the compositor asks.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.by_slot.len()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Fades, Shown};
    use crate::effect::plan::{Owner, PaneSlot};
    use crate::effect::rules::{Fade, Origin, RuleKey, Slot};
    use crate::pane::PaneId;

    fn key(index: u32, generation: u32) -> RuleKey {
        RuleKey {
            origin: Origin::Style,
            index,
            generation,
        }
    }

    fn linear(ms: u64) -> Fade {
        Fade {
            duration: Duration::from_millis(ms),
            curve: solium_animation::Curve::Linear,
        }
    }

    fn ms(ms: u64) -> Duration {
        Duration::from_secs(1000) + Duration::from_millis(ms)
    }

    fn owner(pane: u64) -> Owner {
        Owner::Pane(PaneId::from_raw(pane), PaneSlot::Pane)
    }

    fn opacity(shown: Option<Shown>) -> Option<f32> {
        shown.map(|shown| (shown.opacity * 100.0).round() / 100.0)
    }

    /// **A slot fades in as its rule comes and out as it goes**: from 0 to 1
    /// over the fade's 200 ms, then held; and when the rule stops, its result
    /// still shown, from 1 to 0 over 200 ms, and then not at all, and nothing
    /// held for it. Frames are owed while it moves and not once it rests.
    #[test]
    fn a_slot_fades_in_as_its_rule_comes_and_out_as_it_goes() {
        let mut fades = Fades::default();
        let (pane, shade) = (owner(1), Some((key(1, 1), Some(linear(200)))));
        let bound = |_| true;
        let seen: Vec<Option<f32>> = [0, 50, 100, 200, 400]
            .into_iter()
            .map(|at| opacity(fades.step(&pane, Slot::Front, shade, bound, ms(at))))
            .collect();
        assert_eq!(
            seen,
            [Some(0.0), Some(0.25), Some(0.5), Some(1.0), Some(1.0)]
        );
        assert!(fades.moving(ms(150)) && !fades.moving(ms(400)));
        let seen: Vec<Option<f32>> = [1000, 1050, 1150, 1200, 1300]
            .into_iter()
            .map(|at| opacity(fades.step(&pane, Slot::Front, None, bound, ms(at))))
            .collect();
        assert_eq!(seen, [Some(1.0), Some(0.75), Some(0.25), None, None]);
        assert_eq!(fades.len(), 0, "a slot faded out is held no more");
        assert!(!fades.moving(ms(1300)));
        let shown = fades.step(&pane, Slot::Front, shade, bound, ms(2000));
        assert_eq!(shown.map(|shown| shown.key), Some(key(1, 1)));
    }

    /// **A fade turned round goes back from where it got to**, in that share
    /// of its duration: out at 0.75 when its rule stops 150 ms into a 200 ms
    /// fade in, gone 150 ms later; and in again from where the fade out had
    /// got to.
    #[test]
    fn a_fade_turned_round_goes_back_from_where_it_got_to() {
        let mut fades = Fades::default();
        let (pane, shade) = (owner(1), Some((key(1, 1), Some(linear(200)))));
        let bound = |_| true;
        let _ = fades.step(&pane, Slot::Front, shade, bound, ms(0));
        let out: Vec<Option<f32>> = [150, 225, 290, 300]
            .into_iter()
            .map(|at| opacity(fades.step(&pane, Slot::Front, None, bound, ms(at))))
            .collect();
        assert_eq!(out, [Some(0.75), Some(0.38), Some(0.05), None]);
        let _ = fades.step(&pane, Slot::Front, shade, bound, ms(400));
        let _ = fades.step(&pane, Slot::Front, None, bound, ms(500));
        let back: Vec<Option<f32>> = [550, 600]
            .into_iter()
            .map(|at| opacity(fades.step(&pane, Slot::Front, shade, bound, ms(at))))
            .collect();
        assert_eq!(back, [Some(0.25), Some(0.5)]);
    }

    /// **A rule without a fade comes and goes at once**, as every rule did
    /// before fades, and holds nothing.
    #[test]
    fn a_rule_without_a_fade_comes_and_goes_at_once() {
        let mut fades = Fades::default();
        let pane = owner(1);
        let shown = fades.step(
            &pane,
            Slot::Behind,
            Some((key(0, 1), None)),
            |_| true,
            ms(0),
        );
        assert_eq!(
            shown,
            Some(Shown {
                key: key(0, 1),
                opacity: 1.0
            })
        );
        assert_eq!(fades.step(&pane, Slot::Behind, None, |_| true, ms(1)), None);
        assert_eq!(fades.len(), 0);
        // And one whose fade is 0 ms is there whole from its first frame.
        let instant = Some((key(0, 1), Some(linear(0))));
        assert_eq!(
            opacity(fades.step(&pane, Slot::Behind, instant, |_| true, ms(2))),
            Some(1.0)
        );
        assert_eq!(fades.step(&pane, Slot::Behind, None, |_| true, ms(3)), None);
    }

    /// **A reloaded rule takes its slot over where the fade had got to**: a
    /// reload binds every rule again under a new generation, and the slot
    /// goes on from the opacity it had, rather than fading in again; a rule
    /// gone with the reload takes its slot's result with it at once.
    #[test]
    fn a_reloaded_rule_takes_its_slot_over_where_the_fade_had_got() {
        let mut fades = Fades::default();
        let pane = owner(1);
        let _ = fades.step(
            &pane,
            Slot::Front,
            Some((key(1, 1), Some(linear(200)))),
            |_| true,
            ms(0),
        );
        let _ = fades.step(
            &pane,
            Slot::Front,
            Some((key(1, 1), Some(linear(200)))),
            |_| true,
            ms(300),
        );
        let reloaded = fades.step(
            &pane,
            Slot::Front,
            Some((key(1, 2), Some(linear(200)))),
            |key| key.generation == 2,
            ms(310),
        );
        assert_eq!(
            reloaded,
            Some(Shown {
                key: key(1, 2),
                opacity: 1.0
            })
        );
        let _ = fades.step(&pane, Slot::Front, None, |_| true, ms(400));
        assert_eq!(
            fades.step(&pane, Slot::Front, None, |key| key.generation == 3, ms(450)),
            None,
            "the reload dropped the rule that was fading out"
        );
    }

    /// **A gone owner's fades are dropped**, and only its own: a slot
    /// stepped this pass is kept, and one that was not is kept while its
    /// owner is there.
    #[test]
    fn a_gone_owners_fades_are_dropped() {
        let mut fades = Fades::default();
        let shade = Some((key(1, 1), Some(linear(200))));
        for pane in [1, 2, 3] {
            let _ = fades.step(&owner(pane), Slot::Front, shade, |_| true, ms(0));
        }
        fades.sweep(|_| false);
        assert_eq!(fades.len(), 3, "every slot was stepped this pass");
        let _ = fades.step(&owner(3), Slot::Front, shade, |_| true, ms(50));
        fades.sweep(|owner| *owner == self::owner(2));
        assert_eq!(fades.len(), 2, "pane 1 is gone, 2 is out of sight");
        assert_eq!(
            opacity(fades.step(&owner(2), Slot::Front, shade, |_| true, ms(100))),
            Some(0.5)
        );
    }
}
