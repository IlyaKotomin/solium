import QtQuick

Rectangle {
    property bool dark: false
    property bool overview: false
    color: dark ? "#10172e" : "#707fae"
    gradient: Gradient {
        GradientStop { position: 0; color: dark ? "#172042" : "#899cb7" }
        GradientStop { position: 0.48; color: dark ? "#252340" : "#656d9a" }
        GradientStop { position: 1; color: dark ? "#131c38" : "#333f70" }
    }
    clip: true
    Rectangle {
        width: parent.width * 1.9; height: parent.height * 1.05; radius: width / 2
        x: -width * 0.65; y: parent.height * 0.22; rotation: -28
        color: parent.dark ? "#26395c" : "#90b7c1"; opacity: 0.5
    }
    Rectangle {
        width: parent.width * 1.4; height: parent.height * 0.9; radius: width / 2
        x: parent.width * 0.62; y: -height * 0.3; rotation: -30
        color: parent.dark ? "#4b3261" : "#b99caf"; opacity: 0.55
    }
    Rectangle { anchors.fill: parent; color: "#171a30"; opacity: parent.overview ? 0.25 : 0
        Behavior on opacity { NumberAnimation { duration: 280 } }
    }
}
