-- A single-display mobile session. Membership, navigation and layout have one
-- authority here. Home never closes a client; Recents transforms real panes.
local config = require("mobile.config")
local mobile = {}
local last_geometry
local presentations = {}
local pane_style = "mobile-flat"
local swipe = { offset = 0, lift = 0, id = 0 }
local pending_launch
local card_slots = {}
local dismissals = {}
local home_ids, home_targets = {}, {}
local fading = {}
local corner_target, corner_radius = 0, -1
local home_serial = 0
local state = sol.keep("mobile.v1", {
    page = "home", previous = "home", active = 0, order = {},
    split = {}, ratio = 0.5, picker = 0, closing = {},
})
state.overview_offset = state.overview_offset or 0
state.launcher_icons = state.launcher_icons or {}
local launcher_icons = state.launcher_icons
state.settings = state.settings or { brightness = 1, dark = false, reduceMotion = false }
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

-- Keep the departing cards' exact rectangles and depth. These stay parked at
-- alpha zero until the next overview, including through client configures.
local function fade_recents()
    local page = state.page == "shade" and state.previous or state.page
    if page ~= "recents" then return end
    fading = {}
    for _, card in ipairs(card_slots) do fading[card.id] = card end
end

local function set_corners(radius)
    radius = math.max(0, math.min(30, tonumber(radius) or 0))
    if math.abs(radius - corner_radius) < 0.05 then return end
    corner_radius = radius
    local monitor = screen()
    local scale = monitor and monitor.scale or 1
    sol.effects({ rules = {
        { match = { fullscreen = false }, part = "client", slot = "replace",
          effect = { "mobile-corners", radius = radius * scale } },
    } })
end

local function hide(window, whole)
    local card = fading[window.id]
    if card then
        sol.present(window.id, { x = card.x, y = card.y, w = card.w, h = card.h,
            z = card.z, opacity = 0 })
        return
    end
    if home_ids[window.id] then
        local target = home_targets[window.id]
        -- A window with no visible launcher icon falls down and shrinks out
        -- of view. Never send it sideways across the desktop.
        target = target or { x = whole.x + whole.w / 2 - 24,
            y = whole.y + whole.h + 24, w = 48, h = 48 }
        sol.present(window.id, { x = target.x, y = target.y, w = target.w, h = target.h, opacity = 0 })
        return
    end
    sol.present(window.id, {
        x = whole.x + whole.w + 32, y = whole.y + 64,
        w = math.max(1, whole.w * 0.88), h = math.max(1, whole.h * 0.88), opacity = 0,
    })
end

local function card_metrics(area, tablet)
    if tablet then
        local columns = area.w >= area.h and 3 or 2
        local rows = 2
        local cell_w = (area.w - 64 - (columns - 1) * 28) / columns
        local row_pitch = math.max(80, (area.h - 132) / rows)
        local h = math.max(24, math.min(cell_w * area.h / area.w, row_pitch - 60))
        return h * area.w / area.h, h, cell_w + 28, columns, row_pitch, cell_w
    end
    local w = math.min(area.w * 0.80, 540)
    return w, math.min(area.h * 0.79, w * area.h / math.max(1, area.w)), w * 0.64
end

