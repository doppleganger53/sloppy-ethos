-- Ethos system tool wrapper for the upstream ethos_events helper.
-- Source: FrSkyRC/ETHOS-Feedback-Community (branch 1.6)

local LOG_CAPACITY = 80
local TOGGLE_ROW_Y = 40
local TOGGLE_ROW_HEIGHT = 16
local LOG_START_Y = 72
local LOG_LINE_HEIGHT = 14
local FONT_HEADER = rawget(_G, "FONT_STD") or rawget(_G, "FONT_XS")
local FONT_EVENTS = rawget(_G, "FONT_XS") or rawget(_G, "FONT_STD")

local function findConstantName(value, patterns)
  if type(value) ~= "number" then
    return nil
  end
  for k, v in pairs(_G) do
    if type(v) == "number" and v == value then
      for _, pattern in ipairs(patterns) do
        if k:match(pattern) then
          return k
        end
      end
    end
  end
  return nil
end

local function nameWithNumber(value, patterns)
  if value == nil then
    return "nil"
  end
  local name = findConstantName(value, patterns)
  if name then
    return string.format("%s (%s)", name, tostring(value))
  end
  return tostring(value)
end

local function trimEventTag(line)
  if type(line) ~= "string" then
    return line
  end
  return (line:gsub("^%[[^%]]+%]%s*", "", 1))
