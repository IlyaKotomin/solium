-- effects-check's `motion` section (Task 54): a clear terminal (its
-- background fully transparent, so all of it is what is behind it) with the
-- wallpaper blurred behind it, over bars.qml, a wallpaper no two places of
-- which look alike. REF=1 places it over the whole monitor: the wallpaper
-- blurred everywhere, the reference. Otherwise it is laid out at WINDOW, and
--   super+p  presents it at HELD (`sol.present`, at once): drawn away from
--            its layout place, as a glide's frame is, and held still;
--   super+m  glides it to GLIDE over GLIDE_MS (3000 unless set), linearly,
--            as a relayout does;
--   super+g  holds it half way through the shipped genie.
-- EFFECT is the effect behind it ("blur", or "blur-tint" for a chain with a
-- step of its own per part); WARPED is effects.xray.warped (the control runs
-- "rest"); NORULE=1 sets no rule: the sharp wallpaper the judge measures the
-- window against.
sol.pane("none")
sol.surface("bars", { scene = "bars.qml", layer = "background" })
local WINDOW = { x = 80, y = 120, w = 560, h = 360 }
local HELD = { x = 900, y = 300, w = 560, h = 360 }
local GLIDE = { x = 880, y = 120, w = 560, h = 360 }
if os.getenv("NORULE") ~= "1" then
    sol.effects({
        xray = { warped = os.getenv("WARPED") or "drawn" },
        rules = {
            {
                match = { title = "effects-check-clear" },
                part = "client",
                slot = "behind",
                effect = os.getenv("EFFECT") or "blur",
            },
        },
    })
end
sol.on("open", function(id)
    sol.animate({ duration = 0 })
    if os.getenv("REF") == "1" then
        sol.place(id, { x = 0, y = 0, w = 1600, h = 900 })
    else
        sol.place(id, WINDOW)
    end
end)
local function each(f)
    for _, window in ipairs(sol.windows()) do
        f(window.id)
    end
end
sol.bind("super+p", function()
    each(function(id)
        sol.animate({ duration = 0 })
        sol.present(id, HELD)
    end)
end)
sol.bind("super+m", function()
    each(function(id)
        sol.animate({ duration = tonumber(os.getenv("GLIDE_MS") or "3000"), easing = "linear" })
        sol.place(id, GLIDE)
    end)
end)
sol.bind("super+g", function()
    each(function(id)
        sol.animate({ duration = 0 })
        sol.present(id, {
            deform = { effect = "genie", progress = 0.5, axis = "down", to = { x = 600, y = 1000, w = 120, h = 24 } },
        })
    end)
end)