local function strip_limit(area, count)
    local _, _, stride, columns = card_metrics(area, true)
    return math.max(0, (math.ceil(count / 2) - columns) * stride)
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
        corner_radius = -1
    end
    if ready and not tablet then state.split = {}; state.picker = 0 end
    local ordered, by_id, dialogs = {}, {}, {}
    if ready then ordered, by_id, dialogs = windows() end
    if tablet then state.overview_offset = math.max(-strip_limit(area, #ordered), math.min(0, state.overview_offset)) end
    local placed, cards = {}, {}
    sol.animate(state.settings.reduceMotion and { duration = 0 } or motion or config.motion)
    local content_page = state.page == "shade" and state.previous or state.page
    -- Styles select pane-bound chrome; both have a square client mask.
    -- The self-input shader cuts client pixels; QML supplies only its radius.
    if content_page == "recents" then corner_target = 30
    elseif content_page == "app" then corner_target = 0 end
    if corner_radius < 0 then set_corners(corner_target) end
    local wanted_style = content_page == "recents" and "mobile" or "mobile-flat"
    if pane_style ~= wanted_style then
        pane_style = wanted_style
        sol.pane(wanted_style)
    end
    local card_w = card_metrics(area, tablet)
    sol.pane_values({ mobilePage = state.page, mobileHeaderScale = math.min(2.4, math.max(1, area.w / card_w * 0.8)) })
    card_slots = {}
    local focused_index = 1
    for index, window in ipairs(ordered) do
        if window.id == state.active then focused_index = index end
    end

    for index, window in ipairs(ordered) do
        local slot = area
        if #state.split == 2 and content_page == "app" then
            if window.id == state.split[1] then
                slot = rect(area.x, area.y, area.w * state.ratio - 5, area.h)
            elseif window.id == state.split[2] then
                slot = rect(area.x + area.w * state.ratio + 5, area.y,
                    area.w * (1 - state.ratio) - 5, area.h)
            end
        end
        sol.place(window.id, slot)
        local visible = content_page == "app" and (window.id == state.active
            or (#state.split == 2 and (window.id == state.split[1] or window.id == state.split[2])))
        if content_page == "recents" then
            local card_w, card_h, stride, columns, row_pitch, cell_w = card_metrics(area, tablet)
            local w, h, x, y
            if tablet then
                local column, row = math.floor((index - 1) / 2), (index - 1) % 2
                w, h = card_w, card_h
                x = area.x + 32 + column * stride + (cell_w - w) / 2
                    + state.overview_offset + swipe.offset
                y = area.y + 58 + row * row_pitch
            else
                local distance = index - focused_index + swipe.offset / stride
                local scale = math.max(0.86, 1 - math.abs(distance) * 0.045)
                w, h = card_w * scale, card_h * scale
                x = whole.x + (whole.w - w) / 2 + distance * stride
                y = area.y + (area.h - h) / 2 + math.abs(distance) * 8
            end
            y = y + (window.id == swipe.id and swipe.lift or 0)
            local card = rect(x, y, w, h)
            card.id = window.id
            card.selected = window.id == state.active
            card.z = index -- right cards always overlap left cards; browsing never changes depth
            cards[#cards + 1] = card
            card_slots[#card_slots + 1] = card
            sol.present(window.id, { x = x, y = y, w = w, h = h,
                z = card.z, opacity = 1 })
            presentations[window.id] = "recents"
        elseif visible then
            fading[window.id] = nil
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
        properties = { dark = state.settings.dark, overview = content_page == "recents" },
    })
    sol.surface("mobile-shell", {
        scene = "mobile/Shell.qml", layer = "overlay", on = "primary", interactive = true,
        reserve = { top = config.status, bottom = config.navigation },
        properties = {
            page = state.page, tablet = tablet, cards = cards, taskCount = #ordered,
            cornerRadius = corner_target, cornerDuration = config.motion.duration,
            activeId = state.active, splitActive = #state.split == 2, pickingSplit = state.picker ~= 0,
            statusHeight = config.status, navigationHeight = config.navigation, homeRequest = home_serial,
            shadeBackground = state.previous, brightness = state.settings.brightness,
            darkAppearance = state.settings.dark, reduceMotion = state.settings.reduceMotion,
        },
    })
    -- Own scene controls are routed before this grab. The grab keeps keys and
    -- clicks on real Recents thumbnails from leaking into their clients.
    sol.grab_input(state.page ~= "app")
    if state.page ~= "app" then sol.bind("escape", mobile.back)
    else sol.unbind("escape") end
    sol.status(tablet and "mobile · tablet" or "mobile · phone")
end

local function reset_swipe() swipe = { offset = 0, lift = 0, id = 0 } end

local function app_key(window)
    if window.title == "Notes" or window.app_id == "mobile-notes" then return "notes" end
    if window.title == "Gallery" or window.app_id == "mobile-gallery" then return "gallery" end
    if window.app_id == "mobile-terminal" then return "terminal" end
    return window.app_id:gsub("%.desktop$", "")
end

local function icon_target(window, whole)
    local icon = launcher_icons[app_key(window)]
    if not icon or icon.screen_w ~= whole.w or icon.screen_h ~= whole.h then return end
    return rect(whole.x + icon.x, whole.y + icon.y, icon.size, icon.size)
end

function mobile.home()
    pending_launch = nil
    reset_swipe()
    local ordered = windows()
    local whole = dimensions()
    local previous = state.page == "shade" and state.previous or state.page
    fade_recents()
    home_ids, home_targets = {}, {}
    for _, window in ipairs(ordered) do
        if previous == "app" and window.id == state.active then
            home_ids[window.id] = true
            home_targets[window.id] = icon_target(window, whole)
        end
    end
    home_serial = home_serial + 1
    state.page = "home"; state.picker = 0
    mobile.apply()
end

function mobile.recents()
    fading = {}
    if state.page ~= "recents" and state.page ~= "shade" then state.overview_offset = 0 end
    home_ids, home_targets = {}, {}
    reset_swipe()
    if state.page ~= "recents" and state.page ~= "shade" then state.previous = state.page end
    state.page = "recents"; state.picker = 0
    mobile.apply()
end

function mobile.back()
    reset_swipe()
    if state.picker ~= 0 then state.picker = 0
    elseif state.page == "shade" then state.page = state.previous
    elseif state.page == "recents" then
        if state.previous == "app" then fade_recents(); state.page = "app"
        else mobile.home(); return end
    else mobile.home(); return end
    if state.page == "home" then mobile.home(); return end
    mobile.apply()
    if state.page == "app" and state.active ~= 0 then sol.focus(state.active) end
end

function mobile.select(id)
    fade_recents()
    home_ids, home_targets = {}, {}
    reset_swipe()
    local _, by_id = windows()
    if not by_id[id] or state.picker == id then return end
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
    reset_swipe()
    local ordered = windows()
    if #ordered == 0 then return end
    local current = 1
    for index, window in ipairs(ordered) do if window.id == state.active then current = index end end
    state.active = ordered[math.max(1, math.min(#ordered, current + by))].id
    mobile.apply(config.quick)
end

-- Capture the icon at release, in monitor-local coordinates, before spawning.
-- A launch result consumes this origin once; keyboard launches have no icon.
local function remember_origin(data)
    local origin = data and data.origin
    pending_launch = nil
    if type(origin) ~= "table" then return end
    local x, y, size = tonumber(origin.x), tonumber(origin.y), tonumber(origin.size)
    local whole = dimensions()
    if not x or not y or not size or x ~= x or y ~= y or size ~= size then return end
    if size < 16 or size > 160 or x < 0 or y < 0 or x + size > whole.w + 1 or y + size > whole.h + 1 then return end
    pending_launch = { rect = rect(whole.x + x, whole.y + y, size, size),
        app = data.app, desktop = data.id }
end

function mobile.launch(name, data)
    remember_origin(data)
    if name == "terminal" then
        if pending_launch then pending_launch.program = "foot" end
        sol.spawn("foot", "--override=resize-by-cells=no", "--app-id=mobile-terminal", "--title=Terminal", "/bin/bash", "--noprofile", "--norc")
    elseif name == "notes" or name == "gallery" then
        -- The file paths are shipped assets, selected from a closed set.
        local source = package.searchpath("mobile.policy", package.path)
        if not source then return end
        local root = source:match("^(.*)/lua/mobile/policy.lua$")
        if not root then return end
        if pending_launch then pending_launch.program = config.qml_runner:match("([^/]+)$") end
        sol.spawn(config.qml_runner, root .. "/qml/mobile/demo/" .. (name == "notes" and "Notes.qml" or "Gallery.qml"))
    end
end

local function act(action, data)
    data = type(data) == "table" and data or {}
    local id = math.tointeger(tonumber(data.id))
    if action == "mobile.icons" then
        if tonumber(data.epoch) ~= home_serial or (state.page ~= "home" and state.page ~= "apps") then return end
        local whole = dimensions()
        if tonumber(data.w) ~= whole.w or tonumber(data.h) ~= whole.h then return end
        local icons = {}
        if type(data.items) == "table" then
            for key, icon in pairs(data.items) do
                if type(key) == "string" and type(icon) == "table" then
                    local x, y, size = tonumber(icon.x), tonumber(icon.y), tonumber(icon.size)
                    if x and y and size and size >= 16 and size <= 160 and x >= 0 and y >= 0
                        and x + size <= whole.w + 1 and y + size <= whole.h + 1 then
                        icons[key:gsub("%.desktop$", "")] = { x = x, y = y, size = size,
                            screen_w = whole.w, screen_h = whole.h }
                    end
                end
            end
        end
        launcher_icons = icons
        state.launcher_icons = icons
        local ordered = windows()
        for _, window in ipairs(ordered) do
            if home_ids[window.id] then home_targets[window.id] = icon_target(window, whole) end
        end
        mobile.apply()
    elseif action == "mobile.home" then mobile.home()
    elseif action == "mobile.back" then mobile.back()
    elseif action == "mobile.recents" then mobile.recents()
    elseif action == "mobile.apps" then state.page = "apps"; mobile.apply()
    elseif action == "mobile.shade" then
        if state.page == "shade" then mobile.back()
        else state.previous = state.page; state.page = "shade"; mobile.apply() end
    elseif action == "mobile.select" and id then mobile.select(id)
    elseif action == "mobile.previous" then mobile.cycle(-1)
    elseif action == "mobile.next" then mobile.cycle(1)
    elseif action == "mobile.launch" then mobile.launch(data.app, data)
    elseif action == "mobile.desktop" and type(data.id) == "string" then
        remember_origin(data)
        local origin = pending_launch
        sol.act("apps.launch", { id = data.id }, function(ok)
            if not ok and pending_launch == origin then pending_launch = nil end
        end)
    elseif action == "mobile.close" and id then
        local _, by_id = windows()
        if by_id[id] then sol.close(id) end
    elseif action == "mobile.split" and id then
        local _, _, tablet = dimensions()
        local ordered, by_id = windows()
        if tablet and by_id[id] and #ordered > 1 then state.picker = id; mobile.apply() end
    elseif action == "mobile.swipe.begin" and state.page == "recents" then
        reset_swipe()
        local whole, area = dimensions()
        local x, y = (tonumber(data.x) or 0) + whole.x, (tonumber(data.y) or 0) + whole.y
        local top
        for _, card in ipairs(card_slots) do
            if x >= card.x and x <= card.x + card.w and y >= card.y - 80 * card.w / area.w and y <= card.y + card.h
                and (not top or card.z > top.z) then top = card end
        end
        swipe.id = top and top.id or 0
    elseif action == "mobile.swipe.move" and state.page == "recents" then
        local dx, dy = tonumber(data.dx) or 0, tonumber(data.dy) or 0
        if math.abs(dy) > math.abs(dx) and dy < 0 then swipe.lift = dy; swipe.offset = 0
        else
            local ordered = windows()
            local _, area, tablet = dimensions()
            local _, _, stride = card_metrics(area, tablet)
            local current = 1
            for index, window in ipairs(ordered) do if window.id == state.active then current = index end end
            if tablet then
                swipe.offset = math.max(-strip_limit(area, #ordered) - state.overview_offset - 32,
                    math.min(-state.overview_offset + 32, dx))
            else
                swipe.offset = math.max(-(#ordered - current) * stride - 40,
                    math.min((current - 1) * stride + 40, dx))
            end
            swipe.lift = 0
        end
        mobile.apply({ duration = 0 })
    elseif action == "mobile.swipe.end" and state.page == "recents" then
        local _, area, tablet = dimensions()
        local _, _, stride = card_metrics(area, tablet)
        local offset, lift, selected = swipe.offset, swipe.lift, swipe.id
        reset_swipe()
        if lift < -85 and selected ~= 0 then
            for _, card in ipairs(card_slots) do
                if card.id == selected then dismissals[selected] = card; break end
            end
            sol.close(selected)
        elseif math.abs(offset) < 8 and math.abs(lift) < 8 and data.tap then
            if selected ~= 0 then mobile.select(selected) else mobile.home() end
        elseif tablet then
            local ordered = windows()
            local velocity = state.settings.reduceMotion and 0 or math.max(-2, math.min(2, tonumber(data.velocity) or 0))
            state.overview_offset = math.max(-strip_limit(area, #ordered),
                math.min(0, state.overview_offset + offset + velocity * 150))
            mobile.apply({ duration = 320, easing = "outCubic" })
        else mobile.cycle(math.floor(-offset / stride + 0.5)) end
    elseif action == "mobile.corners" then
        set_corners(data.radius)
    elseif action == "mobile.setting" then
        if data.key == "brightness" then
            state.settings.brightness = math.max(0.25, math.min(1, tonumber(data.value) or 1))
        elseif data.key == "dark" then state.settings.dark = not state.settings.dark
        elseif data.key == "reduceMotion" then state.settings.reduceMotion = not state.settings.reduceMotion
        end
        mobile.apply(config.quick)
    elseif action == "mobile.reload" then sol.reload()
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
        -- A delayed, unrelated window must not inherit a previous icon.
        local origin = pending_launch
        local window = by_id[id]
        local matches = origin and ((window.app_id == "" and (origin.desktop or window.title == origin.program))
            or (origin.app == "notes" and window.title == "Notes")
            or (origin.app == "gallery" and window.title == "Gallery")
            or (origin.app == "terminal" and window.app_id == "mobile-terminal")
            or (origin.desktop and (window.app_id == origin.desktop or window.app_id .. ".desktop" == origin.desktop)))
        if matches then
            sol.present_from(id, origin.rect)
            pending_launch = nil
        else
            sol.present_from(id, { x = area.x + area.w * 0.04, y = area.y + area.h * 0.04,
                w = area.w * 0.92, h = area.h * 0.92, opacity = 0 })
        end
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
    if why == "launch" then
        local origin = pending_launch
        pending_launch = nil
        mobile.select(id)
        if origin then sol.present_from(id, origin.rect) end
    end
end)
sol.on("closing", function(id)
    local card = dismissals[id]
    state.closing[id] = true
    mobile.apply()
    if card then
        local whole = dimensions()
        -- Native close requests are delivered after the compositor's 190 ms
        -- closing phase. Land off-screen before that, with no on-screen fade.
        sol.animate({ duration = state.settings.reduceMotion and 0 or 180, easing = "outCubic" })
        sol.present(id, { x = card.x, y = whole.y - card.h - 48,
            w = card.w, h = card.h, z = card.z, opacity = 1 })
    end
end)
sol.on("close", function(id)
    dismissals[id] = nil; home_ids[id] = nil; home_targets[id] = nil; fading[id] = nil
    state.closing[id] = nil -- close's snapshot marks the departing pane leaving
    mobile.apply()
    if state.page == "app" and state.active ~= 0 then sol.focus(state.active) end
end)
sol.on("refused", function(id)
    dismissals[id] = nil; state.closing[id] = nil; mobile.apply()
end)
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
