-- The glass style's one rule: the shipped `glass` effect behind the whole
-- pane, frame and client together, cut to the window's outline, every corner
-- rounded at the window's (Pane.qml's `client.radius`). So the window is one
-- piece of glass, with one rim round its rounded outline and no seam where the
-- bar meets the client.
--
-- Not on a fullscreen window, which keeps direct scanout. A fullscreen window
-- has no frame, so no style's rule reaches it anyway; the match says so here
-- too, where the rule is read.
--
-- Your own `effects.rules` win over this one for the same part and slot:
-- `{ match = { style = "glass" }, part = "pane", slot = "behind", effect = false }`
-- takes the glass away, and `effect = { "glass", haze = 1 }` there gives it
-- other params.
return {
    { match = { fullscreen = false }, part = "pane", slot = "behind", effect = "glass" },
}
