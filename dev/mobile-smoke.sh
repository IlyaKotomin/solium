#!/usr/bin/env bash
# Host-side, repeatable swipe and button scenarios using the real pointer path.
set -euo pipefail
profile="${1:-phone}"
container="${SOLIUM_CONTAINER:-solium-mobile-check}"
out="/tmp/mobile-smoke-$profile"
case "$profile" in
    phone)
        pointer='1500:300,400>70,400;3000:70,400>300,400;4500:56,90>56,90;6500:195,826>195,826;7500:149,90>149,90;9500:279,826>279,826;11000:280,400>80,400;12500:195,450>195,240;14000:195,400>195,400;15500:350,16>350,180;17000:100,130>100,130;18000:120,410>120,410;20000:111,826>111,826;21500:195,826>195,826'
        frames=23; stop=24000
        ;;
    tablet)
        pointer='1500:91,125>91,125;3500:640,782>640,782;4500:201,125>201,125;6500:736,782>736,782;8000:850,400>450,400;9500:200,450>200,450;11000:1200,16>1200,180;12500:550,500>550,500;14000:200,450>200,450;15500:35,782>35,782'
        frames=17; stop=18500
        ;;
    *) echo "usage: bash dev/mobile-smoke.sh [phone|tablet]" >&2; exit 2 ;;
esac
docker exec -u nobody -e HOME=/tmp \
    -e "SOLIUM_LAB_OUT=$out" -e "SOLIUM_CAPTURE=$out/frame.ppm" \
    -e SOLIUM_CAPTURE_AT=1000 -e "SOLIUM_CAPTURE_FRAMES=$frames" -e SOLIUM_CAPTURE_INTERVAL=1000 \
    -e "SOLIUM_DRAG_AT=$pointer" -e "SOLIUM_TRIGGER_AT=$stop:super+shift+q" \
    "$container" bash /src/dev/mobile-session.sh "$profile"
echo "Frames and session logs: $container:$out"
