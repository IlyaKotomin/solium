# Mobile shell prototype

An independent Lua/QML shell for a single phone or tablet display. It uses the
same Solium compositor and real Wayland clients as the desktop. The desktop
`init.lua`, default configuration, tiling and workspace scripts are not loaded.

![Phone launcher](mobile/phone-home.png)

## Behaviour

- **Phone:** one foreground task, full usable screen, with persistent status
  and three-button navigation bars. Home parks clients without terminating
  them; unbound keys cannot leak into those hidden clients.
- **Tablet:** the same launcher and task switcher, plus two real clients in
  split screen. Choose **Split** on a Recents card, then choose another card.
  **Resize split** cycles 50/50, 60/40 and 40/60; **Single app** returns to the
  selected task. Controls occupy the reserved navigation bar.
- **Recents:** MRU task cards use the compositor's actual client textures,
  transformed by Lua, with QML chrome and buttons. Previous/Next, Open and
  Close work with mouse presses; no touch gesture is required.
- **Apps:** the launcher includes Notes and Gallery demo clients and a real
  foot terminal. All apps also shows the compositor's live desktop-file app
  index. A matched relaunch restores its existing task.
- **Back:** closes the app drawer, shade or Recents, cancels split selection,
  or returns from the root app to Home while preserving its task. Linux
  desktop applications have no common Android activity-back protocol;
  in-app back navigation is not synthesized by this prototype.
- **Animations:** app scale/fade, Home fade, app-drawer reveal, live task
  scaling/sliding, split resize, and close fade use the compositor clock and
  hosted QML animations. A small dedicated loading scene avoids displaying
  the QML runner's technical name during launch.

Notes is an editable **in-memory draft**, not a persistent notes database.
Gallery is an interactive collection of generated geometric studies, not a
photo-library integration. The status bar shows a real clock; battery,
connectivity and notification data are not invented.

![Tablet with two independent clients in split screen](mobile/tablet-split.png)

## Start

After building Solium, from the checkout on a Linux Wayland desktop:

```sh
SOLIUM_LUA_INIT="$PWD/crates/solium/lua/mobile.lua" ./target/debug/solium
```

Use the ordinary nested backend first. `SOLIUM_MOBILE_PROFILE=auto` chooses
tablet at a shorter logical side of at least 600 pixels, phone otherwise.
`phone` and `tablet` force either profile. Changes of output size re-evaluate
the layout, including leaving split screen when switching to phone size.
The shell is intentionally scoped to the primary output; external-monitor
desktop behaviour is outside this prototype.

The demo launchers require foot and a Qt Quick `qml` runner on PATH.
`SOLIUM_QML_RUNNER` overrides that runner's executable. Rendering defaults to
the ordinary automatic GPU probe; `SOLIUM_MOBILE_SOFTWARE=1` selects software
QML for headless development.

The supplied mobile entry disables systemd session integration because this
prototype is tested nested. Review session/lock policy before deploying it
as a hardware login session.

## Fedora on macOS

The existing `solium-dev:fc44` image contains the required runtime. The
`solium-mobile-check` container in this workspace uses the original Rust 1.98
toolchain and the `solium-lua-lab-target` volume at `/target`.

```sh
docker exec solium-mobile-check bash dev/lua-lab/build.sh build

# Foreground, private headless sway host; no default Solium configuration.
docker exec -u nobody -e HOME=/tmp solium-mobile-check \
  bash /src/dev/mobile-session.sh phone

# Or tablet; SOLIUM_MOBILE_SIZE=800x1280 also exercises portrait.
docker exec -u nobody -e HOME=/tmp solium-mobile-check \
  bash /src/dev/mobile-session.sh tablet
```

For an interactive VNC session, from this checkout, after the build:

```sh
docker run -d --name solium-mobile-phone \
  -p 127.0.0.1:5902:5900 --user nobody \
  --mount "type=bind,source=$PWD,target=/src,readonly" \
  -v solium-lua-lab-target:/target:ro \
  -e HOME=/tmp -e SOLIUM_VNC=1 -e TZ=Europe/Sofia \
  solium-dev:fc44 bash /src/dev/mobile-session.sh phone
```

Connect macOS Screen Sharing to `vnc://127.0.0.1:5902`. The port is bound only
to localhost. For a second session use container `solium-mobile-tablet`, port
5903, and the `tablet` argument. Stop them with `docker stop` followed by the
container name. Logs are under `/tmp/solium-mobile-phone` or
`/tmp/solium-mobile-tablet` inside the respective container.

