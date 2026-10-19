import QtQuick
import QtQuick.Window

Window {
    id: app
    visible: true; width: 390; height: 748
    title: "Notes"
    color: "#ffffff"
    Rectangle { x: 24; y: 24; width: 42; height: 42; radius: 14; color: "#ffcc31"
        Text { anchors.centerIn: parent; text: "Aa"; color: "#ffffff"; font.pixelSize: 20 }
    }
    Text { x: 80; y: 25; text: "Notes"; color: "#1c1c1e"; font { pixelSize: 20; weight: Font.Medium } }
    Text { x: 80; y: 51; text: "Personal draft"; color: "#8e8e93"; font.pixelSize: 10 }
    Text { x: 24; y: 102; text: "Notes"; color: "#1c1c1e"; font { pixelSize: app.width < 500 ? 32 : 42; weight: Font.Bold } }
    Rectangle { x: 24; y: 205; width: parent.width - 48; height: 1; color: "#e5e5ea" }
    Flickable {
        anchors { top: parent.top; topMargin: 230; bottom: parent.bottom; bottomMargin: 24; left: parent.left; right: parent.right; margins: 24 }
        contentHeight: editor.height; clip: true
        TextEdit {
            id: editor
            width: parent.width; height: Math.max(implicitHeight, 300)
            text: "A new idea.\n\nWrite something here…"
            color: "#1c1c1e"; selectionColor: "#b8d9ff"
            font { pixelSize: 18; family: "sans-serif" }
            wrapMode: TextEdit.Wrap; focus: true
            selectByMouse: true; textFormat: TextEdit.PlainText
        }
    }
}
