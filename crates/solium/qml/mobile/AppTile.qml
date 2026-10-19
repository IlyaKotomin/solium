import QtQuick

Item {
    id: tile
    property string title: ""
    property string glyph: ""
    property bool gridIcon: false
    property color tone: "#c5dfb3"
    signal triggered()
    width: 80; height: 104
    Accessible.role: Accessible.Button
    Accessible.name: title
    Rectangle {
        id: icon
        width: 58; height: 58; radius: 20
        anchors { horizontalCenter: parent.horizontalCenter; top: parent.top }
        color: tile.tone
        scale: mouse.pressed ? 0.9 : 1
        Behavior on scale { NumberAnimation { duration: 170; easing.type: Easing.OutCubic } }
        Text { visible: !tile.gridIcon; anchors.centerIn: parent; text: tile.glyph; color: "#20322b"; font { pixelSize: 26; weight: Font.Medium } }
        Grid {
            visible: tile.gridIcon; anchors.centerIn: parent; columns: 2; spacing: 5
            Repeater { model: 4; Rectangle { width: 8; height: 8; radius: 2; color: "#20322b" } }
        }
    }
    Text {
        anchors { top: icon.bottom; topMargin: 10; horizontalCenter: parent.horizontalCenter }
        width: parent.width; horizontalAlignment: Text.AlignHCenter
        text: tile.title; color: "#e6f0df"; font.pixelSize: 12; elide: Text.ElideRight
    }
    MouseArea { id: mouse; anchors.fill: parent; onClicked: tile.triggered() }
}
