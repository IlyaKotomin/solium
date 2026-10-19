local function shell(world)
    return assert(world.surfaces["mobile-shell"]).properties
end
return {
    monitors = { { 1280, 800 } },
    init = [[ require("mobile") ]],
    steps = {
        { expect = function(world)
            assert(shell(world).tablet and shell(world).page == "home", "tablet starts at its launcher")
        end },
        { open = true, app_id = "mobile-notes" }, { answer = 1 },
        { open = true, app_id = "mobile-gallery" }, { answer = 2 },
        { key = "super+shift+s" }, { answer = 1 }, { answer = 2 }, { wait = 400 },
        { expect = function(world)
            assert(shell(world).splitActive, "two real clients enter split screen")
            local a, b = world.windows[1], world.windows[2]
            assert(a.w == 635 and b.w == 635 and a.h == 704 and b.h == 704,
                "split tiles share usable bounds with a ten-pixel divider")
            assert(math.abs(a.x - b.x) == 645 and a.y == 32 and b.y == 32,
                "the two clients never overlap or cover the navigation bar")
        end },
        { key = "super+r" }, { answer = 1 }, { answer = 2 }, { wait = 400 },
        { expect = function(world)
            local a, b = world.windows[1], world.windows[2]
            assert(a.w ~= b.w and a.w + b.w == 1270, "resizing the split preserves total width")
        end },
        { key = "super+h" }, { wait = 400 },
        { expect = function(world) assert(shell(world).page == "home" and shell(world).taskCount == 2) end },
        { key = "super+space" }, { wait = 400 },
        { expect = function(world)
            assert(shell(world).page == "recents" and #shell(world).cards == 2,
                "both split clients remain available in Recents")
        end },
        { key = "super+b" }, { key = "super+shift+s" }, { key = "super+u" },
        { answer = 1 }, { answer = 2 }, { wait = 400 },
        { expect = function(world)
            assert(not shell(world).splitActive and shell(world).page == "app")
            local active = shell(world).activeId
            for _, window in ipairs(world.windows) do
                if window.id == active then
                    assert(window.w == 1280 and window.h == 704, "single-app mode fills the usable display")
                end
            end
        end },
    },
}
