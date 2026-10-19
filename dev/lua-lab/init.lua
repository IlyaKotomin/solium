-- SOLIUM_LUA_INIT=/src/dev/lua-lab/init.lua
-- The entry's directory precedes the shipped module directory in package.path:
-- config.lua here is the entire configuration, without merging defaults.
sol.qml({ renderer = "software" })
sol.session({ systemd = false })
sol.pane("none")

local modes = require("modes")
require("workspaces")
require("tiling")
require("scrolling")
require("direction")

sol.bind("super+return", function()
    sol.spawn("foot", "/bin/bash", "--noprofile", "--norc")
end)
sol.bind("super+q", function()
    for _, window in ipairs(sol.windows()) do
        if window.focused then
            sol.close(window.id)
            return
        end
    end
end)
sol.bind("super+shift+r", function() sol.reload() end)
sol.bind("super+shift+q", function() sol.quit() end)

-- Start tiled once; a reload uses the normal restore event.
local lab = sol.keep("lua-layout-lab", { started = false })
if not lab.started then
    lab.started = true
    modes.use("tiling")
end
