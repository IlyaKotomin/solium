// The real independent mobile entry, loaded once and then with kept state.
// A fresh VM initially has no snapshot: that is not an empty live desktop.
fn mobile_reload_fixture() -> (Scripts, Snapshot, std::path::PathBuf) {
    let entry = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/lua/mobile.lua"));
    let mut scripts = Scripts::load(&entry).expect("mobile entry loads");
    let snapshot = Snapshot {
        monitors: one_screen(),
        windows: vec![window(1, "DP-1", WIDE), window(2, "DP-1", WIDE)],
        work_area: WIDE,
        ..Snapshot::default()
    };
    scripts.monitors_changed(snapshot.clone());
    scripts.opened(1, snapshot.clone());
    scripts.opened(2, snapshot.clone());
    (scripts, snapshot, entry)
}

#[test]
fn mobile_reload_keeps_the_recents_page_and_task_order() {
    let (mut scripts, snapshot, entry) = mobile_reload_fixture();
    scripts.key("super+space", snapshot.clone());
    assert_eq!(
        scripts.evaluate("return sol.keep('mobile.v1', {}).page"),
        "recents"
    );
    let before = scripts.evaluate("return table.concat(sol.keep('mobile.v1', {}).order, ',')");
    let mut fresh = Scripts::load_carrying(&entry, scripts.kept()).expect("mobile reloads");
    fresh.restored(snapshot);
    assert_eq!(
        fresh.evaluate("return sol.keep('mobile.v1', {}).page"),
        "recents"
    );
    assert_eq!(
        fresh.evaluate("return table.concat(sol.keep('mobile.v1', {}).order, ',')"),
        before
    );
}

#[test]
fn mobile_reload_keeps_a_tablets_split_and_active_task() {
    let (mut scripts, snapshot, entry) = mobile_reload_fixture();
    scripts.key("super+shift+s", snapshot.clone());
    assert_eq!(
        scripts.evaluate("return tostring(#sol.keep('mobile.v1', {}).split)"),
        "2"
    );
    let before = scripts.evaluate("return tostring(sol.keep('mobile.v1', {}).active)");
    let mut fresh = Scripts::load_carrying(&entry, scripts.kept()).expect("mobile reloads");
    fresh.restored(snapshot);
    assert_eq!(
        fresh.evaluate("return tostring(#sol.keep('mobile.v1', {}).split)"),
        "2"
    );
    assert_eq!(
        fresh.evaluate("return tostring(sol.keep('mobile.v1', {}).active)"),
        before
    );
}

fn mobile_action(
    scripts: &mut Scripts,
    snapshot: &Snapshot,
    action: &str,
    data: &str,
) -> super::Outcome {
    scripts.surface_action(
        "mobile-shell",
        &format!("mobile.{action}"),
        &crate::json::Json::parse(data).expect("mobile action JSON"),
        snapshot.clone(),
    )
}

#[test]
fn mobile_drag_moves_real_panes_then_snaps_without_reordering_tasks() {
    let (mut scripts, snapshot, _) = mobile_reload_fixture();
    scripts.key("super+space", snapshot.clone());
    let before = scripts.evaluate("return table.concat(sol.keep('mobile.v1', {}).order, ',')");
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.begin",
        r#"{"x":400,"y":250}"#,
    );
    let dragging = mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":-350,"dy":0}"#,
    );
    assert!(
        dragging
            .commands
            .iter()
            .filter(|command| matches!(command,
                Command::Present { animation, .. } if animation.duration.is_zero()
            ))
            .count()
            >= 2,
        "every live card follows the pointer without a second animation"
    );
    mobile_action(&mut scripts, &snapshot, "swipe.end", "{}");
    assert_eq!(
        scripts.evaluate("return tostring(sol.keep('mobile.v1', {}).active)"),
        "2"
    );
    assert_eq!(
        scripts.evaluate("return table.concat(sol.keep('mobile.v1', {}).order, ',')"),
        before
    );
}

#[test]
fn mobile_swipe_up_closes_a_task_without_activating_its_client() {
    let (mut scripts, snapshot, _) = mobile_reload_fixture();
    scripts.key("super+space", snapshot.clone());
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.begin",
        r#"{"x":400,"y":250}"#,
    );
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":2,"dy":-200}"#,
    );
    let closed = mobile_action(&mut scripts, &snapshot, "swipe.end", "{}");
    assert!(
        closed
            .commands
            .iter()
            .any(|c| matches!(c, Command::Close { id: 2 }))
    );
    assert!(
        !closed
            .commands
            .iter()
            .any(|c| matches!(c, Command::Focus { .. }))
    );
    assert_eq!(
        scripts.evaluate("return sol.keep('mobile.v1', {}).page"),
        "recents"
    );
}

