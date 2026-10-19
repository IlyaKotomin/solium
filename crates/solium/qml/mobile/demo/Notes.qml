import QtQuick
import QtQuick.Window

Window {
    id: app
    visible: true; width: 390; height: 748
    title: "Notes"
    color: "#f1f0e5"
    Rectangle { x: 24; y: 24; width: 42; height: 42; radius: 14; color: "#d9e6bf"
        Text { anchors.centerIn: parent; text: "Aa"; color: "#33472b"; font.pixelSize: 20 }
    }
    Text { x: 80; y: 25; text: "Notes"; color: "#293d30"; font { pixelSize: 20; weight: Font.Medium } }
    Text { x: 80; y: 51; text: "Draft · kept while this app is open"; color: "#78816e"; font.pixelSize: 10 }
    Text { x: 24; y: 102; text: "Small things,\nbig possibilities."; color: "#293d30"; font { pixelSize: app.width < 500 ? 32 : 42; weight: Font.Light } }
    Rectangle { x: 24; y: 205; width: parent.width - 48; height: 1; color: "#d4d9c8" }
    Flickable {
        anchors { top: parent.top; topMargin: 230; bottom: parent.bottom; bottomMargin: 24; left: parent.left; right: parent.right; margins: 24 }
        contentHeight: editor.height; clip: true
        TextEdit {
            id: editor
            width: parent.width; height: Math.max(implicitHeight, 300)
            text: "A slower morning.\nA new idea.\nA little space to think.\n\nWrite something here…"
            color: "#43533f"; selectionColor: "#c8ddb3"
            font { pixelSize: 18; family: "sans-serif" }
            wrapMode: TextEdit.Wrap; focus: true
            selectByMouse: true; textFormat: TextEdit.PlainText
        }
    }
}
