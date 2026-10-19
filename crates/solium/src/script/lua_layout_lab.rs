// Included in script::directions so these probes reuse its snapshot/command
// harness and the real Lua host and Rust layout engines. They do not simulate
// Wayland grabs or render frames. The ignored tests express desired invariants
// that stage 32a9f18 violates; they are research probes, not claimed fixes.

fn lab_desk(monitors: Vec<MonitorInfo>) -> Desk {
    Desk::new(
        &format!(
            "package.loaded.config = dofile({:?})",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev/lua-lab/config.lua")
        ),
        monitors,
    )
}

#[test]
fn lua_layout_lab_minimal_init_loads() {
    let entry = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../dev/lua-lab/init.lua"
    ));
    let scripts = Scripts::load(entry).expect("loading the independent lab configuration");
    assert_eq!(
        scripts.evaluate("return require('modes').current()"),
        "tiling"
    );
    assert_eq!(
        scripts.evaluate("return tostring(require('config').wallpaper)"),
        "false"
    );
    assert_eq!(
        scripts.evaluate("return tostring(package.loaded['preview.init'])"),
        "nil"
    );
}

#[test]
#[ignore = "research: workspace send leaves the source tree occupied on stage 32a9f18"]
fn lua_layout_lab_send_reconciles_tree_membership() {
    let mut desk = lab_desk(one_screen());
    desk.press("super+t");
    desk.open(1, "DP-1", (600.0, 400.0));
    desk.open(2, "DP-1", (1800.0, 400.0));
    desk.press("super+shift+2");
    assert_eq!(desk.workspace_of(2), "2", "the membership changed");

    let outcome = desk.scripts.relayout(desk.snapshot());
    desk.apply(&outcome.commands);
    let source_contains = desk
        .scripts
        .evaluate("return tostring(require('tiling').trees['1@DP-1']:contains(2))");
    assert_eq!(
        source_contains, "false",
        "the source tree must release window 2"
    );
}

#[test]
#[ignore = "research: drop into a monitor showing another workspace retains the old membership"]
fn lua_layout_lab_cross_monitor_drop_carries_workspace() {
    let mut desk = lab_desk(two_screens());
    desk.press("super+t");
    desk.open(1, "DP-1", (600.0, 400.0));
    desk.open(2, "DP-1", (1800.0, 400.0));
    let snapshot = desk.snapshot();
    desk.scripts
        .evaluate_in(snapshot, "require('workspaces').go(2, 'DP-2'); return 'ok'");
    assert_eq!(desk.workspace_of(2), "1", "the source workspace");
    assert_eq!(
        desk.scripts
            .evaluate("return tostring(require('workspaces').on('DP-2'))"),
        "2"
    );

    // Supply the snapshot after a physical move, as trigger_drop does. This
    // deliberately tests the drop boundary, not the pointer-grab machinery.
    let moved = desk
        .windows
        .iter_mut()
        .find(|window| window.id == 2)
        .expect("the dragged window");
    moved.monitor = "DP-2".to_owned();
    moved.rect = Rect {
        x: 2700.0,
        y: 100.0,
        w: 600.0,
        h: 400.0,
    };
    desk.cursor = (3000.0, 300.0);
    let outcome = desk.scripts.dropped(2, 3000.0, 300.0, desk.snapshot());
    desk.apply(&outcome.commands);
    assert_eq!(
        desk.scripts
            .evaluate("return tostring(require('tiling').trees['2@DP-2']:contains(2))"),
        "true",
        "the destination arrangement accepted the window"
    );
    assert_eq!(
        desk.workspace_of(2),
        "2",
        "membership must agree with the destination tree"
    );
}
