// A 64 px black and white checker filling its surface: a wallpaper whose
// blur is measured by its gradient energy (judge.py sharpness). With
// `animated` set, as `SOLIUM_CHECKER_PHASE=1` sets it in an FX3 section's
// .lua, it moves 8 px a second to the left, for an animated wallpaper; the
// loop is two cells long, so it starts again where it began.
import QtQuick

Item {
    id: root

    property bool animated: false
    property real phase: 0
    readonly property int columns: Math.ceil(width / 64) + 2

    NumberAnimation on phase {
        running: root.animated
        from: 0
        to: 128
        duration: 16000
        loops: Animation.Infinite
    }

    Grid {
        x: -root.phase
        columns: root.columns
        Repeater {
            model: root.columns * Math.ceil(root.height / 64)
            Rectangle {
                width: 64
                height: 64
                color: (index % root.columns + Math.floor(index / root.columns)) % 2 ? "black" : "white"
            }
        }
    }
}
