import QtQuick
import Solium

Item {
    id: frame
    property string title: ""
    property string appId: ""
    property int bleedTop: 0
    property var values: ({})
    property bool dormant: values.mobilePage !== "recents"
    property bool overlay: true
    readonly property real headerScale: values.mobileHeaderScale || 1
    readonly property string appName: title || appId || "App"
    Row {
        x: 12 * frame.headerScale; y: frame.bleedTop - 60
        scale: frame.headerScale; transformOrigin: Item.BottomLeft
        spacing: 10
        Rectangle {
            width: 34; height: 34; radius: 9
            color: frame.title === "Notes" ? "#ffcd35" : (frame.title === "Gallery" ? "#ff537a" : "#303642")
            Text {
                anchors.centerIn: parent
                text: frame.title === "Notes" ? "≡" : (frame.title === "Gallery" ? "✿" : frame.appName.slice(0, 1))
                color: "white"; font { pixelSize: 24; weight: Font.DemiBold }
                visible: appIcon.status !== Image.Ready
            }
            Image {
                id: appIcon
                anchors { fill: parent; margins: 3 }
                source: frame.title !== "Notes" && frame.title !== "Gallery" && frame.appId ? "image://solium/icon/" + frame.appId + "?size=32" : ""
                visible: status === Image.Ready
            }
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(0, frame.width / frame.headerScale - 90)
            text: frame.appName; elide: Text.ElideRight; color: "white"
            font { pixelSize: 22; weight: Font.DemiBold }
        }
    }
}