#[test]
fn mobile_launch_origin_is_used_for_the_loading_pane_once() {
    let (mut scripts, mut snapshot, _) = mobile_reload_fixture();
    scripts.key("super+h", snapshot.clone());
    mobile_action(
        &mut scripts,
        &snapshot,
        "launch",
        r#"{"app":"notes","origin":{"x":24,"y":70,"size":60}}"#,
    );
    let mut placeholder = window(3, "DP-1", WIDE);
    placeholder.title = "qml".to_owned();
    snapshot.windows.push(placeholder);
    let opened = scripts.opened(3, snapshot.clone());
    assert!(
        opened.commands.iter().any(|c| matches!(c,
            Command::PresentFrom { id: 3, rect, .. }
                if rect.x == 24.0 && rect.y == 70.0 && rect.w == 60.0 && rect.h == 60.0
        )),
        "the loading pane, later adopted by the application, grows from its actual icon"
    );
    snapshot.windows.push(window(4, "DP-1", WIDE));
    let unrelated = scripts.opened(4, snapshot);
    assert!(
        !unrelated.commands.iter().any(|c| matches!(c,
            Command::PresentFrom { rect, .. } if rect.w == 60.0
        )),
        "an unrelated later window must not inherit the icon"
    );
}

#[test]
fn mobile_shade_preserves_foreground_geometry_and_keeps_settings_on_reload() {
    let (mut scripts, snapshot, entry) = mobile_reload_fixture();
    let shade = mobile_action(&mut scripts, &snapshot, "shade", "{}");
    assert!(
        !shade
            .commands
            .iter()
            .any(|c| matches!(c, Command::Present { id: 2, .. })),
        "opening Control Center keeps the foreground client in its real slot"
    );
    mobile_action(
        &mut scripts,
        &snapshot,
        "setting",
        r#"{"key":"brightness","value":0.6}"#,
    );
    mobile_action(&mut scripts, &snapshot, "setting", r#"{"key":"dark"}"#);
    let mut fresh = Scripts::load_carrying(&entry, scripts.kept()).expect("mobile reloads");
    fresh.restored(snapshot);
    assert_eq!(
        fresh.evaluate("return tostring(sol.keep('mobile.v1', {}).settings.brightness)"),
        "0.6"
    );
    assert_eq!(
        fresh.evaluate("return tostring(sol.keep('mobile.v1', {}).settings.dark)"),
        "true"
    );
}

#[test]
fn mobile_rounds_only_overview_and_does_not_rebuild_frames_while_swiping() {
    let (mut scripts, snapshot, _) = mobile_reload_fixture();
    let overview = scripts.key("super+space", snapshot.clone());
    assert!(overview.commands.iter().any(|c| matches!(c,
        Command::Decoration { name: Some(name) } if name == "mobile"
    )));
    let drag = mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":-100,"dy":0}"#,
    );
    assert!(
        !drag
            .commands
            .iter()
            .any(|c| matches!(c, Command::Decoration { .. }))
    );
    let shade = mobile_action(&mut scripts, &snapshot, "shade", "{}");
    assert!(
        !shade
            .commands
            .iter()
            .any(|c| matches!(c, Command::Decoration { .. })),
        "a shade over overview preserves its rounded panes"
    );
    mobile_action(&mut scripts, &snapshot, "back", "{}");
    let opened = mobile_action(&mut scripts, &snapshot, "select", r#"{"id":2}"#);
    assert!(
        opened.commands.iter().any(|c| matches!(c,
            Command::Decoration { name: Some(name) } if name == "mobile-flat"
        )),
        "an opened app must have square corners again"
    );
    let app_shade = mobile_action(&mut scripts, &snapshot, "shade", "{}");
    assert!(
        !app_shade
            .commands
            .iter()
            .any(|c| matches!(c, Command::Decoration { .. })),
        "a shade over an app keeps the app square"
    );
}

#[test]
fn mobile_card_depth_stays_fixed_across_drag_and_selection() {
    let (mut scripts, snapshot, _) = mobile_reload_fixture();
    let depth = |outcome: &super::Outcome| -> std::collections::BTreeMap<u64, i32> {
        outcome
            .commands
            .iter()
            .filter_map(|c| match c {
                Command::Present { id, z, .. } => Some((*id, *z as i32)),
                _ => None,
            })
            .collect()
    };
    let entered = scripts.key("super+space", snapshot.clone());
    let before = depth(&entered);
    assert!(
        before[&1] > before[&2],
        "the right card covers its left neighbour"
    );
    let moving = mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":-250,"dy":0}"#,
    );
    assert_eq!(depth(&moving), before);
    let snapped = mobile_action(&mut scripts, &snapshot, "swipe.end", "{}");
    assert_eq!(
        depth(&snapped),
        before,
        "centering another card must not raise it"
    );
}

#[test]
fn mobile_swipe_close_lands_wholly_outside_before_native_close_deadline() {
    let (mut scripts, mut snapshot, _) = mobile_reload_fixture();
    scripts.key("super+space", snapshot.clone());
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.begin",
        r#"{"x":400,"y":250}"#,
    );
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":0,"dy":-200}"#,
    );
    mobile_action(&mut scripts, &snapshot, "swipe.end", "{}");
    snapshot.windows[1].leaving = true;
    let leaving = scripts.closing(2, snapshot);
    assert!(
        leaving.commands.iter().any(|c| matches!(c,
            Command::Present { id: 2, rect: Some(rect), opacity: Some(alpha), animation, .. }
                if rect.y + rect.h < 0.0 && *alpha == 1.0
                    && animation.duration <= crate::present::CLOSING
        )),
        "the entire opaque card must leave before the native close is sent"
    );
}

