-- An independent configuration for the layout investigation. No default
-- init, user config, shell, wallpaper, effects or cursor warping is loaded.
local instant = { duration = 0 }

return {
    gap = 8,
    wallpaper = false,
    bindings = {},
    focus = {
        click = true,
        follow = false,
        clear_on_empty_click = true,
        modes = {},
        after_close = {},
        cursor = { follows = {}, empty_workspace = "stay" },
    },
    resize = {},
    floating = { client_limits = true },
    tiling = {
        split = 0.5,
        minimum = { w = 120, h = 80 },
        client_minimum = true,
        client_maximum = true,
        client_size_ignore = {},
        overflow = { "allow" },
        follow_overflow = false,
        move = "swap",
        reflow_on_close = "immediate",
        motion = instant,
        snap = instant,
    },
    scrolling = {
        widths = { 0.5, 1 },
        default_width = 1,
        reflow_on_close = "immediate",
        motion = instant,
        snap = instant,
    },
    workspaces = {
        per_monitor = true,
        arrangement = "horizontal",
        columns = 3,
        rows = 1,
        spread = 1,
        follow_new_windows = true,
        motion = instant,
    },
}
