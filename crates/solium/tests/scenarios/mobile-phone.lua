-- The independent mobile policy, real Wayland clients, phone-sized output.
local function shell(world)
    return assert(world.surfaces["mobile-shell"], "mobile shell declared").properties
end
local function page(wanted, count)
    return function(world)
        local ui = shell(world)
        assert(ui.page == wanted, "expected " .. wanted .. ", got " .. ui.page)
        assert(ui.tablet == false, "phone bounds select the phone profile")
        assert(ui.taskCount == count, "the running task count is real")
    end
end
return {
    monitors = { { 390, 844 } },
    init = [[ require("mobile") ]],
    steps = {
        { expect = page("home", 0) },
        -- Desk.window_of settles a full second after mapping; launch's
        -- intermediate poses are checked with live framebuffer captures.
        { open = true, app_id = "mobile-notes" }, { answer = 1 }, { wait = 400 },
        { expect = page("app", 1) },
        { expect = function(world)
            local w = world.windows[1]
            assert(w.x == 0 and w.y == 32 and w.w == 390 and w.h == 776,
                "a phone app leaves space for both system bars")
        end },
        { open = true, app_id = "mobile-gallery" }, { answer = 2 }, { wait = 400 },
        { key = "super+h" }, { wait = 400 }, { expect = page("home", 2) },
        { expect = function(world)
            for _, window in ipairs(world.windows) do
                assert(window.drawn.x >= 390, "Home parks real clients offstage")
            end
        end },
        { key = "super+a" }, { expect = page("apps", 2) },
        { key = "escape" }, { expect = page("home", 2) },
        { key = "super+space" }, { wait = 400 }, { expect = page("recents", 2) },
        { expect = function(world)
            local ui = shell(world)
            assert(#ui.cards == 2, "one live thumbnail slot per task")
            local card = ui.cards[1]
            assert(card.w < 390 and card.h < 776 and card.x >= 0,
                "the active app shrinks to an on-screen card")
        end },
        { key = "super+bracketright" }, { wait = 400 },
        { expect = function(world)
            assert(shell(world).cards[2].selected, "Next selects the older task without changing MRU order")
        end },
        { key = "super+shift+s" },
        { expect = function(world) assert(not shell(world).splitActive, "phone does not enter tablet split") end },
        { key = "super+b" }, { wait = 400 }, { expect = page("home", 2) },
    },
}
