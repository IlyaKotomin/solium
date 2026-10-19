-- A single-display mobile session. Membership, navigation and layout have one
-- authority here. Home never closes a client; Recents transforms real panes.
local config = require("mobile.config")
local mobile = {}
local last_geometry
local presentations = {}
local state = sol.keep("mobile.v1", {
    page = "home", previous = "home", active = 0, order = {},
    split = {}, ratio = 0.5, picker = 0, closing = {},
})
local pages = { home = true, apps = true, recents = true, app = true, shade = true }
if not pages[state.page] then state.page = "home" end

local function screen()
    local all = sol.monitors()
    for _, monitor in ipairs(all) do
        if monitor.primary then return monitor end
    end
    return all[1]
end

local function dimensions()
    local monitor = screen()
    local whole = monitor and (monitor.whole or monitor) or { x = 0, y = 0, w = 390, h = 844 }
    local tablet = config.profile == "tablet"
        or (config.profile ~= "phone" and math.min(whole.w, whole.h) >= 600)
    return whole, {
        x = whole.x, y = whole.y + config.status,
        w = whole.w, h = math.max(1, whole.h - config.status - config.navigation),
    }, tablet
end

local function promote(id)
    local out = { id }
    for _, other in ipairs(state.order) do
        if other ~= id then out[#out + 1] = other end
    end
    state.order = out
end

local function windows()
    local monitor = screen()
    local by_id, dialogs = {}, {}
    for _, window in ipairs(sol.windows()) do
        if (not monitor or window.monitor == monitor.name)
            and not window.leaving and not state.closing[window.id] then
            if window.modal then dialogs[#dialogs + 1] = window
            else by_id[window.id] = window end
        end
    end
    for id in pairs(presentations) do
        if not by_id[id] then presentations[id] = nil end
    end
    local ordered, held = {}, {}
    for _, id in ipairs(state.order) do
        if by_id[id] then
            ordered[#ordered + 1] = by_id[id]
            held[id] = true
        end
    end
    for _, window in ipairs(sol.windows()) do
        if by_id[window.id] and not held[window.id] then
            ordered[#ordered + 1] = window
            held[window.id] = true
        end
    end
    state.order = {}
    for _, window in ipairs(ordered) do state.order[#state.order + 1] = window.id end
    if not by_id[state.active] then state.active = ordered[1] and ordered[1].id or 0 end
    if #state.split == 2 and (not by_id[state.split[1]] or not by_id[state.split[2]]) then
        state.split = {}
    end
    if not by_id[state.picker] then state.picker = 0 end
    if state.page == "app" and state.active == 0 then state.page = "home" end
    return ordered, by_id, dialogs
end

local function rect(x, y, w, h)
    return { x = x, y = y, w = math.max(1, w), h = math.max(1, h) }
end

local function hide(window, whole)
    sol.present(window.id, {
        x = whole.x + whole.w + 32, y = whole.y + 64,
        w = math.max(1, whole.w * 0.88), h = math.max(1, whole.h * 0.88), opacity = 0,
    })
end

function mobile.apply(motion)
    -- Loading a new Lua VM has no live snapshot yet. Keep its committed task
    -- state until restore/monitors supplies one; no snapshot is not no tasks.
    local ready = screen() ~= nil
    local whole, area, tablet = dimensions()
    local geometry = string.format("%s %dx%d", tablet and "tablet" or "phone", whole.w, whole.h)
    if ready and geometry ~= last_geometry then
        sol.log("mobile layout: " .. geometry)
        last_geometry = geometry
        presentations = {}
    end
    if ready and not tablet then state.split = {}; state.picker = 0 end
    local ordered, by_id, dialogs = {}, {}, {}
    if ready then ordered, by_id, dialogs = windows() end
    local placed, cards = {}, {}
    sol.animate(motion or config.motion)
    local focused_index = 1
    for index, window in ipairs(ordered) do
        if window.id == state.active then focused_index = index end
    end

    for index, window in ipairs(ordered) do
        local slot = area
        if #state.split == 2 and state.page == "app" then
            if window.id == state.split[1] then
                slot = rect(area.x, area.y, area.w * state.ratio - 5, area.h)
            elseif window.id == state.split[2] then
                slot = rect(area.x + area.w * state.ratio + 5, area.y,
                    area.w * (1 - state.ratio) - 5, area.h)
            end
        end
        sol.place(window.id, slot)
        local visible = state.page == "app" and (window.id == state.active
            or (#state.split == 2 and (window.id == state.split[1] or window.id == state.split[2])))
        if state.page == "recents" then
            local card_w = math.min(area.w * (tablet and 0.46 or 0.78), 520)
            local card_h = math.max(48, math.min(area.h * 0.65,
                card_w * area.h / math.max(area.w, 1), area.h - 228))
            local x = whole.x + (whole.w - card_w) / 2 + (index - focused_index) * (card_w + 24)
            local y = area.y + math.max(96, (area.h - card_h) / 2 - 12)
            local card = rect(x, y, card_w, card_h)
            card.id = window.id
            card.title = window.title ~= "" and window.title or (window.app_id ~= "" and window.app_id or "Opening app")
            card.selected = window.id == state.active
            cards[#cards + 1] = card
            sol.present(window.id, { x = x, y = y, w = card_w, h = card_h, opacity = 1 })
            presentations[window.id] = "recents"
        elseif visible then
            -- Repeated layout/configure events must not cancel the entry
            -- transition started by open after its initial placement.
            if presentations[window.id] ~= "app" then sol.present_clear(window.id) end
            presentations[window.id] = "app"
            placed[window.id] = slot
        else
            -- A client configure can update the placement beneath a parked
            -- pane. Reassert its hidden transform on every layout pass.
            hide(window, whole)
            presentations[window.id] = "hidden"
        end
    end

    for _, window in ipairs(dialogs) do
        local parent = placed[window.parent]
        if parent then
            local w, h = math.min(window.w, parent.w - 24), math.min(window.h, parent.h - 24)
            sol.place(window.id, rect(parent.x + (parent.w - w) / 2,
                parent.y + (parent.h - h) / 2, w, h))
            sol.present_clear(window.id)
        else hide(window, whole) end
    end

    sol.surface("mobile-wallpaper", {
        scene = "mobile/Wallpaper.qml", layer = "background", on = "primary",
    })
    sol.surface("mobile-shell", {
        scene = "mobile/Shell.qml", layer = "overlay", on = "primary", interactive = true,
        reserve = { top = config.status, bottom = config.navigation },
        properties = {
            page = state.page, tablet = tablet, cards = cards, taskCount = #ordered,
            activeId = state.active, splitActive = #state.split == 2, pickingSplit = state.picker ~= 0,
            statusHeight = config.status, navigationHeight = config.navigation,
        },
    })
    -- Own scene controls are routed before this grab. The grab keeps keys and
    -- clicks on real Recents thumbnails from leaking into their clients.
    sol.grab_input(state.page ~= "app")
    if state.page ~= "app" then sol.bind("escape", mobile.back)
    else sol.unbind("escape") end
    sol.status(tablet and "mobile · tablet" or "mobile · phone")
end

function mobile.home()
    state.page = "home"; state.picker = 0
    mobile.apply()
end

function mobile.recents()
    if state.page ~= "recents" then state.previous = state.page end
    state.page = "recents"; state.picker = 0
    mobile.apply()
end

function mobile.back()
    if state.picker ~= 0 then state.picker = 0
    elseif state.page == "shade" then state.page = state.previous
    elseif state.page == "recents" then state.page = state.previous == "app" and "app" or "home"
    else state.page = "home" end
    mobile.apply()
    if state.page == "app" and state.active ~= 0 then sol.focus(state.active) end
end

function mobile.select(id)
    local _, by_id = windows()
    if not by_id[id] then return end
    if state.picker ~= 0 and state.picker ~= id then
        state.split = { state.picker, id }; state.picker = 0
    elseif #state.split == 2 and id ~= state.split[1] and id ~= state.split[2] then
        state.split = {}
    end
    state.active = id; state.page = "app"; promote(id)
    mobile.apply()
    sol.focus(id)
end

function mobile.cycle(by)
    local ordered = windows()
    if #ordered == 0 then return end
    local current = 1
    for index, window in ipairs(ordered) do if window.id == state.active then current = index end end
    state.active = ordered[math.max(1, math.min(#ordered, current + by))].id
    mobile.apply(config.quick)
end

function mobile.launch(name)
    if name == "terminal" then
        sol.spawn("foot", "--app-id=mobile-terminal", "--title=Terminal", "/bin/bash", "--noprofile", "--norc")
    elseif name == "notes" or name == "gallery" then
        -- The file paths are shipped assets, selected from a closed set.
        local source = package.searchpath("mobile.policy", package.path)
        if not source then return end
        local root = source:match("^(.*)/lua/mobile/policy.lua$")
        if not root then return end
        sol.spawn(config.qml_runner, root .. "/qml/mobile/demo/" .. (name == "notes" and "Notes.qml" or "Gallery.qml"))
    end
end

local function act(action, data)
    data = type(data) == "table" and data or {}
    local id = math.tointeger(tonumber(data.id))
    if action == "mobile.home" then mobile.home()
    elseif action == "mobile.back" then mobile.back()
    elseif action == "mobile.recents" then mobile.recents()
    elseif action == "mobile.apps" then state.page = "apps"; mobile.apply()
    elseif action == "mobile.shade" then
        if state.page == "shade" then mobile.back()
        else state.previous = state.page; state.page = "shade"; mobile.apply() end
    elseif action == "mobile.select" and id then mobile.select(id)
    elseif action == "mobile.previous" then mobile.cycle(-1)
    elseif action == "mobile.next" then mobile.cycle(1)
    elseif action == "mobile.launch" then mobile.launch(data.app)
    elseif action == "mobile.desktop" and type(data.id) == "string" then
        sol.act("apps.launch", { id = data.id })
    elseif action == "mobile.close" and id then
        local _, by_id = windows()
        if by_id[id] then sol.close(id) end
    elseif action == "mobile.split" and id then
        local _, _, tablet = dimensions()
        local ordered, by_id = windows()
        if tablet and by_id[id] and #ordered > 1 then state.picker = id; mobile.apply() end
    elseif action == "mobile.unsplit" then state.split = {}; mobile.apply()
    elseif action == "mobile.ratio" then
        state.ratio = state.ratio >= 0.59 and 0.4 or state.ratio + 0.1; mobile.apply()
    end
end

sol.on("surface", function(surface, action, data)
    if surface == "mobile-shell" then act(action, data) end
end)
sol.on("open", function(id)
    local _, by_id = windows()
    if by_id[id] then
        state.active = id; state.page = "app"; state.split = {}; state.picker = 0; promote(id)
    end
    mobile.apply()
    if by_id[id] then
        local _, area = dimensions()
        sol.present_from(id, { x = area.x + area.w * 0.08, y = area.y + area.h * 0.1,
            w = area.w * 0.84, h = area.h * 0.84, opacity = 0 })
    end
end)
sol.on("focus", function(id)
    if state.page == "app" and state.active ~= id then
        local _, by_id = windows()
        if by_id[id] then state.active = id; promote(id); mobile.apply(config.quick) end
    end
end)
sol.on("activate", function(id, why)
    -- Only a launch the compositor matched, never an unsolicited activation.
    if why == "launch" then mobile.select(id) end
end)
sol.on("closing", function(id) state.closing[id] = true; mobile.apply() end)
sol.on("close", function(id)
    state.closing[id] = nil -- close's snapshot marks the departing pane leaving
    mobile.apply()
    if state.page == "app" and state.active ~= 0 then sol.focus(state.active) end
end)
sol.on("refused", function(id) state.closing[id] = nil; mobile.apply() end)
sol.on("layout", function() mobile.apply(config.quick) end)
sol.on("monitors", function() mobile.apply({ duration = 0 }) end)
sol.on("restore", function() mobile.apply({ duration = 0 }) end)

sol.bind("super+h", mobile.home)
sol.bind("super+space", mobile.recents)
sol.bind("super+a", function() act("mobile.apps") end)
sol.bind("super+b", mobile.back)
sol.bind("super+n", function() mobile.launch("notes") end)
sol.bind("super+g", function() mobile.launch("gallery") end)
sol.bind("super+return", function() mobile.launch("terminal") end)
sol.bind("super+bracketleft", function() mobile.cycle(-1) end)
sol.bind("super+bracketright", function() mobile.cycle(1) end)
sol.bind("super+r", function() act("mobile.ratio") end)
sol.bind("super+u", function() act("mobile.unsplit") end)
sol.bind("super+shift+s", function()
    local ordered = windows()
    local _, _, tablet = dimensions()
    if tablet and #ordered >= 2 then
        state.split = { ordered[1].id, ordered[2].id }; state.page = "app"; mobile.apply()
    end
end)
sol.bind("super+shift+r", function() sol.reload() end)
sol.bind("super+shift+q", function() sol.quit() end)
return mobile
