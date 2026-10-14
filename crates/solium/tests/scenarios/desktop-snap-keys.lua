-- Desktop mode's snap keys, pressed through the real input path with
-- Russian live, on a window placed by `placement` (#222, kept by #240):
-- `super+left` and `super+right` snap the focused window to a half,
-- `super+up` from a half goes to its top quarter, and `super+down` puts it
-- back where it was placed.
--
-- Played by `scenario::tests::every_scenario_with_a_client_passes`.

local function show(window)
    return string.format("%d,%d %dx%d", window.x, window.y, window.w, window.h)
end

local function is(n, x, y, w, h, why)
    return function(world)
        local window = world.windows[n]
        assert(window.x == x and window.y == y and window.w == w and window.h == h,
            why .. ": window " .. n .. " at " .. x .. "," .. y .. " " .. w .. "x" .. h .. ": "
                .. show(window))
    end
end

return {
    init = [[
        require("modes")
        require("open")
        require("workspaces")
        require("tiling")
        require("scrolling")
        require("direction")
        require("floating")
    ]],
    steps = {
        { open = true },
        { open = true },
        { expect = is(2, 892, 522, 200, 100, "the premise: placed one step from the first") },
        { key = "super+left" },
        { answer = 2 },
        { expect = is(2, 0, 0, 960, 1080, "super+left: the left half") },
        { key = "super+right" },
        { answer = 2 },
        { expect = is(2, 960, 0, 960, 1080, "super+right: the right half") },
        { key = "super+up" },
        { answer = 2 },
        { expect = is(2, 960, 0, 960, 540, "super+up from a half: its top quarter") },
        { key = "super+down" },
        { answer = 2 },
        { expect = is(2, 892, 522, 200, 100, "super+down: back where it was placed") },
    },
}
