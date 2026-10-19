#!/usr/bin/env bash
# Host-side, repeatable button scenarios using the real pointer path.
set -euo pipefail
profile="${1:-phone}"
container="${SOLIUM_CONTAINER:-solium-mobile-check}"
out="/tmp/mobile-smoke-$profile"
case "$profile" in
    phone)
        pointer='1500:56,688>56,688;4500:195,812>195,812;5500:148,688>148,688;8500:291,812>291,812;10500:270,744>270,744;11500:195,400>195,400;13500:99,812>99,812;14500:343,688>343,688;16500:99,812>99,812;17500:291,812>291,812;19000:230,675>230,675;21000:195,812>195,812'
        frames=21; stop=22000
        ;;
    tablet)
        pointer='1500:484,620>484,620;4000:640,768>640,768;5000:588,620>588,620;7500:760,768>760,768;9000:630,550>630,550;10500:1100,400>1100,400;12000:70,768>70,768'
        frames=14; stop=15500
        ;;
    *) echo "usage: bash dev/mobile-smoke.sh [phone|tablet]" >&2; exit 2 ;;
esac
docker exec -u nobody -e HOME=/tmp \
    -e "SOLIUM_LAB_OUT=$out" -e "SOLIUM_CAPTURE=$out/frame.ppm" \
    -e SOLIUM_CAPTURE_AT=1000 -e "SOLIUM_CAPTURE_FRAMES=$frames" -e SOLIUM_CAPTURE_INTERVAL=1000 \
    -e "SOLIUM_DRAG_AT=$pointer" -e "SOLIUM_TRIGGER_AT=$stop:super+shift+q" \
    "$container" bash /src/dev/mobile-session.sh "$profile"
echo "Frames and session logs: $container:$out"
