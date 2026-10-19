import QtQuick
import Solium

// No titlebar reservation. Both the client mask and this header belong to
// the pane, and go through exactly the same presentation matrix as its pixels.
PaneStyle {
    requires: []
    client.radius: 30
    Layer { name: "shadow"; depth: "behind"; bleed: 24; source: "Shadow.qml" }
    Layer { name: "identity"; depth: "above"; bleed: { "top": 160 }; source: "Identity.qml" }
}
