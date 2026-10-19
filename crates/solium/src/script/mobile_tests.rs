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