end

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
  local lastLine = nil
  return {
    debug = function(tag, category, value, x, y, options)
      options = options or {}
      local categoryText = nameWithNumber(category, { "^EVT_" })
      local valueText = tostring(value)
      local evtKey = rawget(_G, "EVT_KEY")
      local evtTouch = rawget(_G, "EVT_TOUCH")
      if type(evtKey) == "number" and category == evtKey then
        valueText = nameWithNumber(value, { "^KEY_", "^ROTARY_" })
      elseif type(evtTouch) == "number" and category == evtTouch then
        valueText = nameWithNumber(value, { "^TOUCH_", "^EVT_TOUCH_" })
      end
      local line = string.format(
        "[%s] %s  %s  x=%s y=%s",
        tostring(tag or "event"),
        categoryText,
        valueText,
        tostring(x),
        tostring(y)
      )
      if options.throttleSame and line == lastLine then
        return nil
      end
      lastLine = line
      if not options.returnOnly then
        print(line)
      end
      return line
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
  return {
    throttleSame = true,
    logCapacity = LOG_CAPACITY,
    logBuffer = {},
    logNext = 1,
    logCount = 0,
    toggleTouchArmed = false,
  }
end

local function wakeup(widget)
end

local function getWindowSize()
  if type(lcd) == "table" and type(lcd.getWindowSize) == "function" then
    local ok, w, h = pcall(lcd.getWindowSize)
    if ok and type(w) == "number" and type(h) == "number" then
      return w, h
    end
  end
  return 480, 272
end

local function invalidate()
  if type(lcd) == "table" and type(lcd.invalidate) == "function" then
    pcall(lcd.invalidate)
  end
end

local function toggleThrottle(widget)
  widget.throttleSame = not widget.throttleSame
  print("throttleSame=" .. (widget.throttleSame and "ON" or "OFF"))
end

local function appendLog(widget, line)
  if type(widget) ~= "table" or type(line) ~= "string" then
    return
  end

  if type(widget.logBuffer) ~= "table" then
    widget.logBuffer = {}
  end

  local capacity = widget.logCapacity or LOG_CAPACITY
  local nextIndex = widget.logNext or 1
  widget.logBuffer[nextIndex] = line

  nextIndex = nextIndex + 1
  if nextIndex > capacity then
    nextIndex = 1
  end
  widget.logNext = nextIndex

  local count = (widget.logCount or 0) + 1
  if count > capacity then
    count = capacity
  end
  widget.logCount = count
end

local function getLogLineFromNewest(widget, newestIndex)
  if type(widget) ~= "table" or type(widget.logBuffer) ~= "table" then
    return nil
  end

  local count = widget.logCount or 0
  if newestIndex < 1 or newestIndex > count then
    return nil
  end

  local capacity = widget.logCapacity or LOG_CAPACITY
  local nextIndex = widget.logNext or 1
  local index = nextIndex - newestIndex
  while index <= 0 do
    index = index + capacity
  end
  return widget.logBuffer[index]
end

local function matchesCategory(category, names)
  if type(category) ~= "number" then
    return false
  end
  for _, key in ipairs(names) do
    local code = rawget(_G, key)
    if type(code) == "number" and category == code then
      return true
    end
  end
  return false
end

local function matchesValue(value, names)
  if type(value) ~= "number" then
    return false
  end
  for _, key in ipairs(names) do
    local code = rawget(_G, key)
    if type(code) == "number" and value == code then
      return true
    end
  end
  return false
end

local function resolveTouchPhase(category, value)
  local evtTouch = rawget(_G, "EVT_TOUCH")
  local startCodes = { "TOUCH_START", "EVT_TOUCH_FIRST", "EVT_TOUCH_START", "EVT_TOUCH_TAP" }
  local endCodes = { "TOUCH_END", "EVT_TOUCH_BREAK", "EVT_TOUCH_END" }
  local moveCodes = { "TOUCH_MOVE", "EVT_TOUCH_MOVE", "EVT_TOUCH_SLIDE", "EVT_TOUCH_REPT" }

  if type(evtTouch) == "number" then
    if category ~= evtTouch then
      return nil
    end
    if value == 16640 or matchesValue(value, startCodes) then
      return "start"
    end
    if value == 16641 or matchesValue(value, endCodes) then
      return "end"
    end
    if value == 16642 or matchesValue(value, moveCodes) then
      return "move"
    end
    return nil
  end

  if matchesCategory(category, startCodes) then
    return "start"
  end
  if matchesCategory(category, endCodes) then
    return "end"
  end
  if matchesCategory(category, moveCodes) then
    return "move"
  end
  return nil
end

local function isTouchInToggleRow(x, y, width)
  if type(x) ~= "number" or type(y) ~= "number" then
    return false
  end
  if x < 0 or x > width then
    return false
  end
  return y >= TOGGLE_ROW_Y and y <= (TOGGLE_ROW_Y + TOGGLE_ROW_HEIGHT)
end

local function handleToggle(widget, category, value, x, y)
  local width = getWindowSize()

  local phase = resolveTouchPhase(category, value)
  if not phase then
    return false
  end

  if phase == "start" then
    widget.toggleTouchArmed = isTouchInToggleRow(x, y, width)
    return widget.toggleTouchArmed
  end

  if not widget.toggleTouchArmed then
    return false
  end

  if phase == "move" then
    if not isTouchInToggleRow(x, y, width) then
      widget.toggleTouchArmed = false
    end
    return true
  end

  if phase == "end" then
    local shouldToggle = isTouchInToggleRow(x, y, width)
    widget.toggleTouchArmed = false
    if shouldToggle then
      toggleThrottle(widget)
      invalidate()
    end
    return true
  end

  return false
end

local function paint(widget)
  if type(widget) ~= "table" then
    return
  end
  if type(lcd) ~= "table" or type(lcd.drawText) ~= "function" then
    return
  end

  local _, height = getWindowSize()
  local throttleText = widget.throttleSame and "ON" or "OFF"

  if type(lcd.font) == "function" and FONT_HEADER then
    lcd.font(FONT_HEADER)
  end
  lcd.drawText(8, 8, "Ethos Events")
  lcd.drawText(8, 24, "Recent events:")
  lcd.drawText(8, TOGGLE_ROW_Y, "ThrottleSame: " .. throttleText .. " (tap here)")

  local maxLines = math.floor((height - LOG_START_Y) / LOG_LINE_HEIGHT)
  if maxLines < 1 then
    maxLines = 1
  end

  local lineCount = widget.logCount or 0
  if lineCount == 0 then
    if type(lcd.font) == "function" and FONT_EVENTS then
      lcd.font(FONT_EVENTS)
    end
    lcd.drawText(8, LOG_START_Y, "Waiting for events...")
    return
  end

  if type(lcd.font) == "function" and FONT_EVENTS then
    lcd.font(FONT_EVENTS)
  end
  local visible = math.min(lineCount, maxLines)
  for i = 1, visible do
    local line = getLogLineFromNewest(widget, i)
    if line then
      local y = LOG_START_Y + ((i - 1) * LOG_LINE_HEIGHT)
      lcd.drawText(8, y, line)
    end
  end
end

local function event(widget, category, value, x, y)
  if type(widget) ~= "table" then
    return false
  end

  local ok, line = pcall(events.debug, "ethos_events", category, value, x, y, {
    throttleSame = widget.throttleSame,
    returnOnly = true,
  })
  if ok and type(line) == "string" then
    local formattedLine = trimEventTag(line)
    if type(print) == "function" then
      print(formattedLine)
    end
    appendLog(widget, formattedLine)
    invalidate()
  elseif not ok then
    print("[ethos_events] debug error: " .. tostring(line))
  end

  local touchPhase = resolveTouchPhase(category, value)
  if handleToggle(widget, category, value, x, y) then
    return true
  end

  -- Consume touch events so simulator/system-level fallback actions do not
  -- clear or reset the tool view on unhandled tap gestures.
  if touchPhase then
    return true
  end

  return false
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
