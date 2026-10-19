import QtQuick

Item {
    id: shadow
    property var values: ({})
    property bool dormant: values.mobilePage !== "recents"
    property int bleedLeft: 0
    property int bleedTop: 0
    property int paneWidth: 0
    property int paneHeight: 0
    Repeater {
        model: 6
        Rectangle {
            required property int index
            readonly property real spread: index * 3
            x: shadow.bleedLeft - spread
            y: shadow.bleedTop - spread + 5
            width: shadow.paneWidth + spread * 2
            height: shadow.paneHeight + spread * 2
            radius: 30 + spread
            color: "#000000"; opacity: 0.035
        }
    }
}
