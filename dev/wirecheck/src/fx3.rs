//! FX3's GPU cases: the memo a T2 slot reads and the crop of it a part draws,
//! which no unit test can reach. Run alone with `WIRECHECK_ONLY=fx3`, and after
//! FX2's cases in a full run. Each prints a `=== FX3:` heading. The effect
//! programs, the executor and the effect element are the compositor's own, as
//! FX2's cases include them (`crate::fx2::{gl, run, element}`).

use anyhow::{Result, anyhow};
use smithay::backend::renderer::Texture as _;
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::utils::Rectangle;
use solium_effects::stage::{self, Split};

use crate::fx0::pool;
use crate::fx2::{self, element, run};

/// The monitor 13a's xray covers, in pixels.
const MONITOR: (usize, usize) = (640, 360);

/// The part 13a and 13b crop: x, y, w, h. Off the monitor's centre in both
/// directions, so a crop whose y were measured from the bottom edge (rows 240
/// to 320) reads other pixels, which 13b checks it would.
const PART: (usize, usize, usize, usize) = (200, 40, 120, 80);

/// The shipped blur's `passes`, `offset` and reach (`offset · 2^(passes + 1)`,
/// its `effect.lua`'s), at its defaults.
const PASSES: u32 = 3;
const OFFSET: f64 = 3.0;
const REACH: usize = 48;

/// Every FX3 case, in order.
pub(crate) fn all(renderer: &mut GlesRenderer) -> Result<()> {
    let memo = the_memo_matches_a_whole_monitor_blur(renderer)?;
    a_crop_of_the_memo_draws_its_own_pixels(renderer, &memo)?;
    Ok(())
}

/// 13a's xray at `(x, y)`, anywhere, the monitor's corner at (0, 0): a
/// checker of 64-pixel cells, 0 and 255 at a different phase in each
/// channel. Cells this size keep most of their contrast through the shipped
/// blur, whose kernel is some 17 pixels wide: 8 or 16-pixel cells come out
/// grey within one everywhere, so a result read from the wrong place, or not
/// blurred at all where it should be, would agree with the reference anyway.
fn checker(x: i64, y: i64) -> [u8; 4] {
    const PHASES: [(i64, i64); 4] = [(0, 0), (21, 43), (43, 11), (11, 53)];
    std::array::from_fn(|k| {
        let (px, py) = PHASES[k];
        if ((x + px).div_euclid(64) + (y + py).div_euclid(64)).rem_euclid(2) == 0 {
            255
        } else {
            0
        }
    })
}

/// `bytes` (RGBA, rows of `width`) inside `(x, y, w, h)`, row by row.
fn sub(bytes: &[u8], width: usize, (x, y, w, h): (usize, usize, usize, usize)) -> Vec<u8> {
    (y..y + h)
        .flat_map(|row| {
            let at = (row * width + x) * 4;
            bytes.get(at..at + w * 4).unwrap_or_default().to_vec()
        })
        .collect()
}

/// The largest channel difference between two pictures of the same size.
fn worst(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b)
        .map(|(p, q)| p.abs_diff(*q))
        .max()
        .unwrap_or(0)
}

/// What 13a leaves 13b: the memo as drawn on the GPU, read back.
struct Memo {
    texture: GlesTexture,
    bytes: Vec<u8>,
}

