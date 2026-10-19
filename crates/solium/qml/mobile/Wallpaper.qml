import QtQuick

Rectangle {
    color: "#141e28"
    gradient: Gradient {
        GradientStop { position: 0; color: "#253a43" }
        GradientStop { position: 0.55; color: "#1c3039" }
        GradientStop { position: 1; color: "#101923" }
    }
    clip: true
    Rectangle {
        width: parent.width * 1.6; height: width; radius: width / 2
        x: -width * 0.55; y: parent.height * 0.3
        color: "#31514f"; rotation: -25; opacity: 0.75
    }
    Rectangle {
        width: parent.width * 1.25; height: width * 1.7; radius: width / 2
        x: parent.width * 0.48; y: -height * 0.4
        color: "#556855"; rotation: -28; opacity: 0.55
    }
    Rectangle {
        width: parent.width * 1.3; height: width * 0.65; radius: width / 2
        x: -width * 0.28; y: parent.height * 0.85
        color: "#62765b"; rotation: -28; opacity: 0.32
    }
}
