import QtQuick
import QtQuick.Window

Window {
    id: app
    visible: true; width: 390; height: 748
    title: "Gallery"
    color: "#ffffff"
    property int selected: -1
    readonly property var tones: ["#efb855", "#e8889f", "#72b8d2", "#9994d0", "#eab998", "#91b8a5"]
    Text { x: 24; y: 24; text: "Gallery"; color: "#1c1c1e"; font { pixelSize: 26; weight: Font.Bold } }
    Text { x: 24; y: 64; text: "Your collection"; color: "#8e8e93"; font.pixelSize: 12 }
    Flickable {
        id: collection
        x: 20; y: 110; width: parent.width - 40; height: parent.height - 130
        contentWidth: width; contentHeight: studies.height; clip: true
        Grid {
        id: studies
        width: collection.width
        columns: app.width >= 700 ? 3 : 2; spacing: 12
        Repeater {
            model: 6
            Rectangle {
                required property int index
                width: (parent.width - (parent.columns - 1) * 12) / parent.columns
                height: width * 0.95; radius: 18; clip: true
                color: app.tones[index]
                Rectangle { width: parent.width * 0.45; height: width; radius: width / 2; x: parent.width * 0.5; y: 18; color: "#e7dbba"; opacity: 0.75 }
                Rectangle { width: parent.width * 1.6; height: parent.height; radius: width / 2; x: -parent.width * 0.2; y: parent.height * 0.45; rotation: -24; color: index % 2 ? "#4d625d" : "#405d51" }
                Text { x: 14; anchors { bottom: parent.bottom; bottomMargin: 12 } text: "STUDY 0" + (index + 1); color: "#e6ecdf"; font { pixelSize: 9; letterSpacing: 2 } }
                MouseArea { anchors.fill: parent; onClicked: app.selected = index }
            }
        }
        }
    }
    Rectangle {
        visible: app.selected >= 0
        anchors.fill: parent; color: app.tones[Math.max(0, app.selected)]
        Rectangle { width: parent.width * 0.5; height: width; radius: width / 2; x: parent.width * 0.4; y: parent.height * 0.15; color: "#e7dbba" }
        Rectangle { width: parent.width * 1.6; height: parent.height; radius: width / 2; x: -parent.width * 0.3; y: parent.height * 0.45; rotation: -24; color: "#405d51" }
        Text { x: 28; y: 28; text: "←  Collection"; color: "#243e32"; font.pixelSize: 18 }
        MouseArea { anchors.fill: parent; onClicked: app.selected = -1 }
    }
}
