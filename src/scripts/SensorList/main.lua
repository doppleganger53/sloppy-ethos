local WIDGET_NAME = "SensorList"

local FONT_BODY = FONT_XS or FONT_STD
local FONT_HEADER = FONT_STD_BOLD or FONT_STD

local DEFAULT_WINDOW_WIDTH = 480
local DEFAULT_WINDOW_HEIGHT = 272
local SCROLL_ROW_HEIGHT = 16
local HEADER_HEIGHT = 18

local COLOR_PALETTE = {
  { 255, 235, 59 },
  { 129, 212, 250 },
  { 255, 171, 145 },
  { 179, 157, 219 },
  { 165, 214, 167 },
  { 244, 143, 177 },
}

local function safeCall(fn, ...)
  if type(fn) ~= "function" then
    return nil
  end
  local ok, result = pcall(fn, ...)
  if ok then
    return result
  end
  return nil
end

local function toInt(value)
  if type(value) == "number" then
    return math.floor(value)
  end
  if type(value) ~= "string" then
    return nil
  end
  local trimmed = value:gsub("^%s+", ""):gsub("%s+$", "")
  if trimmed == "" then
    return nil
  end
  local decimal = tonumber(trimmed)
  if decimal then
    return math.floor(decimal)
  end
  local noPrefix = trimmed:gsub("^0x", ""):gsub("^0X", "")
  local asHex = tonumber(noPrefix, 16)
  if asHex then
    return math.floor(asHex)
  end
  return nil
end

local function formatHex(value, width)
  local numeric = toInt(value)
  if numeric == nil then
    return "--"
  end
  if numeric < 0 then
    numeric = 0
  end
  return string.format("%0" .. tostring(width) .. "X", numeric)
end

local function colorFromRgb(rgb)
  if type(lcd.RGB) ~= "function" then
    return nil
  end
  return safeCall(lcd.RGB, rgb[1], rgb[2], rgb[3])
end

local function getWindowSizeSafe()
  if type(lcd.getWindowSize) == "function" then
    local ok, w, h = pcall(lcd.getWindowSize)
    if ok and type(w) == "number" and w > 0 and type(h) == "number" and h > 0 then
      return w, h
    end
  end
  return DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT
end

local function calculateVisibleRows()
  local _, h = getWindowSizeSafe()
  local rows = math.floor((h - HEADER_HEIGHT) / SCROLL_ROW_HEIGHT)
  if rows < 1 then
    return 1
  end
  return rows
end

local function shortenText(text, maxChars)
  if #text <= maxChars then
    return text
  end
  if maxChars <= 3 then
    return text:sub(1, maxChars)
  end
  return text:sub(1, maxChars - 3) .. "..."
end

local function sourceName(source)
  if source and type(source.name) == "function" then
    return safeCall(source.name, source)
  end
  if type(source) == "table" then
    return source.name
  end
  return nil
end

