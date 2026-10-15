-- The `glass` style: a bar along the bottom of the window with no fill at all,
-- focused or not, so the glass behind the pane shows through it; and, while
-- the window does not have the keyboard, a thin dark shade just inside the
-- window's rounded outline, on all four sides and through the corners, that
-- comes and goes with the focus, animated both ways. The shade's picture is
-- drawn once and the fade is its `layerOpacity`, which the compositor applies
-- as it composites it: `world.opacity.shade`.
--
-- Played by `scenario::tests::every_scenario_on_the_qt_thread_passes`, in
-- software. The pane is 400 by 300, its bar the bottom 32 rows, 268 to 299,
-- and its corners 16 round. The buttons are at the left (`pane-buttons-drawn.lua`
-- reads them), the title, "terminal", right after them, inside the box from
-- (56, 276) to (130, 292), and with no app id told there is no icon at the
-- right. x = 200 of the bar is bar and nothing else. The shade is the layer
-- `shade`, 12 px deep.

-- The alpha of a layer at (x, y), and the lightest of its channels:
-- premultiplied, so a dark shade is near-black at any alpha.
local function at(world, layer, x, y)
    local r, g, b, a = world.pixel(layer, x, y)
    return a, math.max(r, g, b)
end

-- The bar draws nothing at x = 200, nor where an icon would be.
local function bar_clear(world, what)
    for y = 268, 299 do
        local a = at(world, "bar", 200, y)
        assert(a == 0, string.format("%s: the bar at (200, %d) has alpha %d, not nothing", what, y, a))
    end
    for y = 276, 292 do
        for x = 372, 388 do
            local a = at(world, "bar", x, y)
            assert(a == 0, string.format("%s: an icon at (%d, %d) with no app id", what, x, y))
        end
    end
end

-- The lightest pixel of the title's box: a grey, so light text on nothing.
local function title(world)
    local lightest = 0
    for y = 276, 292 do
        for x = 56, 130 do
            local r, g, b = world.pixel("bar", x, y)
            assert(math.abs(r - g) <= 2 and math.abs(g - b) <= 2,
                string.format("(%d, %d) of the title is %d %d %d, not a grey", x, y, r, g, b))
            lightest = math.max(lightest, r)
        end
    end
    return lightest
end

-- The shade at rest: composited whole, and its picture dark at the very edge
-- of all four sides, fainter a few pixels in, nothing past its depth nor in
-- the middle, and round at the corners, nothing outside the curve and dark on
-- it. Answers the top edge's alpha.
local function shaded(world)
    assert(not world.dormant.shade, "the shade is dormant on a window without the keyboard")
    assert(world.opacity.shade == 1, "the shade is composited at " .. world.opacity.shade)
    for _, edge in ipairs({ { 200, 0 }, { 0, 150 }, { 399, 150 }, { 200, 299 } }) do
        local a, light = at(world, "shade", edge[1], edge[2])
        assert(a >= 100 and light <= 8,
            string.format("the shade at (%d, %d) is %d, lightest %d: not a dark edge", edge[1], edge[2], a, light))
    end
    local edge = at(world, "shade", 200, 0)
    local inward = at(world, "shade", 200, 6)
    assert(inward > 0 and inward < edge - 24,
        string.format("the shade does not fall away inward: %d at the edge, %d 6 px in", edge, inward))
    for _, inside in ipairs({ { 200, 14 }, { 14, 150 }, { 200, 150 } }) do
        local a = at(world, "shade", inside[1], inside[2])
        assert(a == 0, string.format("the shade reaches (%d, %d): %d", inside[1], inside[2], a))
    end
    -- The top-left corner's curve: (1, 1) is outside it, (5, 5) on it.
    local outside, on = at(world, "shade", 1, 1), at(world, "shade", 5, 5)
    assert(outside <= 8 and on >= 60,
        string.format("the shade does not follow the round corner: %d outside it, %d on it", outside, on))
    return edge
end

local seen = {}

-- Part of the way between nothing and the shade at rest: the change is
-- animated, not a cut, and the picture is the one at rest.
local function on_its_way(world, which)
    local alpha = world.opacity.shade
    assert(alpha > 0.05 and alpha < 0.95,
        string.format("the shade did not animate %s: composited at %.3f after 48 ms", which, alpha))
    assert(not world.dormant.shade, which .. ": the shade's layer slept mid-fade")
    assert(at(world, "shade", 200, 0) == seen.edge, which .. ": the shade's picture changed while it faded")
end

-- A window with the keyboard: no shade, composited at nothing and its layer
-- asleep.
local function unshaded(world, what)
    assert(world.dormant.shade, what .. ": the shade's layer is not dormant")
    assert(world.opacity.shade == 0, string.format("%s: the shade composited at %.3f", what, world.opacity.shade))
end

return {
    qt = true,
    steps = {
        { pane = "glass", client = { 400, 268 } },
        { tell = { focused = true, title = "terminal" } },
        { wait = 400 },
        {
            expect = function(world)
                unshaded(world, "focused")
                bar_clear(world, "focused")
                seen.title = title(world)
                assert(seen.title >= 160, string.format("the focused title is not light: %d", seen.title))
            end,
        },

        { tell = { focused = false } },
        { wait = 400 },
        {
            expect = function(world)
                seen.edge = shaded(world)
                bar_clear(world, "unfocused")
                local dim = title(world)
                assert(dim < seen.title,
                    string.format("an unfocused title (%d) is not dimmer than a focused one (%d)", dim, seen.title))
            end,
        },

        -- Given the keyboard: on its way out, then gone.
        { tell = { focused = true } },
        { wait = 32 },
        { expect = function(world) on_its_way(world, "out") end },
        { wait = 400 },
        { expect = function(world) unshaded(world, "focused again") end },

        -- Losing it again: on its way in, then at rest as before.
        { tell = { focused = false } },
        { wait = 32 },
        { expect = function(world) on_its_way(world, "in") end },
        { wait = 400 },
        { expect = function(world) shaded(world) end },
    },
}
