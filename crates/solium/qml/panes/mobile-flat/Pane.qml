import QtQuick
import Solium

// The same pane-bound chrome as mobile, with no client corner mask while
// an application occupies its full tile. Insets remain identical in both.
PaneStyle {
    requires: []
    client.radius: 0
    Layer { name: "shadow"; depth: "behind"; bleed: 24; source: "../mobile/Shadow.qml" }
    Layer { name: "identity"; depth: "above"; bleed: { "top": 160 }; source: "../mobile/Identity.qml" }
}