local function collectFromCategory(category, maxMember)
  local sensors = {}
  local highestMember = -1
  local emptyRun = 0

  for member = 0, maxMember do
    local source = safeCall(system.getSource, { category = category, member = member })
    local name = sourceName(source)
    if source and type(name) == "string" and name ~= "" and name ~= "---" then
      sensors[#sensors + 1] = source
      highestMember = member
      emptyRun = 0
    else
      emptyRun = emptyRun + 1
      if member > 32 and emptyRun >= 24 then
        break
      end
    end
  end

  return sensors, highestMember
end

local function getSensorList(widget, allowDeepScan)
  local debug = {
    hasSystemGetSensors = type(system.getSensors) == "function",
    hasModelGetSensors = model and type(model.getSensors) == "function" or false,
    hasSystemGetSource = type(system.getSource) == "function",
    categoryCount = 0,
    scannedCategories = 0,
  }

  if type(system.getSensors) == "function" then
    local sensors = safeCall(system.getSensors)
    if type(sensors) == "table" and #sensors > 0 then
      debug.strategy = "system.getSensors"
      return sensors, debug
    end
  end
  if model and type(model.getSensors) == "function" then
    local sensors = safeCall(model.getSensors)
    if type(sensors) == "table" and #sensors > 0 then
      debug.strategy = "model.getSensors"
      return sensors, debug
    end
  end

  if type(system.getSource) == "function" then
    if widget and widget.sourceCategory ~= nil then
      local sensors, highestMember = collectFromCategory(widget.sourceCategory, widget.sourceMaxMember or 255)
      if #sensors > 0 then
        widget.sourceMaxMember = math.max(widget.sourceMaxMember or 0, highestMember + 8)
        debug.strategy = "cached-category"
        debug.scannedCategories = 1
        return sensors, debug
      end
    end

    if not allowDeepScan then
      debug.strategy = "deferred-deep-scan"
      return {}, debug
    end

    local categories = {}
    for key, value in pairs(_G) do
      if type(key) == "string" and key:match("^CATEGORY_") and type(value) == "number" then
        categories[#categories + 1] = { key = key, value = value }
      end
    end
    table.sort(categories, function(a, b)
      return a.key < b.key
    end)
    debug.categoryCount = #categories

    local preferred = {}
    for _, item in ipairs(categories) do
      if item.key:find("TELEMETRY", 1, true) or item.key:find("SENSOR", 1, true) then
        preferred[#preferred + 1] = item
      end
    end
    if #preferred == 0 then
      preferred = categories
    end

    local bestSensors = {}
    local bestCategory = nil
    local bestHighestMember = -1

    for _, item in ipairs(preferred) do
      debug.scannedCategories = debug.scannedCategories + 1
      local sensors, highestMember = collectFromCategory(item.value, 255)
      if #sensors > #bestSensors then
        bestSensors = sensors
        bestCategory = item.value
        bestHighestMember = highestMember
      end
    end

    if widget and bestCategory ~= nil then
      widget.sourceCategory = bestCategory
      widget.sourceMaxMember = math.max(64, bestHighestMember + 8)
    end

    debug.strategy = "deep-scan"
    return bestSensors, debug
  end

  debug.strategy = "no-source-api"
  return {}, debug
end

local function readCandidate(sensor, keys)
  for _, key in ipairs(keys) do
    if type(sensor) == "table" and sensor[key] ~= nil then
      return sensor[key]
    end
    local method = sensor and sensor[key]
    if type(method) == "function" then
      local value = safeCall(method, sensor)
      if value ~= nil then
        return value
      end
    end
  end
  return nil
end

local function normalizeSensors(rawSensors)
  local normalized = {}
  for idx, sensor in ipairs(rawSensors) do
    local name = readCandidate(sensor, { "label", "name", "text", "title", "stringValue" }) or ("Sensor " .. tostring(idx))

    local physical = toInt(readCandidate(sensor, { "physicalId", "physicalID", "physId", "id1", "id" }))
    local application = toInt(readCandidate(sensor, { "applicationId", "appId", "param", "subId", "id2", "instance" }))

    normalized[#normalized + 1] = {
      name = tostring(name),
      physical = physical or 65535,
      application = application or 65535,
      physicalText = formatHex(physical, 2),
      applicationText = formatHex(application, 4),
    }
  end

  table.sort(normalized, function(a, b)
    if a.physical ~= b.physical then
      return a.physical < b.physical
    end
    if a.application ~= b.application then
      return a.application < b.application
    end
    return a.name < b.name
  end)
  return normalized
end

local function buildPhysicalGroups(sensors)
  local counts = {}
  for _, sensor in ipairs(sensors) do
    if sensor.physical ~= 65535 then
      counts[sensor.physical] = (counts[sensor.physical] or 0) + 1
    end
  end
  local groups = {}
  local nextIndex = 1
  for _, sensor in ipairs(sensors) do
    if sensor.physical ~= 65535 and counts[sensor.physical] and counts[sensor.physical] > 1 and not groups[sensor.physical] then
      groups[sensor.physical] = nextIndex
      nextIndex = nextIndex + 1
    end
  end
  return groups
end

local function groupColor(physical, groups, cache)
  local groupIndex = groups[physical]
  if not groupIndex then
    return nil
  end
  if cache[groupIndex] then
    return cache[groupIndex]
  end
  local paletteIndex = ((groupIndex - 1) % #COLOR_PALETTE) + 1
  cache[groupIndex] = colorFromRgb(COLOR_PALETTE[paletteIndex])
  return cache[groupIndex]
end

local function buildSignature(sensors)
  local out = {}
  for _, sensor in ipairs(sensors) do
    out[#out + 1] = sensor.name .. "|" .. sensor.physicalText .. "|" .. sensor.applicationText
  end
  return table.concat(out, ";")
end

local function refreshSensors(widget, allowDeepScan)
  if type(widget) ~= "table" then
    return
  end
  local raw, debug = getSensorList(widget, allowDeepScan)
  widget.lastRawCount = #raw
  widget.lastDebug = debug or widget.lastDebug
  local normalized = normalizeSensors(raw)
  local signature = buildSignature(normalized)

  if signature ~= widget.lastSignature then
    widget.sensors = normalized
    widget.groups = buildPhysicalGroups(widget.sensors)
    widget.colorCache = {}
    widget.lastSignature = signature
    widget.needsInvalidate = true
    local visibleRows = calculateVisibleRows()
    local maxOffset = math.max(0, #widget.sensors - visibleRows)
    if widget.scrollOffset < 0 then
      widget.scrollOffset = 0
    elseif widget.scrollOffset > maxOffset then
      widget.scrollOffset = maxOffset
    end
  end
end

local function drawText(x, y, text, font, color)
  if color then
    lcd.color(color)
  else
    lcd.color(COLOR_WHITE)
  end
  lcd.font(font)
  lcd.drawText(x, y, text)
end

local function getVisibleRows()
  return calculateVisibleRows()
end

local function getMaxOffset(widget)
  return math.max(0, #widget.sensors - getVisibleRows())
end

local function clampOffset(widget)
  local maxOffset = getMaxOffset(widget)
  if widget.scrollOffset < 0 then
    widget.scrollOffset = 0
  elseif widget.scrollOffset > maxOffset then
    widget.scrollOffset = maxOffset
  end
end

local function drawSensorRows(widget)
  local w, _ = getWindowSizeSafe()
  local _, h = getWindowSizeSafe()
  local rowHeight = SCROLL_ROW_HEIGHT
  local headerHeight = HEADER_HEIGHT

  -- Paint our own background so Ethos focus highlight does not bleed through.
  lcd.color(COLOR_BLACK or 0)
  if type(lcd.drawFilledRectangle) == "function" then
    lcd.drawFilledRectangle(0, 0, w, h)
  end

  local colNameX = 0
  local colPhysX = math.floor(w * 0.56)
  local colAppX = math.floor(w * 0.76)
  local rowsY = headerHeight
  local visibleRows = getVisibleRows()

  drawText(colNameX, 0, "Name", FONT_HEADER)
  drawText(colPhysX, 0, "Physical ID", FONT_HEADER)
  drawText(colAppX, 0, "Application ID", FONT_HEADER)

  if #widget.sensors == 0 then
    drawText(0, 20, "No sensors configured.", FONT_BODY)
    drawText(0, 36, "Raw sensors found: " .. tostring(widget.lastRawCount or 0), FONT_BODY)
    if widget.lastDebug then
      local d = widget.lastDebug
      drawText(
        0,
        52,
        "API S:" .. tostring(d.hasSystemGetSensors and 1 or 0)
          .. " M:" .. tostring(d.hasModelGetSensors and 1 or 0)
          .. " Src:" .. tostring(d.hasSystemGetSource and 1 or 0),
        FONT_BODY
      )
      drawText(
        0,
        68,
        "Categories: " .. tostring(d.categoryCount or 0) .. " scanned: " .. tostring(d.scannedCategories or 0),
        FONT_BODY
      )
    end
    return visibleRows
  end

  local start = widget.scrollOffset + 1
  for row = 0, visibleRows - 1 do
    local sensor = widget.sensors[start + row]
    if not sensor then
      break
    end
    local y = rowsY + row * rowHeight
    local color = groupColor(sensor.physical, widget.groups, widget.colorCache)
    drawText(colNameX, y, shortenText(sensor.name, 20), FONT_BODY, color)
    drawText(colPhysX, y, sensor.physicalText, FONT_BODY, color)
    drawText(colAppX, y, sensor.applicationText, FONT_BODY, color)
  end

  return visibleRows
end

local function create()
  local widget = {
    sensors = {},
    groups = {},
    colorCache = {},
    scrollOffset = 0,
    lastSignature = "",
    lastRawCount = 0,
    lastDebug = {},
    sourceCategory = nil,
    sourceMaxMember = 255,
    lastPoll = 0,
    lastDeepScan = 0,
    lastInvalidate = 0,
    needsInvalidate = true,
    touchActive = false,
    touchLastY = nil,
    touchAccumY = 0,
  }
  refreshSensors(widget, true)
  return widget
end

local function paint(widget)
  if type(widget) ~= "table" then
    return
  end
  drawSensorRows(widget)
end

local function applyScroll(widget, delta)
  if delta == 0 then
    return false
  end
  local previous = widget.scrollOffset
  widget.scrollOffset = widget.scrollOffset + delta
  clampOffset(widget)
  return widget.scrollOffset ~= previous
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

local function handleTouchScroll(widget, category, x, y)
  if type(widget) ~= "table" then
    return false
  end
  if type(x) ~= "number" or type(y) ~= "number" then
    return false
  end

  local touchStart = matchesCategory(category, { "EVT_TOUCH_FIRST", "EVT_TOUCH_START", "EVT_TOUCH_TAP" })
  local touchEnd = matchesCategory(category, { "EVT_TOUCH_BREAK", "EVT_TOUCH_END", "EVT_TOUCH_LONG" })

  if touchEnd then
    widget.touchActive = false
    widget.touchLastY = nil
    widget.touchAccumY = 0
    return true
  end

  if touchStart or not widget.touchActive then
    widget.touchActive = true
    widget.touchLastY = y
    widget.touchAccumY = 0
    return true
  end

  local dy = y - (widget.touchLastY or y)
  widget.touchLastY = y
  widget.touchAccumY = (widget.touchAccumY or 0) + dy

  local moved = false
  while math.abs(widget.touchAccumY) >= SCROLL_ROW_HEIGHT do
    local delta = widget.touchAccumY < 0 and 1 or -1
    if applyScroll(widget, delta) then
      moved = true
    end
    widget.touchAccumY = widget.touchAccumY - (delta == 1 and -SCROLL_ROW_HEIGHT or SCROLL_ROW_HEIGHT)
  end

  if moved then
    widget.needsInvalidate = true
    return true
  end

  return true
end

local function wakeup(widget, event)
  if type(widget) ~= "table" then
    return
  end
  local now = os.clock()

  if now - widget.lastPoll >= 5.0 then
    local allowDeepScan = (now - widget.lastDeepScan) >= 30.0
    refreshSensors(widget, allowDeepScan)
    widget.lastPoll = now
    if allowDeepScan then
      widget.lastDeepScan = now
    end
  end

  if widget.needsInvalidate and lcd.isVisible() and now - widget.lastInvalidate >= 0.05 then
    lcd.invalidate()
    widget.lastInvalidate = now
    widget.needsInvalidate = false
  end
end

local function event(widget, category, value, x, y)
  if type(widget) ~= "table" then
    return false
  end

  local evtTouch = rawget(_G, "EVT_TOUCH")

  if type(evtTouch) == "number" then
    if category ~= evtTouch then
      return false
    end
    return handleTouchScroll(widget, category, x, y)
  end

  -- Fallback for firmware variants that don't expose EVT_TOUCH as a category:
  -- treat event callbacks with valid pointer coordinates as touch input.
  if type(x) == "number" and type(y) == "number" then
    return handleTouchScroll(widget, category, x, y)
  end

  return false
end

local function name()
  return WIDGET_NAME
end

local function init()
  system.registerWidget({
    key = "slist",
    name = name,
    create = create,
    paint = paint,
    wakeup = wakeup,
    event = event,
    persistent = false,
  })
end

return { init = init }