## Controls

| Control | Action |
|---|---|
| Navigation triangle / circle / square | Back / Home / Recents |
| Click status bar | Open or close the session panel |
| Super+H / Super+B / Super+Space / Super+A | Home / Back / Recents / All apps |
| Super+N / Super+G / Super+Return | Launch Notes / Gallery / Terminal |
| Super+[ / Super+] | Previous / Next task card |
| Super+Shift+S | Split the two most recent tasks, tablet only |
| Super+R / Super+U | Resize split / Single app |
| Super+Shift+R / Super+Shift+Q | Reload / Quit |

## Verification

Two Lua scenarios under `crates/solium/tests/scenarios/` run against real
Wayland clients: phone foreground geometry, Home parking, app drawer/Back,
Recents/selection and phone split rejection; tablet split bounds, ratio changes
and return to single-app mode. The existing runner discovers them dynamically.
Two additional script-host regressions reload the actual mobile entry with
kept state and verify MRU/Recents and tablet split/active-task preservation.
Both failed before guarding the entry's initial empty snapshot: a fresh Lua
VM has not received live windows yet, so treating that as an empty desktop
discarded the saved task order and split.

Live button scenarios use actual pointer press/release through `synth::drag`
with equal start/end coordinates. `SOLIUM_CLICK_AT` is deliberately not used:
it calls the Lua click handler and does not exercise hosted QML input.

```sh
bash dev/mobile-smoke.sh phone
bash dev/mobile-smoke.sh tablet
docker exec -u nobody -e HOME=/tmp solium-mobile-check \
  bash /src/dev/mobile-rotation.sh

# The lab helper supplies the existing Fedora Qt/EGL link-order workaround.
docker exec -e SOLIUM_TEST_FILTER=scenario::tests::every_scenario_with_a_client_passes \
  solium-mobile-check bash dev/lua-lab/build.sh test --exact
```

The live rotation run changes the host output through 390x844, 844x390,
1280x800, 800x1280 and back to 390x844. The compositor's own framebuffer is
captured, including intermediate animation frames. Notes input survives Home,
a key typed while Home is visible does not reach hidden Notes, and closing
Notes from Recents leaves the still-running Gallery task.

One compositor defect was fixed: `outputs_resized` previously updated native
output geometry and hosted canvases but never notified Lua. A phone-sized
window could retain the initial tablet policy. It now dispatches `monitors`
then `layout` against the resized bounds before syncing surfaces, like the
hotplug path. `resizing_an_output_notifies_scripts_before_relayout` fails before
the fix and passes after it.

Results on 2026-10-19:

| Check | Result |
|---|---|
| Complete Solium test binary, Rust 1.98 | 2042 passed, 0 failed, 3 pre-existing ignored |
| Workspace clippy, Rust 1.99, `--all-targets -- -D warnings` | Passed |
| Rust formatting, shell syntax, mobile `solium --check` | Passed |
| Phone buttons / typing / restore / close | Passed with real Qt Wayland clients |
| Tablet card picker / split / ratio | Passed with real Qt Wayland clients |
| Four size/orientation changes and automatic profile selection | Passed |
| Live reload during tablet split and Recents | Passed; task order and split preserved |
| Launch and Recents animation intermediate/settled frames | Captured and visually checked |

This verifies a mobile-sized **nested compositor**, with llvmpipe and software
QML. It does not verify a phone's DRM/GPU driver, suspend/power behaviour,
hardware sensors, physical screen rotation, or hardware performance. Touch,
gesture navigation, an on-screen keyboard, a lock screen and Android APK
compatibility are outside this iteration. XWayland was unavailable in this
unprivileged container; the tested clients are native Wayland.

## Source map

- `lua/mobile.lua`: independent entry and host settings.
- `lua/mobile/config.lua`: dimensions, motion and profile selection.
- `lua/mobile/policy.lua`: task ownership, MRU, navigation, real geometry,
  presentation and event handling; state survives Lua reload via `sol.keep`.
- `qml/mobile/`: hosted shell, controls and native QML demo clients.
- `qml/loading/mobile.qml`: loading presentation.
- `dev/mobile-session.sh`, `mobile-smoke.sh`, `mobile-rotation.sh`: isolated
  session and reproducible runtime exercises.
