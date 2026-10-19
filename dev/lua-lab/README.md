# Lua layout lab

Research based on upstream `stage` at `32a9f18`. The design discussion is in
[`docs/spikes/2026-10-19-lua-layout-policy.md`](../../docs/spikes/2026-10-19-lua-layout-policy.md).

`init.lua` and `config.lua` are independent of the default entry point and
default settings. No user configuration is merged. They load only the layout
modules being studied: three workspaces per monitor, no shell or wallpaper,
no decorations, no pointer warping and zero-duration layout animations.
Startup selects tiling. Super+T toggles tiling, Super+S scrolling, Super+1..3
switches workspaces and Super+Shift+1..3 sends the focused window. Super+Return
opens foot; Super+Shift+Q exits.

## Fedora environment

The existing `solium-dev:fc44` image has the build and nested-session tools.
Use a fresh target volume, preserving the old checkout and compiled binaries:

```sh
# From this checkout; create once. --mount source must be an absolute path.
docker run -d --name solium-lua-lab \
  --mount "type=bind,source=$PWD,target=/src" \
  -v solium-cargo:/cargo -v solium-lua-lab-target:/target \
  -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/target \
  -e CARGO_BUILD_JOBS=3 -w /src solium-dev:fc44 sleep infinity

docker exec solium-lua-lab bash dev/lua-lab/build.sh test
```

The smoke test loads this actual `init.lua`. The two ignored probes express
desired ownership invariants and are expected to fail on this baseline:

```sh
docker exec solium-lua-lab bash dev/lua-lab/build.sh test \
  --ignored --test-threads=1 --nocapture
```

They run the real Lua host, shipped mode modules and Rust layout trees against
snapshots, using the existing `script::directions::Desk` harness. They do not
drive a Wayland pointer or prove a rendered result. The cross-monitor probe
supplies the post-move snapshot at the same boundary used by `trigger_drop`.

The helper repeats Qt and EGL libraries after the QML host archive, working
around the baseline's link order with Fedora's `ld --as-needed`. It changes
neither `build.rs` nor dependency build flags. A direct `cargo test` in this
image fails at linking unless the same workaround is supplied.

## A separate nested session

```sh
docker exec solium-lua-lab bash dev/lua-lab/build.sh build
docker exec -u nobody -e HOME=/tmp solium-lua-lab \
  bash /src/dev/lua-lab/session.sh
```

The script starts its own headless sway and explicitly selects it as the host.
It selects the lab init with `SOLIUM_LUA_INIT`; the old `solium-mac` launcher
and its default configuration are not involved. Logs and the chosen client
socket are in `/tmp/solium-lua-lab/` inside the container. Stop the foreground
command to clean up the processes it started.

For two virtual Solium monitors, pass `-e SOLIUM_OUTPUTS=2` to `docker exec`.
For framebuffer readback, also pass `-e SOLIUM_CAPTURE=/tmp/lab-frame.ppm`
and `-e SOLIUM_CAPTURE_AT=5000`. This environment has software rendering;
it cannot establish hardware input, DRM behaviour or performance.

## Observed on 2026-10-19

Fedora 44, Cargo 1.98.0, Qt 6.11.2, upstream `32a9f18`:

| Check | Result |
|---|---|
| Independent minimal `init.lua` loads | Passed; current mode `tiling`, wallpaper false, preview init absent |
| `SOLIUM_LUA_INIT=/src/dev/lua-lab/init.lua solium --check` | Passed; 60 bindings, zero declared scenes, zero effect rules |
| Nested session, `SOLIUM_OUTPUTS=2` | Started and stopped via the lab binding; compositor framebuffer readback shows two tiled foot terminals on the first virtual monitor, the second empty |
| Workspace send releases source tree | Failed at the intended assertion: `contains(2)` is `true`, expected `false` |
| Cross-monitor drop carries membership | Destination tree accepted the pane; failed at the intended assertion: workspace `1`, expected `2` |
| Rust formatting, shell syntax, whitespace | Passed on the macOS host (the image has no cargo-fmt) |

Both failing probes are explicitly ignored in a normal test run and selected
with `--ignored` above. They record unresolved baseline bugs and must become
ordinary passing regressions when the transfer path is implemented. The full
workspace gate has not been run for this research branch.

The live smoke run used explicit `/bin/bash --noprofile --norc` in foot:
Fedora's nobody account has a nologin shell, so an unqualified foot initially
opened and immediately exited. The repeated run with bash produced the visible
terminals. The multi-output run also logged `eglQuerySurface / BAD_SURFACE`
messages, and XWayland could not start without its X11 socket directory.
Framebuffer readback succeeded, but renderer health and X11 support are not
claimed as passing checks here. The ownership probes do not depend on either.
