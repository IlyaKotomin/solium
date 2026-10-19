#!/usr/bin/env bash
# Container-side resize/rotation exercise. Each resize is a host output mode
# change: Solium must publish new bounds, choose its profile and reflow clients.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export SOLIUM_LAB_OUT="${SOLIUM_LAB_OUT:-/tmp/mobile-rotation}"
export SOLIUM_CAPTURE="$SOLIUM_LAB_OUT/frame.ppm"
export SOLIUM_CAPTURE_AT=1000 SOLIUM_CAPTURE_FRAMES=18 SOLIUM_CAPTURE_INTERVAL=1000
export SOLIUM_TRIGGER_AT="1000:super+n,5000:super+h,9000:super+g,10000:super+shift+s,14000:super+h,17000:super+space,19000:super+shift+q"
bash "$root/dev/mobile-session.sh" phone &
session=$!
trap 'kill "$session" 2>/dev/null || true' EXIT
socket=""
for ((i = 0; i < 100; i++)); do
    for path in "$SOLIUM_LAB_OUT/runtime"/sway-ipc.*.sock; do
        if [[ -S "$path" ]]; then socket="$path"; break; fi
    done
    [[ -n "$socket" ]] && break
    sleep 0.1
done
[[ -n "$socket" ]] || { echo "No private sway IPC socket" >&2; exit 1; }
sleep 3
swaymsg -s "$socket" output HEADLESS-1 mode 844x390
sleep 4
swaymsg -s "$socket" output HEADLESS-1 mode 1280x800
sleep 5
swaymsg -s "$socket" output HEADLESS-1 mode 800x1280
sleep 3
swaymsg -s "$socket" output HEADLESS-1 mode 390x844
wait "$session"
