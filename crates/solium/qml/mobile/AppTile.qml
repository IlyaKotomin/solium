import QtQuick

Item {
    id: tile
    property string title: ""
    property string glyph: ""
    property string imageSource: ""
    property color tone: "#007aff"
    property bool showTitle: true
    signal triggered(var origin)
    width: 80; height: 92
    Accessible.role: Accessible.Button
    Accessible.name: title
    Rectangle {
        id: icon
        width: 60; height: 60; radius: 15
        anchors { horizontalCenter: parent.horizontalCenter; top: parent.top }
        gradient: Gradient {
            GradientStop { position: 0; color: Qt.lighter(tile.tone, 1.15) }
            GradientStop { position: 1; color: tile.tone }
        }
        scale: mouse.pressed ? 0.92 : 1
        Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
        Text {
            visible: image.status !== Image.Ready
            anchors.centerIn: parent; text: tile.glyph; color: "white"
            font { pixelSize: 30; weight: Font.Medium }
        }
        Image { id: image; anchors { fill: parent; margins: 6 } source: tile.imageSource; visible: status === Image.Ready }
    }
    Text {
        visible: tile.showTitle
        anchors { top: icon.bottom; topMargin: 7; horizontalCenter: parent.horizontalCenter }
        width: parent.width; horizontalAlignment: Text.AlignHCenter
        text: tile.title; color: "white"; font.pixelSize: 12; elide: Text.ElideRight
        style: Text.Raised; styleColor: "#40000000"
    }
    MouseArea {
        id: mouse
        anchors.fill: parent
        onClicked: {
            const p = icon.mapToItem(null, 0, 0)
            tile.triggered({ x: p.x, y: p.y, size: icon.width })
        }
    }
}