#[test]
fn mobile_home_uses_visible_icon_and_falls_below_screen_without_one() {
    let (mut scripts, mut snapshot, _) = mobile_reload_fixture();
    snapshot.windows[0].title = "Notes".to_owned();
    scripts.key("super+h", snapshot.clone());
    mobile_action(
        &mut scripts,
        &snapshot,
        "icons",
        r#"{"epoch":1,"w":2560,"h":1440,"items":{"notes":{"x":24,"y":70,"size":60}}}"#,
    );
    mobile_action(&mut scripts, &snapshot, "select", r#"{"id":1}"#);
    let home = scripts.key("super+h", snapshot.clone());
    assert!(home.commands.iter().any(|c| matches!(c,
        Command::Present { id: 1, rect: Some(rect), opacity: Some(alpha), .. }
            if rect.x == 24.0 && rect.y == 70.0 && rect.w == 60.0 && *alpha == 0.0
    )));
    mobile_action(&mut scripts, &snapshot, "select", r#"{"id":2}"#);
    let unknown = scripts.key("super+h", snapshot);
    assert!(
        unknown.commands.iter().any(|c| matches!(c,
            Command::Present { id: 2, rect: Some(rect), opacity: Some(alpha), .. }
                if rect.y > 1440.0 && rect.w == 48.0 && *alpha == 0.0
        )),
        "an unknown app leaves downwards rather than crossing the desktop"
    );
}

#[test]
fn mobile_tablet_uses_a_continuous_two_row_strip() {
    let (mut scripts, mut snapshot, _) = mobile_reload_fixture();
    for id in 3..=7 {
        snapshot.windows.push(window(id, "DP-1", WIDE));
        scripts.opened(id, snapshot.clone());
    }
    let overview = scripts.key("super+space", snapshot.clone());
    let cards: Vec<_> = overview
        .commands
        .iter()
        .filter_map(|c| match c {
            Command::Present {
                id,
                rect: Some(rect),
                ..
            } => Some((*id, *rect)),
            _ => None,
        })
        .collect();
    let visible: Vec<_> = cards
        .iter()
        .filter(|(_, r)| r.x >= 0.0 && r.x + r.w <= WIDE.w)
        .collect();
    assert_eq!(visible.len(), 6);
    for (i, (_, a)) in visible.iter().enumerate() {
        for (_, b) in visible.iter().skip(i + 1) {
            assert!(
                a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y,
                "tablet cards must occupy separate grid cells"
            );
        }
    }
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":-100,"dy":0}"#,
    );
    mobile_action(&mut scripts, &snapshot, "swipe.end", "{}");
    assert_eq!(
        scripts.evaluate("return tostring(sol.keep('mobile.v1', {}).overview_offset == -100)"),
        "true",
        "a small scroll stays between columns instead of snapping to a page"
    );
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":-1600,"dy":0}"#,
    );
    let next = mobile_action(&mut scripts, &snapshot, "swipe.end", "{}");
    assert!(next.commands.iter().any(|c| matches!(c,
        Command::Present { id: 1, rect: Some(rect), .. } if rect.x >= 0.0 && rect.x + rect.w < WIDE.w
    )), "the seventh task is reachable by scrolling the strip");
}

#[test]
fn mobile_phone_keeps_the_overlapping_carousel_and_fixed_depth() {
    let (mut scripts, mut snapshot, _) = mobile_reload_fixture();
    let phone = Rect {
        x: 0.0,
        y: 0.0,
        w: 390.0,
        h: 844.0,
    };
    snapshot.monitors = vec![screen("DP-1", phone)];
    snapshot.work_area = phone;
    scripts.monitors_changed(snapshot.clone());
    let overview = scripts.key("super+space", snapshot.clone());
    let a = overview
        .commands
        .iter()
        .find_map(|c| match c {
            Command::Present {
                id: 2,
                rect: Some(rect),
                z,
                ..
            } => Some((*rect, *z)),
            _ => None,
        })
        .expect("first phone card");
    let b = overview
        .commands
        .iter()
        .find_map(|c| match c {
            Command::Present {
                id: 1,
                rect: Some(rect),
                z,
                ..
            } => Some((*rect, *z)),
            _ => None,
        })
        .expect("right phone card");
    assert!(b.0.x < a.0.x + a.0.w && b.1 > a.1);
    mobile_action(
        &mut scripts,
        &snapshot,
        "swipe.move",
        r#"{"dx":-210,"dy":0}"#,
    );
    let snapped = mobile_action(&mut scripts, &snapshot, "swipe.end", "{}");
    assert_eq!(
        scripts.evaluate("return tostring(sol.keep('mobile.v1', {}).active)"),
        "1"
    );
    assert!(snapped.commands.iter().any(|c| matches!(c,
        Command::Present { id: 1, z, .. } if *z == b.1
    )));
}
