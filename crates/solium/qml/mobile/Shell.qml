import QtQuick
import Solium

Item {
    id: root
    property string page: "home"
    property bool tablet: false
    property var cards: []
    property int taskCount: 0
    property int activeId: 0
    property bool splitActive: false
    property bool pickingSplit: false
    property int statusHeight: 32
    property int navigationHeight: 64
    property date now: new Date()
    readonly property color ink: "#e6f0df"
    readonly property bool launcher: page === "home" || page === "apps"
    readonly property real inset: tablet ? 40 : 24
    function send(action, data) { Solium.send("mobile." + action, data || {}) }
    Timer { interval: 1000; running: true; repeat: true; onTriggered: root.now = new Date() }

    // A shell page owns the keyboard; client view relinquishes it. The Lua
    // input grab additionally protects thumbnail clients from stray input.
    focus: page !== "app"
    Solium.keyboard.wants: page !== "app"
    Solium.keyboard.claims: ["Escape"]
    Keys.onEscapePressed: root.send("back")

    Rectangle {
        anchors.fill: parent
        color: "#101c24"
        opacity: root.page === "apps" ? 0.97 : (root.page === "recents" ? 0.22 : 0)
        Behavior on opacity { NumberAnimation { duration: 260; easing.type: Easing.OutCubic } }
    }

    // Status is deliberately only a real clock and session identity. No fake
    // battery, radio signal or connectivity data is shown.
    Rectangle {
        id: status
        width: parent.width; height: root.statusHeight
        color: root.page === "app" ? "#101a20" : "transparent"
        Text {
            anchors { left: parent.left; leftMargin: root.inset; verticalCenter: parent.verticalCenter }
            text: Qt.formatTime(root.now, "h:mm"); color: root.ink; font { pixelSize: 12; weight: Font.DemiBold }
        }
        Text {
            anchors { right: parent.right; rightMargin: root.inset; verticalCenter: parent.verticalCenter }
            text: root.splitActive ? "SPLIT SCREEN" : "SOLIUM"; color: "#bfcebd"
            font { pixelSize: 10; letterSpacing: 2 }
        }
        MouseArea { anchors.fill: parent; onClicked: root.send("shade") }
    }

    Item {
        id: home
        anchors { left: parent.left; right: parent.right; top: status.bottom; bottom: navigation.top }
        opacity: root.page === "home" ? 1 : 0
        visible: opacity > 0.001
        enabled: root.page === "home"
        Behavior on opacity { NumberAnimation { duration: 240; easing.type: Easing.OutCubic } }
        MouseArea { anchors.fill: parent }
        Column {
            anchors { top: parent.top; topMargin: root.height < 560 ? 18 : (root.tablet ? 58 : 52); left: parent.left; leftMargin: root.inset }
            spacing: 8
            Text { text: Qt.formatDate(root.now, "dddd, MMMM d"); color: "#d4dfcf"; font.pixelSize: root.tablet ? 18 : 15 }
            Text { text: Qt.formatTime(root.now, "hh:mm"); color: "#e0edcf"; font { pixelSize: root.height < 560 ? 50 : (root.tablet ? 112 : 86); weight: Font.Light; letterSpacing: -3 } }
            Text { visible: root.height >= 560; text: "Make space for your day."; color: "#b9cdbb"; font.pixelSize: 14 }
        }
        Rectangle {
            visible: root.height >= 600
            width: Math.min(parent.width - root.inset * 2, 440); height: 80; radius: 24
            anchors { left: parent.left; leftMargin: root.inset; bottom: dock.top; bottomMargin: root.tablet ? 48 : 32 }
            color: "#a52b4140"; border { width: 1; color: "#456059" }
            Column {
                anchors { left: parent.left; leftMargin: 20; verticalCenter: parent.verticalCenter }
                spacing: 6
                Text { text: root.taskCount ? "Pick up where you left off" : "A fresh start"; color: root.ink; font { pixelSize: 15; weight: Font.Medium } }
                Text { text: root.taskCount ? root.taskCount + (root.taskCount === 1 ? " app running" : " apps running") : "Your apps are one tap away"; color: "#b9ccbb"; font.pixelSize: 12 }
            }
            Text { anchors { right: parent.right; rightMargin: 20; verticalCenter: parent.verticalCenter } text: "→"; color: "#c5dfb3"; font.pixelSize: 26 }
            MouseArea { anchors.fill: parent; onClicked: root.send(root.taskCount ? "recents" : "apps") }
        }
        Row {
            id: dock
            anchors { bottom: parent.bottom; bottomMargin: root.tablet ? 42 : 16; horizontalCenter: parent.horizontalCenter }
            spacing: root.tablet ? 24 : Math.max(0, (root.width - 32 - 320) / 3)
            AppTile { title: "Notes"; glyph: "Aa"; tone: "#c9dfb6"; onTriggered: root.send("launch", { app: "notes" }) }
            AppTile { title: "Gallery"; glyph: "▧"; tone: "#e1ceaf"; onTriggered: root.send("launch", { app: "gallery" }) }
            AppTile { title: "Terminal"; glyph: ">_"; tone: "#b7d5d7"; onTriggered: root.send("launch", { app: "terminal" }) }
            AppTile { title: "All apps"; gridIcon: true; tone: "#d1c7e4"; onTriggered: root.send("apps") }
        }
    }

    Item {
        anchors { top: status.bottom; bottom: navigation.top; left: parent.left; right: parent.right }
        opacity: root.page === "apps" ? 1 : 0
        visible: opacity > 0.001
        enabled: root.page === "apps"
        scale: 0.97 + opacity * 0.03
        Behavior on opacity { NumberAnimation { duration: 240; easing.type: Easing.OutCubic } }
        MouseArea { anchors.fill: parent }
        Text {
            id: appsTitle
            x: root.inset; y: 28; text: "Your apps"; color: root.ink; font { pixelSize: 30; weight: Font.Medium }
        }
        Text { x: root.inset; y: 72; text: "OPEN SOMETHING GOOD"; color: "#9cb5a4"; font { pixelSize: 10; letterSpacing: 2 } }
        Flickable {
            anchors { top: appsTitle.bottom; topMargin: 56; bottom: parent.bottom; left: parent.left; right: parent.right; margins: root.inset }
            clip: true; contentHeight: appGrid.height + 24
            Grid {
                id: appGrid
                columns: Math.max(3, Math.floor(parent.width / 94)); spacing: 14
                AppTile { title: "Notes"; glyph: "Aa"; onTriggered: root.send("launch", { app: "notes" }) }
                AppTile { title: "Gallery"; glyph: "▧"; tone: "#e1ceaf"; onTriggered: root.send("launch", { app: "gallery" }) }
                AppTile { title: "Terminal"; glyph: ">_"; tone: "#b7d5d7"; onTriggered: root.send("launch", { app: "terminal" }) }
                Repeater {
                    model: Apps
                    AppTile {
                        title: model.name
                        glyph: model.name.slice(0, 1).toUpperCase()
                        tone: "#d1c7e4"
                        onTriggered: root.send("desktop", { id: model.id })
                    }
                }
            }
        }
    }

    Item {
        id: recents
        anchors { fill: parent; topMargin: root.statusHeight; bottomMargin: root.navigationHeight }
        opacity: root.page === "recents" ? 1 : 0
        visible: opacity > 0.001
        enabled: root.page === "recents"
        Behavior on opacity { NumberAnimation { duration: 250; easing.type: Easing.OutCubic } }
        clip: true
        MouseArea { anchors.fill: parent }
        Text {
            x: root.inset; y: 22
            text: root.pickingSplit ? "Choose a second app" : "Recent apps"
            color: root.ink; font { pixelSize: root.tablet ? 30 : 26; weight: Font.Medium }
        }
        Text {
            x: root.inset; y: 62
            text: root.pickingSplit ? "Select another card to share the screen" : root.taskCount + (root.taskCount === 1 ? " app, ready when you are" : " apps, ready when you are")
            color: "#b9ccbb"; font.pixelSize: 12
        }
        Repeater {
            model: root.cards
            Item {
                id: card
                required property var modelData
                x: modelData.x - Solium.monitor.whole.x
                y: modelData.y - Solium.monitor.whole.y - root.statusHeight
                width: modelData.w; height: modelData.h
                // The pixels in this rectangle are the compositor's live pane.
                // Only chrome and hit regions belong to QML.
                Rectangle { anchors.fill: parent; color: "transparent"; radius: 16; border { width: 1; color: "#6b8173" } }
                Text {
                    anchors { left: parent.left; right: parent.right; bottom: parent.top; bottomMargin: 12 }
                    text: card.modelData.title; color: root.ink; elide: Text.ElideRight
                    font { pixelSize: 14; weight: Font.Medium }
                }
                MouseArea { anchors.fill: parent; onClicked: root.send("select", { id: card.modelData.id }) }
                Row {
                    anchors { top: parent.bottom; topMargin: 14; horizontalCenter: parent.horizontalCenter } spacing: 10
                    ActionButton { label: "Open"; highlighted: true; onTriggered: root.send("select", { id: card.modelData.id }) }
                    ActionButton { visible: root.tablet && root.taskCount > 1; label: "Split"; onTriggered: root.send("split", { id: card.modelData.id }) }
                    ActionButton { symbol: "×"; label: "Close"; onTriggered: root.send("close", { id: card.modelData.id }) }
                }
            }
        }
        Column {
            visible: root.taskCount === 0
            anchors.centerIn: parent; spacing: 12
            Text { anchors.horizontalCenter: parent.horizontalCenter; text: "A little breathing room"; color: root.ink; font.pixelSize: 21 }
            Text { anchors.horizontalCenter: parent.horizontalCenter; text: "Open an app and it will appear here."; color: "#b9ccbb"; font.pixelSize: 13 }
            ActionButton { anchors.horizontalCenter: parent.horizontalCenter; label: "All apps"; highlighted: true; onTriggered: root.send("apps") }
        }
        Row {
            visible: root.taskCount > 1
            anchors { bottom: parent.bottom; bottomMargin: 12; horizontalCenter: parent.horizontalCenter } spacing: 16
            ActionButton { symbol: "←"; label: "Previous"; onTriggered: root.send("previous") }
            ActionButton { label: "Next"; symbol: "→"; onTriggered: root.send("next") }
        }
    }

    Rectangle {
        anchors { top: status.bottom; left: parent.left; right: parent.right; bottom: navigation.top; margins: 12 }
        radius: 28; color: "#edf2e6"; visible: root.page === "shade"
        MouseArea { anchors.fill: parent }
        Column {
            x: 24; y: 30; width: parent.width - 48; spacing: 20
            Text { text: Qt.formatDate(root.now, "dddd, MMMM d"); color: "#344b3b"; font.pixelSize: 15 }
            Text { text: "Take a moment"; color: "#23392d"; font { pixelSize: 30; weight: Font.Medium } }
            Text { text: root.taskCount + " apps running"; color: "#536a59"; font.pixelSize: 15 }
            ActionButton { label: "Recent apps"; highlighted: true; onTriggered: root.send("recents") }
            ActionButton { label: "All apps"; onTriggered: root.send("apps") }
            Rectangle { width: parent.width; height: 1; color: "#c5d2be" }
            Text { width: parent.width; text: "A little space to focus."; color: "#536a59"; wrapMode: Text.WordWrap; font.pixelSize: 16 }
        }
    }

    Rectangle {
        id: navigation
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: root.navigationHeight
        color: root.page === "app" ? "#101a20" : "#d9142329"
        Row {
            visible: root.page === "app" && root.splitActive
            anchors { left: parent.left; leftMargin: 24; verticalCenter: parent.verticalCenter }
            spacing: 8
            ActionButton { label: root.width >= 800 ? "Resize split" : ""; symbol: root.width >= 800 ? "" : "↔"; Accessible.name: "Resize split"; onTriggered: root.send("ratio") }
            ActionButton { label: root.width >= 800 ? "Single app" : ""; symbol: root.width >= 800 ? "" : "▣"; Accessible.name: "Single app"; onTriggered: root.send("unsplit") }
        }
        Row {
            anchors.centerIn: parent
            spacing: root.tablet ? 64 : 40
            ActionButton { width: 56; height: 48; symbol: "◁"; fill: "transparent"; Accessible.name: "Back"; onTriggered: root.send("back") }
            ActionButton { width: 56; height: 48; symbol: "○"; fill: "transparent"; Accessible.name: "Home"; onTriggered: root.send("home") }
            ActionButton { width: 56; height: 48; symbol: "□"; fill: "transparent"; Accessible.name: "Recent apps"; onTriggered: root.send("recents") }
        }
    }
}
