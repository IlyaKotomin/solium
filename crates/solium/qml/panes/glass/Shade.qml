// The `shade` layer of `panes/glass`: a thin dark edge just inside the
// window's rounded outline, all the way round and through the corners, on a
// window that does not have the keyboard. It comes in as the window loses the
// keyboard and goes as it gets it back, on the compositor's clock.
//
// **The fade draws nothing.** It is `layerOpacity`, which the compositor
// applies as it composites this layer's picture: the rings are drawn once, at
// each size the window takes, and a fade blends that picture at another alpha
// each frame without asking Qt for a new one
// (`decoration::tests::the_glass_shade_fades_without_drawing_and_the_bar_draws_once`).
//
// **Above the client**, so it darkens the window's own edge: an opaque
// application's as well as the glass behind a clear one. A layer at `behind`
// would be covered by every opaque window. It holds no `MouseArea`, so the
// pointer reaches the client and the bar's buttons as it did before it. While
// the window has the keyboard and the shade has faded out it is `dormant`: not
// drawn, nothing blended over the client, and in software no image kept, so
// the window being typed in costs nothing for it.
//
// Stacked rounded rectangles, `shadeWidth` of them, each a one-pixel border a
// pixel further in than the last with its corner a pixel smaller, so every
// ring is concentric with the window's outline, the corners included, and each
// is fainter than the one outside it. The numbers under "The look" are the
// style's to change, as the panes README says.

import QtQuick

Item {
    id: shade

    // Set by the compositor: whether the window has the keyboard, and its
    // corner (`Pane.qml`'s `client.radius`, the largest of the four), which
    // the glass behind it and the bar's corners are rounded to as well.
    property bool focused: false
    property int clientRadius: 0

    // --- The look ----------------------------------------------------------
    // The shade's colour, how opaque it is at the window's very edge, and how
    // far in it reaches, in logical pixels; it fades to nothing there.
    property color shadeColor: "#000000"
    property real shadeStrength: 0.5
    property int shadeWidth: 12
    // How long it takes to come and go with the focus, in milliseconds
    // (one and a half times `Theme.normal`), and on which curve: any `Easing`
    // type of QtQuick's.
    property int shadeDuration: 240
    property int shadeEasing: Easing.OutCubic

    // How opaque the compositor draws the rings: the fade, which draws no new
    // picture. Read by the compositor.
    property real layerOpacity: shade.focused ? 0 : 1

    Behavior on layerOpacity {
        NumberAnimation { duration: shade.shadeDuration; easing.type: shade.shadeEasing }
    }

    // Nothing to draw: the window has the keyboard and the shade has gone.
    // Awake from the moment it loses the keyboard, before the fade moves.
    property bool dormant: shade.focused && shade.layerOpacity === 0

    Item {
        id: rings

        anchors.fill: parent

        Repeater {
            model: Math.max(shade.shadeWidth, 0)

            Rectangle {
                required property int index

                // Falling with the square of the way in, so the very edge is
                // dark and the inside soft.
                readonly property real fall: 1 - index / shade.shadeWidth

                x: index
                y: index
                width: rings.width - 2 * index
                height: rings.height - 2 * index
                radius: Math.max(shade.clientRadius - index, 0)
                antialiasing: true
                color: "transparent"
                border.width: 1
                border.color: Qt.rgba(shade.shadeColor.r, shade.shadeColor.g, shade.shadeColor.b,
                                      shade.shadeStrength * fall * fall)
            }
        }
    }
}
