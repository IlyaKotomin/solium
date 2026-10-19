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
        r#"{"x":1280,"y":700}"#,
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
        "1"
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
        r#"{"x":1280,"y":700}"#,
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
