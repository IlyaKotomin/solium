import QtQuick

Rectangle {
    property string program: ""
    property int waited: 0
    color: "#f1f0e5"
    Column {
        anchors.centerIn: parent; spacing: 20
        Row {
            anchors.horizontalCenter: parent.horizontalCenter; spacing: 8
            Repeater {
                model: 3
                Rectangle {
                    required property int index
                    width: 9; height: 9; radius: 4.5; color: "#57744e"
                    SequentialAnimation on opacity {
                        loops: Animation.Infinite
                        PauseAnimation { duration: index * 100 }
                        NumberAnimation { from: 0.3; to: 1; duration: 350 }
                        NumberAnimation { from: 1; to: 0.3; duration: 350 }
                    }
                }
            }
        }
        Text { text: "Opening…"; color: "#43533f"; font.pixelSize: 15 }
    }
}
