#!/usr/bin/env bash
# Fedora lab helper. Repeat Qt/EGL after the QML host archive: the baseline
# build.rs emits them before it, and ld --as-needed discards them too early.
# cargo rustc confines the workaround to Solium, preserving dependency caches.
set -euo pipefail
kind="${1:-test}"
shift || true
case "$kind" in
    test) profile=test ;;
    build) profile=dev ;;
    *) echo "usage: bash dev/lua-lab/build.sh [test [test-options...]|build]" >&2; exit 2 ;;
esac

result="$(mktemp)"
trap 'rm -f "$result"' EXIT
if ! cargo rustc -p solium --bin solium --locked --profile "$profile" \
    --message-format=json -- \
    -C link-arg=-lQt6Quick -C link-arg=-lQt6OpenGL \
    -C link-arg=-lQt6Gui -C link-arg=-lQt6Qml \
    -C link-arg=-lQt6Network -C link-arg=-lQt6Core \
    -C link-arg=-lEGL >"$result"; then
    python3 - "$result" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    item = json.loads(line)
    if item.get("reason") == "compiler-message":
        print(item["message"].get("rendered", ""), file=sys.stderr)
PY
    exit 1
fi

binary="$(python3 - "$result" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    item = json.loads(line)
    if item.get("reason") == "compiler-artifact" and item.get("executable"):
        if item.get("target", {}).get("name") == "solium":
            print(item["executable"])
PY
)"
[[ -n "$binary" ]] || { echo "No Solium executable in Cargo output" >&2; exit 1; }
if [[ "$kind" == test ]]; then
    "$binary" lua_layout_lab "$@"
else
    echo "$binary"
fi
