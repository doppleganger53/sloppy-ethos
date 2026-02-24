-- Ethos system tool wrapper for the upstream ethos_events helper.
-- Source: FrSkyRC/ETHOS-Feedback-Community (branch 1.6)

local function loadEventsHelper()
  local loader = rawget(_G, "loadScript")
  if type(loader) == "function" then
    local candidates = {
      "/scripts/ethos_events/ethos_events.lua",
      "/SCRIPTS/ethos_events/ethos_events.lua",
      "scripts/ethos_events/ethos_events.lua",
      "SCRIPTS/ethos_events/ethos_events.lua",
      "ethos_events.lua",
    }
    for _, path in ipairs(candidates) do
      local chunk = loader(path)
      if type(chunk) == "function" then
        local loaded = chunk()
        if type(loaded) == "table" and type(loaded.debug) == "function" then
          return loaded
        end
      end
    end
  end

  -- Last-resort fallback keeps the tool usable if helper loading fails.
  return {
    debug = function(tag, category, value, x, y)
      print(
        string.format(
          "[%s] category=%s value=%s x=%s y=%s",
          tostring(tag or "event"),
          tostring(category),
          tostring(value),
          tostring(x),
          tostring(y)
        )
      )
    end,
  }
end

local events = loadEventsHelper()

local function name()
  return "Ethos Events"
end

local function loadIcon()
  if type(lcd) ~= "table" or type(lcd.loadMask) ~= "function" then
    return nil
  end
  local iconCandidates = {
    "/scripts/ethos_events/ethos_events.png",
    "/SCRIPTS/ethos_events/ethos_events.png",
    "ethos_events.png",
  }
  for _, path in ipairs(iconCandidates) do
    local ok, icon = pcall(lcd.loadMask, path)
    if ok and icon then
      return icon
    end
  end
  return nil
end

local icon = loadIcon()

local function create()
  return {}
end

local function wakeup(widget)
end

local function paint(widget)
  if type(lcd) ~= "table" then
    return
  end
  if type(lcd.drawText) == "function" then
    lcd.drawText(8, 8, "Ethos Events active.")
    lcd.drawText(8, 24, "Check console for event logs.")
  end
end

local function event(widget, category, value, x, y)
  events.debug("ethos_events", category, value, x, y)
end

local function init()
  print("[ethos_events] init() called")
  local ok, err = pcall(system.registerSystemTool, {
    name = name,
    icon = icon,
    create = create,
    wakeup = wakeup,
    paint = paint,
    event = event,
  })
  if not ok then
    print("[ethos_events] registerSystemTool failed: " .. tostring(err))
  else
    print("[ethos_events] registerSystemTool ok")
  end
end

return { init = init }
