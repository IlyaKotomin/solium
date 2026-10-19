#!/usr/bin/env bash
# Run as an unprivileged user inside the Fedora container. The host Wayland
# session is never used. No default Solium entry point or macOS helper is run.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="${SOLIUM_LAB_OUT:-/tmp/solium-lua-lab}"
mkdir -p "$out"
export XDG_RUNTIME_DIR="$out/runtime"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
export LIBGL_ALWAYS_SOFTWARE=1
export LC_ALL=C.UTF-8
export WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER=pixman
export SOLIUM_LUA_INIT="$here/init.lua"
unset DISPLAY WAYLAND_DISPLAY WAYLAND_SOCKET

pids=()
cleanup() {
    for pid in "${pids[@]}"; do kill "$pid" 2>/dev/null || true; done
}
trap cleanup EXIT

sway -c "$here/sway.conf" >"$out/sway.log" 2>&1 &
host_pid=$!
pids+=("$host_pid")
host=""
for ((attempt = 0; attempt < 100; attempt++)); do
    kill -0 "$host_pid" 2>/dev/null || { cat "$out/sway.log"; exit 1; }
    for socket in "$XDG_RUNTIME_DIR"/wayland-*; do
        if [[ -S "$socket" ]]; then host="${socket##*/}"; break; fi
    done
    [[ -n "$host" ]] && break
    sleep 0.1
done
[[ -n "$host" ]] || { echo "No headless host socket" >&2; exit 1; }
export WAYLAND_DISPLAY="$host"

"${SOLIUM_BIN:-/target/debug/solium}" >"$out/solium.log" 2>&1 &
solium_pid=$!
pids+=("$solium_pid")
socket=""
for ((attempt = 0; attempt < 200; attempt++)); do
    kill -0 "$solium_pid" 2>/dev/null || { cat "$out/solium.log"; exit 1; }
    socket="$(sed -n 's/.*socket=\(wayland-[0-9]*\).*/\1/p' "$out/solium.log" | tail -1)"
    [[ -n "$socket" ]] && break
    sleep 0.1
done
[[ -n "$socket" ]] || { echo "No Solium socket" >&2; exit 1; }
echo "$socket" >"$out/socket"
echo "Solium: $socket, runtime: $XDG_RUNTIME_DIR, config: $SOLIUM_LUA_INIT"

# nobody's login shell is nologin in Fedora. Supply an interactive shell
# explicitly so the terminal does not map and immediately disappear.
WAYLAND_DISPLAY="$socket" foot /bin/bash --noprofile --norc >"$out/foot.log" 2>&1 &
pids+=("$!")
wait "$solium_pid"
