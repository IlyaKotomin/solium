-- effects-check's `xray` section, and the scene later FX3 sections start
-- from: a 64 px checker wallpaper (checker.qml, on `background`), a
-- translucent terminal at (80, 80) 560x360, the first window to open, and a
-- video player at (700, 80) 560x380, outside every checked rectangle. With
-- BLUR set, the shipped blur behind the terminal (matched by its title, so a
-- player that is a kitty too is never blurred), reading the wallpaper (xray,
-- through the memo).
--
--   SOLIUM_CHECKER_PHASE=1  the checker moves, 8 px a second
--   XRAY_SCALE=0.5          effects.xray.scale (1, the default, without it)
--   VEIL=1                  a dark, half-transparent plate on `top` at
--                           (100, 460) 700x340 from the first frame, and no
--                           terminal: the first window to open is the
--                           player; with BLUR the blur is behind the plate
sol.pane("none")
sol.surface("checker", {
    scene = "checker.qml",
    layer = "background",
    properties = { animated = os.getenv("SOLIUM_CHECKER_PHASE") == "1" },
})
local veil = os.getenv("VEIL") == "1"
if veil then
    sol.surface("veil", {
        scene = "veil.qml",
        layer = "top",
        on = { x = 100, y = 460, w = 700, h = 340 },
    })
end
local TERMINAL = { x = 80, y = 80, w = 560, h = 360 }
local PLAYER = { x = 700, y = 80, w = 560, h = 380 }
local opened = 0
sol.on("open", function(id)
    opened = opened + 1
    sol.animate({ duration = 0 })
    sol.place(id, (opened == 1 and not veil) and TERMINAL or PLAYER)
end)
if os.getenv("BLUR") then
    local rule = { match = { title = "effects-check-xray" }, part = "client", slot = "behind", effect = "blur" }
    if veil then
        rule = { match = "*", part = "surface:veil", slot = "behind", effect = "blur" }
    end
    sol.effects({
        rules = { rule },
        xray = { scale = tonumber(os.getenv("XRAY_SCALE") or "1") },
    })
end
