// Liquid glass: the window and its bar as one rounded piece of dark glass,
// the bar along the bottom.
//
//     SOLIUM_PANE=glass
//
// Four files, one job each:
//
//   Pane.qml     what the frame reserves, the corners, and the layers;
//   Frame.qml    the bar: the window buttons at the left, the title after
//                them, the application's icon at the right, and no fill;
//   Shade.qml    a thin dark edge just inside the window's rounded outline,
//                round all four sides and the corners, while the window does
//                not have the keyboard, fading in and out with it;
//   effects.lua  one rule: the shipped `glass` effect behind the whole pane.
//
// **One shape, three things drawing it.** The compositor cuts the client's top
// two corners, which are the window's: `client.radius` below. The client's
// bottom two meet the bar, so they are square, and the bar's own bottom two,
// the window's, are rounded by `Frame.qml` to the same number, which every
// layer is told as `clientRadius`. And the glass behind the pane is cut to the
// whole window rounded at its largest corner, that same number again, so the
// glass, the client and the bar have one outline, with one rim round it and no
// seam where the bar meets the client. The shade follows that outline too.
//
// The numbers here and the ones at the top of `Frame.qml` and `Shade.qml` are
// the style's to change: copy this folder to ~/.config/solium/qml/panes/glass/,
// change them there, and your copy is used in place of this one.

import QtQuick
import Solium

PaneStyle {
    // The bar's height, in logical pixels: the band reserved under the
    // client. The compositor hands it to the layer as `insetBottom`.
    insets.bottom: Theme.titlebarHeight

    // Ordinary QML on the software path: no shader, no Canvas. The glass is
    // an effect behind the pane, the compositor's to draw.
    requires: []

    // The window's corners, in logical pixels: the client's top two, cut as
    // it is drawn, at no extra cost. Its bottom two are square, where the bar
    // meets it; the glass behind the bar is rounded to this number, and so is
    // the shade.
    client.radius: 16
    client.radiusBottomLeft: 0
    client.radiusBottomRight: 0

    // At `frame`, inside the band it was given: nothing it draws covers the
    // client. `bar` is the name a `region:titlebar` rule's slots go around.
    Layer {
        depth: "frame"
        name: "bar"
        source: "Frame.qml"
    }

    // At `above`, over the client and the bar, so the shade darkens the
    // window's own edge whatever the client draws; it takes no input, and is
    // dormant while the window has the keyboard. Before the keyboard pill, so
    // the pill is drawn over it.
    Layer {
        depth: "above"
        name: "shade"
        source: "Shade.qml"
    }

    // The keyboard pill at the focused text field's caret, when the
    // configuration says so (`keyboard.indicator.show = "pane"`). Delete the
    // line and this style draws none. See `Solium/KeyboardPillLayer.qml`.
    KeyboardPillLayer {}
}