/// **Case 13a: the memo matches a whole-monitor CPU blur.** The shipped
/// `blur/` at `passes = 3, offset = 3`, flattened as the host flattens it and
/// split as the compositor splits it (`stage::split_memo`, `Split::Resize`):
/// its prefix is the whole plan, its suffix empty, so every part reads a crop
/// of the memo and runs nothing. The prefix runs as `render::run_xray` runs a
/// memo: at 640×360, `backdrop` the xray (a checker of 64-pixel cells, a
/// different phase in each channel) through `BoxMap::WHOLE`. Read back, every
/// channel is within 1 of `fx2::cpu_kawase` over the **whole** picture, and
/// so is the part's 120×80 box at (200, 40) of each (a crop is the memo's own
/// pixels). Printed, not asserted: the largest difference between that crop
/// and a per-part run of the same blur over the box padded by its reach,
/// which clamps where the memo reads real pixels ([R] §2): why 13a holds the
/// memo to the whole monitor and never to a per-part run.
fn the_memo_matches_a_whole_monitor_blur(renderer: &mut GlesRenderer) -> Result<Memo> {
    println!("\n=== FX3: the memo matches a whole-monitor CPU blur within one ===");
    let side = |n: usize| i64::try_from(n).unwrap_or(0);
    let (xray, xray_bytes) = fx2::upload(renderer, MONITOR, |x, y| checker(side(x), side(y)))?;
    let (plan, programs) = fx2::build(renderer, fx2::shipped_blur(PASSES, OFFSET))?;
    let split = stage::split_memo(&plan, Split::Resize);
    let (Some(prefix), [export]) = (split.prefix.as_ref(), split.exports.as_slice()) else {
        return Err(anyhow!(
            "the shipped blur did not split into a prefix and one export: {split:?}"
        ));
    };
    if !split.suffix.steps.is_empty() || export.step + 1 != plan.steps.len() {
        return Err(anyhow!(
            "the shipped blur's memo is not its whole plan: export {export:?}, suffix {:?}",
            split.suffix.steps
        ));
    }
    // What `want_t2` hands the memo, and `run_xray` runs.
    let memo_plan = prefix.upto(export.step);
    let mut pool = pool::Pool::new(0);
    let mut held = run::Held::default();
    let textures = [("backdrop", xray, run::BoxMap::WHOLE)];
    let outcome = fx2::run_once(
        renderer,
        &mut pool,
        &|key| fx2::lookup(&programs, key),
        &memo_plan,
        &mut held,
        &fx2::whole(MONITOR, &textures),
    )?;
    let run::Outcome::Done(texture, _) = outcome else {
        return Err(anyhow!("the memo's run came to {outcome:?}"));
    };
    if texture.size() != (640, 360).into() {
        return Err(anyhow!("the memo is {:?}, not 640×360", texture.size()));
    }
    let bytes = fx2::read_rgba(renderer, &texture)?;
    let cpu = fx2::cpu_kawase(&plan, &xray_bytes, MONITOR, OFFSET);
    // Rounded as an rgba8 target stores it, each pass's result already is.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "0 to 255"
    )]
    let reference: Vec<u8> = cpu
        .pixels
        .iter()
        .flat_map(|pixel| pixel.map(|channel| channel as u8))
        .collect();
    let mut off = (0_u8, (0, 0));
    for (at, (got, want)) in bytes
        .chunks_exact(4)
        .zip(reference.chunks_exact(4))
        .enumerate()
    {
        let here = worst(got, want);
        if here > off.0 {
            off = (here, (at % MONITOR.0, at / MONITOR.0));
        }
    }
    if off.0 > 1 {
        return Err(anyhow!(
            "the memo differs from the whole-monitor CPU blur by {} at {:?}",
            off.0,
            off.1
        ));
    }
    let cropped = worst(
        &sub(&bytes, MONITOR.0, PART),
        &sub(&reference, MONITOR.0, PART),
    );
    if cropped > 1 {
        return Err(anyhow!(
            "the part's crop differs from the reference's by {cropped}"
        ));
    }
    let per_part = per_part_run(renderer, &programs, &plan)?;
    let edge = worst(&sub(&bytes, MONITOR.0, PART), &per_part);
    held.release(&mut pool);
    pool.sweep(renderer);
    fx2::free(renderer, programs)?;
    println!(
        "  three down and three up over 640×360: at most {} from the CPU's, at {:?}; the part's \
         120×80 crop at most {cropped}; a per-part run over the part padded by its reach differs \
         from the crop by up to {edge} (printed, not judged: it clamps where the memo reads on)",
        off.0, off.1
    );
    Ok(Memo { texture, bytes })
}

/// The blur run per part, as a part reads it when its memo is not held: its
/// whole plan over the part's box padded by the blur's reach, the xray's own
/// pixels there, clamped at that box's edge; the part's 120×80 of the result.
fn per_part_run(
    renderer: &mut GlesRenderer,
    programs: &std::collections::HashMap<u64, fx2::gl::Program>,
    plan: &solium_effects::stage::Plan,
) -> Result<Vec<u8>> {
    let side = |n: usize| i64::try_from(n).unwrap_or(0);
    let padded = (PART.2 + 2 * REACH, PART.3 + 2 * REACH);
    let corner = (side(PART.0) - side(REACH), side(PART.1) - side(REACH));
    let (input, _) = fx2::upload(renderer, padded, |x, y| {
        checker(corner.0 + side(x), corner.1 + side(y))
    })?;
    let mut pool = pool::Pool::new(0);
    let mut held = run::Held::default();
    let textures = [("backdrop", input, run::BoxMap::WHOLE)];
    let outcome = fx2::run_once(
        renderer,
        &mut pool,
        &|key| fx2::lookup(programs, key),
        plan,
        &mut held,
        &fx2::whole(padded, &textures),
    )?;
    let run::Outcome::Done(result, _) = outcome else {
        return Err(anyhow!("the per-part run came to {outcome:?}"));
    };
    let bytes = fx2::read_rgba(renderer, &result)?;
    drop(result);
    held.release(&mut pool);
    pool.sweep(renderer);
    Ok(sub(&bytes, padded.0, (REACH, REACH, PART.2, PART.3)))
}

