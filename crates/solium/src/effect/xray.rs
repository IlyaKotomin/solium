//! xray (Ruling F29): each monitor's layers below a part's band, captured
//! once a pass at most and read by every T2 slot on that monitor as its
//! `backdrop`. Task 43 builds it; the memo is keyed by its cut.
//! `effect::memo::tests::a_memo_is_one_per_monitor_and_cut`.

/// Which of a monitor's layers an xray holds: a part reads the xray layers
/// strictly below its own band, so a part on `bottom` reads the background
/// alone and every part above it reads the background and bottom layers,
/// at most two cuts a monitor (Ruling F29).
/// `effect::memo::tests::a_memo_is_one_per_monitor_and_cut`.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Task 43 builds a cut's capture, and Task 44 keys memos by it"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Cut {
    /// The background layer.
    Background,
    /// The background and bottom layers.
    Bottom,
}
