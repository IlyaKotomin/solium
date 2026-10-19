#!/usr/bin/env bash
# Run inside the prepared Fedora container as nobody (see mobile README).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
profile="${1:-phone}"
case "$profile" in
    phone) size="${SOLIUM_MOBILE_SIZE:-390x844}" ;;
    tablet) size="${SOLIUM_MOBILE_SIZE:-1280x800}" ;;
    *) echo "usage: bash dev/mobile-session.sh [phone|tablet]" >&2; exit 2 ;;
esac
[[ "$size" =~ ^[0-9]+x[0-9]+$ ]] || { echo "Expected WIDTHxHEIGHT" >&2; exit 2; }
export SOLIUM_LAB_OUT="${SOLIUM_LAB_OUT:-/tmp/solium-mobile-$profile}"
mkdir -p "$SOLIUM_LAB_OUT"
export SOLIUM_HOST_CONFIG="$SOLIUM_LAB_OUT/sway.conf"
cat >"$SOLIUM_HOST_CONFIG" <<EOF
output HEADLESS-1 mode $size
default_border none
xwayland disable
for_window [app_id="solium"] fullscreen enable
EOF
export SOLIUM_LUA_INIT="$root/crates/solium/lua/mobile.lua"
export SOLIUM_FORM_FACTOR="$profile"
# Auto also exercises size adaptation on a host output resize.
export SOLIUM_MOBILE_PROFILE="${SOLIUM_MOBILE_PROFILE:-auto}"
export SOLIUM_MOBILE_SOFTWARE=1
export SOLIUM_QML_RUNNER="${SOLIUM_QML_RUNNER:-/usr/lib64/qt6/bin/qml}"
export QT_QPA_PLATFORM=wayland QT_QUICK_BACKEND=software
export SOLIUM_OUTPUTS=1 SOLIUM_LAB_CLIENT=none
exec bash "$root/dev/lua-lab/session.sh"
