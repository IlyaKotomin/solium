// effects-check's `motion` wallpaper (Task 54): black and white columns whose
// widths grow by 4 px from 8 px at the left edge, over rows whose colour
// steps every 90 px, so no two places on the monitor have the same backdrop
// and a blur read from the wrong place differs from the right one.
import QtQuick

Item {
    id: root

    Column {
        Repeater {
            model: Math.ceil(root.height / 90)
            Rectangle {
                width: root.width
                height: 90
                color: Qt.rgba(0.15 * (index % 6), 0.1 * (index % 4), 0.5, 1)
            }
        }
    }
    Row {
        Repeater {
            model: 28
            Rectangle {
                width: 8 + 4 * index
                height: root.height
                color: index % 2 ? "black" : "white"
                opacity: 0.7
            }
        }
    }
}