/// **Case 13b: a crop of the memo draws its own pixels.** The compositor's
/// own `EffectElement` over 13a's memo, cropped to the part's box and then
/// placed under a mask, as the walk places a slot whose suffix is empty with
/// its part's mask (`EffectElement::new(..).cropped((200, 40, 120, 80))
/// .at((0, 0, 120, 80), Some(the whole box, square), 1.0)`): a mask is what
/// sends the draw through `MASKED_TEXTURE`, compiled as `Programs::masked`
/// compiles it, with the crop mapping the mask (smithay draws one with no
/// mask through its own program). Drawn by `pool::paint` into a 120×80
/// pooled target and read back: every channel within 1 of the memo's own
/// pixels in that box, so the mask, mapped through the crop, covers all of
/// it. The box is off the centre, and the memo's rows 240 to 320 (the box
/// measured from the bottom edge) differ from its rows 40 to 120 by far more
/// than 1, checked here, so a crop read upside down cannot pass. Case 12h
/// cuts under a crop.
fn a_crop_of_the_memo_draws_its_own_pixels(renderer: &mut GlesRenderer, memo: &Memo) -> Result<()> {
    use smithay::backend::renderer::Bind as _;
    use smithay::backend::renderer::Frame as _;
    use smithay::backend::renderer::element::Id;
    use smithay::backend::renderer::utils::CommitCounter;
    use smithay::utils::Physical;
    use solium_effects::fragment::{Corners, MASKED_TEXTURE};
    println!("\n=== FX3: a crop of the memo draws its own pixels ===");
    let want = sub(&memo.bytes, MONITOR.0, PART);
    let mirrored = (PART.0, MONITOR.1 - PART.1 - PART.3, PART.2, PART.3);
    let apart = worst(&want, &sub(&memo.bytes, MONITOR.0, mirrored));
    if apart <= 8 {
        return Err(anyhow!(
            "the box and its mirror in y differ by only {apart}, so a crop upside down would pass"
        ));
    }
    let program = renderer
        .compile_custom_texture_shader(
            MASKED_TEXTURE,
            &crate::uniforms::registration(MASKED_TEXTURE),
        )
        .map_err(|err| anyhow!("MASKED_TEXTURE did not compile in every variant: {err}"))?;
    let to_f64 = |n: usize| f64::from(u32::try_from(n).unwrap_or(0));
    let to_i32 = |n: usize| i32::try_from(n).unwrap_or(0);
    let crop = Rectangle::new(
        (to_f64(PART.0), to_f64(PART.1)).into(),
        (to_f64(PART.2), to_f64(PART.3)).into(),
    );
    let side = (to_i32(PART.2), to_i32(PART.3));
    // The whole box, square: every pixel's centre half a pixel or more
    // inside it, so fully covered.
    let mask =
        Rectangle::<f64, Physical>::new((0.0, 0.0).into(), (to_f64(PART.2), to_f64(PART.3)).into());
    // Cropped, then placed under the mask, as the walk places a memo's crop.
    let placed = element::EffectElement::new(
        Id::new(),
        CommitCounter::default(),
        memo.texture.clone(),
        program,
    )
    .cropped(crop)
    .at(
        Rectangle::from_size(side.into()),
        Some((mask, Corners::all(0.0))),
        1.0,
    );
    let mut pool = pool::Pool::new(0);
    let mut carrier = pool
        .carrier(renderer)
        .ok_or_else(|| anyhow!("no carrier"))?;
    let target = pool
        .target(&mut pool::Gl(renderer), side.into(), pool::Format::Rgba8)
        .ok_or_else(|| anyhow!("no pooled target"))?;
    {
        let mut bound = renderer
            .bind(&mut carrier)
            .map_err(|err| anyhow!("{err}"))?;
        let mut frame =
            pool::frame_for(renderer, &mut bound, &target).map_err(|err| anyhow!("{err}"))?;
        pool::paint(&mut frame, side.into(), &[placed], 1.0).map_err(|err| anyhow!("{err}"))?;
        frame
            .finish()
            .map_err(|err| anyhow!("{err}"))?
            .wait()
            .map_err(|err| anyhow!("{err:?}"))?;
    }
    let drawn = fx2::read_rgba(renderer, target.texture())?;
    drop(target);
    pool.sweep(renderer);
    let off = worst(&drawn, &want);
    if drawn.len() != want.len() || off > 1 {
        let at = drawn
            .chunks_exact(4)
            .zip(want.chunks_exact(4))
            .position(|(got, wanted)| worst(got, wanted) > 1)
            .map(|at| (at % PART.2, at / PART.2));
        return Err(anyhow!(
            "the memo's crop drawn differs from the memo's own pixels in that box by {off}, first at {at:?}"
        ));
    }
    println!(
        "  the 120×80 crop at (200, 40), drawn by smithay: at most {off} from the memo there; \
         its mirror in y differs by {apart}; through MASKED_TEXTURE under a mask of the box"
    );
    Ok(())
}
