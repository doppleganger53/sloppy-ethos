local WIDGET_NAME = "SensorList"

local FONT_BODY = FONT_XS or FONT_STD
local FONT_HEADER = FONT_STD_BOLD or FONT_STD

local DEFAULT_WINDOW_WIDTH = 480
local DEFAULT_WINDOW_HEIGHT = 272
local SCROLL_ROW_HEIGHT = 16
local HEADER_HEIGHT = 18
local HEADER_TO_ROWS_GAP = 8
local DEBUG_TRACE_ENABLED = false
local MAX_SCROLL_STEPS_PER_EVENT = 4
local MAX_TOUCH_DELTA_PER_EVENT = 128
local LONG_PRESS_SECONDS = 0.8
local LONG_PRESS_MOVE_TOLERANCE = 4
local HEADER_TAP_MOVE_TOLERANCE = 12
local TOUCH_CONTENT_Y_OFFSET = 18
local HEADER_HITBOX_X_PADDING = 12
local HEADER_HITBOX_TOP_PADDING = 8
local HEADER_HITBOX_BOTTOM_PADDING = 10

local SORT_KEY_NAME = "name"
local SORT_KEY_PHYSICAL = "physical"
local SORT_KEY_APPLICATION = "application"

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
  -- Ethos APIs vary by firmware/runtime; optional calls must fail soft.
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
  local rows = math.floor((h - HEADER_HEIGHT - HEADER_TO_ROWS_GAP) / SCROLL_ROW_HEIGHT)
  if rows < 1 then
    return 1
  end
  return rows
end

local function getColumnLayout(windowWidth)
  local width = type(windowWidth) == "number" and windowWidth or DEFAULT_WINDOW_WIDTH
  if width < 1 then
    width = 1
  end
  local colPhysX = math.floor(width * 0.56)
  local colAppX = math.floor(width * 0.76)
  return {
    { key = SORT_KEY_NAME, title = "Name", x = 0 },
    { key = SORT_KEY_PHYSICAL, title = "Physical ID", x = colPhysX },
    { key = SORT_KEY_APPLICATION, title = "Application ID", x = colAppX },
  }
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

    -- Dynamic category discovery keeps this widget portable across Ethos builds
    -- where CATEGORY_* constants can differ.
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

local function compareDefaultSensors(a, b)
  if a.physical ~= b.physical then
    return a.physical < b.physical
  end
  if a.application ~= b.application then
    return a.application < b.application
  end
  return a.name < b.name
end

local function compareSortedByKey(a, b, sortKey, descending)
  if sortKey == SORT_KEY_NAME then
    if a.name ~= b.name then
      if descending then
        return a.name > b.name
      end
      return a.name < b.name
    end
    return compareDefaultSensors(a, b)
  end
  if sortKey == SORT_KEY_PHYSICAL then
    if a.physical ~= b.physical then
      if descending then
        return a.physical > b.physical
      end
      return a.physical < b.physical
    end
    if a.application ~= b.application then
      return a.application < b.application
    end
    return a.name < b.name
  end
  if sortKey == SORT_KEY_APPLICATION then
    if a.application ~= b.application then
      if descending then
        return a.application > b.application
      end
      return a.application < b.application
    end
    if a.physical ~= b.physical then
      return a.physical < b.physical
    end
    return a.name < b.name
  end
  return compareDefaultSensors(a, b)
end

local function sortSensorsInPlace(sensors, sortKey, descending)
  if type(sensors) ~= "table" or #sensors < 2 then
    return
  end
  table.sort(sensors, function(a, b)
    return compareSortedByKey(a, b, sortKey, descending)
  end)
end

local function cloneSensors(sensors)
  local out = {}
  for idx, sensor in ipairs(sensors) do
    out[idx] = sensor
  end
  return out
end

local function buildDisplaySensors(widget, normalized)
  local display = cloneSensors(normalized)
  if type(widget) == "table" and widget.sortKey then
    sortSensorsInPlace(display, widget.sortKey, widget.sortDescending and true or false)
  end
  return display
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

  sortSensorsInPlace(normalized, nil, false)
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
  local refreshStart = os.clock()
  local raw, debug = getSensorList(widget, allowDeepScan)
  local afterSource = os.clock()
  widget.lastRawCount = #raw
  widget.lastDebug = debug or widget.lastDebug
  local normalized = normalizeSensors(raw)
  local signature = buildSignature(normalized)
  local signatureChanged = signature ~= widget.lastSignature
  local strategy = debug and debug.strategy or "unknown"

  widget.debugRefreshCount = (widget.debugRefreshCount or 0) + 1
  if strategy == "deep-scan" then
    widget.debugDeepScanCount = (widget.debugDeepScanCount or 0) + 1
  elseif strategy == "cached-category" then
    widget.debugCachedScanCount = (widget.debugCachedScanCount or 0) + 1
  elseif strategy == "deferred-deep-scan" then
    widget.debugDeferredScanCount = (widget.debugDeferredScanCount or 0) + 1
  end

  if signatureChanged then
    widget.sensors = buildDisplaySensors(widget, normalized)
    widget.groups = buildPhysicalGroups(normalized)
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

  if DEBUG_TRACE_ENABLED and type(print) == "function" then
    local totalMs = math.floor((os.clock() - refreshStart) * 1000 + 0.5)
    local sourceMs = math.floor((afterSource - refreshStart) * 1000 + 0.5)
    print(
      string.format(
        "SLDBG refresh=%d strategy=%s allowDeep=%d raw=%d norm=%d sigChanged=%d cats=%d scanned=%d sourceMs=%d totalMs=%d offset=%d deep=%d cached=%d deferred=%d",
        widget.debugRefreshCount or 0,
        tostring(strategy),
        allowDeepScan and 1 or 0,
        #raw,
        #normalized,
        signatureChanged and 1 or 0,
        debug and debug.categoryCount or 0,
        debug and debug.scannedCategories or 0,
        sourceMs,
        totalMs,
        widget.scrollOffset or 0,
        widget.debugDeepScanCount or 0,
        widget.debugCachedScanCount or 0,
        widget.debugDeferredScanCount or 0
      )
    )
  end
end

local function triggerManualRefresh(widget)
  if type(widget) ~= "table" then
    return false
  end
  refreshSensors(widget, true)
  widget.needsInvalidate = true
  return true
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

local function headerTitle(widget, column)
  if type(column) ~= "table" then
    return ""
  end
  if type(widget) ~= "table" or widget.sortKey ~= column.key then
    return column.title
  end
  local arrow = widget.sortDescending and " v" or " ^"
  return column.title .. arrow
end

local function headerSortKeyAtPosition(x, y)
  if type(x) ~= "number" or type(y) ~= "number" then
    return nil
  end
  local contentY = y - TOUCH_CONTENT_Y_OFFSET
  if contentY < -HEADER_HITBOX_TOP_PADDING or contentY >= (HEADER_HEIGHT + HEADER_HITBOX_BOTTOM_PADDING) then
    return nil
  end

  local w, _ = getWindowSizeSafe()
  if x < 0 or x >= w then
    return nil
  end
  local columns = getColumnLayout(w)

  local hitboxes = {
    {
      key = columns[1].key,
      left = math.max(0, columns[1].x - HEADER_HITBOX_X_PADDING),
      right = math.min(w, columns[2].x + HEADER_HITBOX_X_PADDING),
      center = (columns[1].x + columns[2].x) / 2,
    },
    {
      key = columns[2].key,
      left = math.max(0, columns[2].x - HEADER_HITBOX_X_PADDING),
      right = math.min(w, columns[3].x + HEADER_HITBOX_X_PADDING),
      center = (columns[2].x + columns[3].x) / 2,
    },
    {
      key = columns[3].key,
      left = math.max(0, columns[3].x - HEADER_HITBOX_X_PADDING),
      right = w,
      center = (columns[3].x + w) / 2,
    },
  }

  local bestKey = nil
  local bestDistance = nil
  for _, hitbox in ipairs(hitboxes) do
    if x >= hitbox.left and x < hitbox.right then
      local distance = math.abs(x - hitbox.center)
      if bestDistance == nil or distance < bestDistance then
        bestDistance = distance
        bestKey = hitbox.key
      end
    end
  end
  return bestKey
end

local function clearHeaderTouch(widget)
  widget.headerTouchKey = nil
  widget.headerTouchStartX = nil
  widget.headerTouchStartY = nil
end

local function applyHeaderSort(widget, sortKey)
  if type(widget) ~= "table" then
    return false
  end
  if sortKey ~= SORT_KEY_NAME and sortKey ~= SORT_KEY_PHYSICAL and sortKey ~= SORT_KEY_APPLICATION then
    return false
  end
  if widget.sortKey == sortKey then
    widget.sortDescending = not widget.sortDescending
  else
    widget.sortKey = sortKey
    widget.sortDescending = false
  end
  sortSensorsInPlace(widget.sensors, widget.sortKey, widget.sortDescending and true or false)
  widget.needsInvalidate = true
  return true
end

local function handleHeaderTap(widget, phase, x, y)
  if type(widget) ~= "table" then
    return false
  end

  if phase == "start" then
    local sortKey = headerSortKeyAtPosition(x, y)
    if not sortKey then
      clearHeaderTouch(widget)
      return false
    end
    widget.headerTouchKey = sortKey
    widget.headerTouchStartX = x
    widget.headerTouchStartY = y
    return true
  end

  if not widget.headerTouchKey then
    return false
  end

  if phase == "move" then
    if type(x) ~= "number" or type(y) ~= "number" then
      clearHeaderTouch(widget)
      return true
    end
    local movedX = math.abs(x - (widget.headerTouchStartX or x))
    local movedY = math.abs(y - (widget.headerTouchStartY or y))
    local contentY = y - TOUCH_CONTENT_Y_OFFSET
    if
      movedX > HEADER_TAP_MOVE_TOLERANCE
      or movedY > HEADER_TAP_MOVE_TOLERANCE
      or contentY < -HEADER_HITBOX_TOP_PADDING
      or contentY >= (HEADER_HEIGHT + HEADER_HITBOX_BOTTOM_PADDING)
    then
      clearHeaderTouch(widget)
    end
    return true
  end

  if phase == "end" then
    local sortKey = widget.headerTouchKey
    local movedX = type(x) == "number" and math.abs(x - (widget.headerTouchStartX or x)) or (HEADER_TAP_MOVE_TOLERANCE + 1)
    local movedY = type(y) == "number" and math.abs(y - (widget.headerTouchStartY or y)) or (HEADER_TAP_MOVE_TOLERANCE + 1)
    local contentY = type(y) == "number" and (y - TOUCH_CONTENT_Y_OFFSET) or nil
    local isHeaderEnd = type(contentY) == "number"
      and contentY >= -HEADER_HITBOX_TOP_PADDING
      and contentY < (HEADER_HEIGHT + HEADER_HITBOX_BOTTOM_PADDING)
    clearHeaderTouch(widget)
    if movedX <= HEADER_TAP_MOVE_TOLERANCE and movedY <= HEADER_TAP_MOVE_TOLERANCE and isHeaderEnd then
      return applyHeaderSort(widget, sortKey)
    end
    return true
  end

  return false
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
  local columns = getColumnLayout(w)

  -- Paint our own background so Ethos focus highlight does not bleed through.
  lcd.color(COLOR_BLACK or 0)
  if type(lcd.drawFilledRectangle) == "function" then
    lcd.drawFilledRectangle(0, 0, w, h)
  end

  local rowsY = headerHeight + HEADER_TO_ROWS_GAP
  local visibleRows = getVisibleRows()

  for _, column in ipairs(columns) do
    drawText(column.x, 0, headerTitle(widget, column), FONT_HEADER)
  end

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
    drawText(columns[1].x, y, shortenText(sensor.name, 20), FONT_BODY, color)
    drawText(columns[2].x, y, sensor.physicalText, FONT_BODY, color)
    drawText(columns[3].x, y, sensor.applicationText, FONT_BODY, color)
  end

  return visibleRows
end

-- Ethos widget API callback: constructs per-instance widget state.
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
    lastInvalidate = 0,
    needsInvalidate = true,
    touchActive = false,
    touchLastY = nil,
    touchAccumY = 0,
    touchStartY = nil,
    touchStartClock = nil,
    touchHoldTriggered = false,
    headerTouchKey = nil,
    headerTouchStartX = nil,
    headerTouchStartY = nil,
    sortKey = nil,
    sortDescending = false,
    debugRefreshCount = 0,
    debugDeepScanCount = 0,
    debugCachedScanCount = 0,
    debugDeferredScanCount = 0,
  }
  refreshSensors(widget, true)
  return widget
end

-- Ethos widget API callback: draws current widget frame.
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

local function matchesCode(value, names)
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
  local startCodes = { "EVT_TOUCH_FIRST", "EVT_TOUCH_START", "EVT_TOUCH_TAP" }
  local endCodes = { "EVT_TOUCH_BREAK", "EVT_TOUCH_END" }
  local moveCodes = { "EVT_TOUCH_MOVE", "EVT_TOUCH_SLIDE", "EVT_TOUCH_REPT" }

  if type(evtTouch) == "number" then
    if category ~= evtTouch then
      return nil
    end
    if value == 16640 or matchesCode(value, startCodes) then
      return "start"
    end
    if value == 16641 or matchesCode(value, endCodes) then
      return "end"
    end
    if value == 16642 or matchesCode(value, moveCodes) then
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

local function handleTouchScroll(widget, phase, x, y, category, value)
  if type(widget) ~= "table" then
    return false
  end
  if type(x) ~= "number" or type(y) ~= "number" then
    return false
  end

  if phase == "end" then
    local startClock = widget.touchStartClock
    local heldFor = startClock and (os.clock() - startClock) or 0
    local startY = widget.touchStartY or y
    local endY = widget.touchLastY or y
    if
      not widget.touchHoldTriggered
      and heldFor >= LONG_PRESS_SECONDS
      and math.abs(endY - startY) <= LONG_PRESS_MOVE_TOLERANCE
    then
      widget.touchHoldTriggered = true
      triggerManualRefresh(widget)
    end
    widget.touchActive = false
    widget.touchLastY = nil
    widget.touchAccumY = 0
    widget.touchStartY = nil
    widget.touchStartClock = nil
    widget.touchHoldTriggered = false
    return true
  end

  if phase == "start" then
    widget.touchActive = true
    widget.touchLastY = y
    widget.touchAccumY = 0
    widget.touchStartY = y
    widget.touchStartClock = os.clock()
    widget.touchHoldTriggered = false
    return true
  end

  if not widget.touchActive then
    return false
  end

  local rawDy = y - (widget.touchLastY or y)
  local dy = rawDy
  if dy > MAX_TOUCH_DELTA_PER_EVENT then
    dy = MAX_TOUCH_DELTA_PER_EVENT
  elseif dy < -MAX_TOUCH_DELTA_PER_EVENT then
    dy = -MAX_TOUCH_DELTA_PER_EVENT
  end

  widget.touchLastY = y
  widget.touchAccumY = (widget.touchAccumY or 0) + dy

  local moved = false
  local steps = 0
  while math.abs(widget.touchAccumY) >= SCROLL_ROW_HEIGHT do
    steps = steps + 1
    if steps > MAX_SCROLL_STEPS_PER_EVENT then
      widget.touchAccumY = widget.touchAccumY < 0 and -(SCROLL_ROW_HEIGHT - 1) or (SCROLL_ROW_HEIGHT - 1)
      if DEBUG_TRACE_ENABLED and type(print) == "function" then
        print(
          string.format(
            "SLDBG event-cap category=%s value=%s x=%s y=%s rawDy=%d clampedDy=%d",
            tostring(category),
            tostring(value),
            tostring(x),
            tostring(y),
            rawDy,
            dy
          )
        )
      end
      break
    end

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

  if DEBUG_TRACE_ENABLED and type(print) == "function" and math.abs(rawDy) >= (MAX_TOUCH_DELTA_PER_EVENT * 2) then
    print(
      string.format(
        "SLDBG event-dy category=%s value=%s x=%s y=%s rawDy=%d clampedDy=%d",
        tostring(category),
        tostring(value),
        tostring(x),
        tostring(y),
        rawDy,
        dy
      )
    )
  end

  return true
end

-- Ethos widget API callback: periodic runtime hook used for redraw scheduling.
local function wakeup(widget, event)
  if type(widget) ~= "table" then
    return
  end
  local now = os.clock()

  -- No periodic refresh: we refresh on create() and explicit long-press only.
  if widget.needsInvalidate and lcd.isVisible() and now - widget.lastInvalidate >= 0.05 then
    lcd.invalidate()
    widget.lastInvalidate = now
    widget.needsInvalidate = false
  end
end

-- Ethos widget API callback: input/event dispatcher from Ethos runtime.
local function event(widget, category, value, x, y)
  if type(widget) ~= "table" then
    return false
  end

  -- Long-press is an explicit user request to rescan all sensors.
  local isLongPress = matchesCode(value, { "EVT_TOUCH_LONG" }) or matchesCategory(category, { "EVT_TOUCH_LONG" })
  if isLongPress then
    return triggerManualRefresh(widget)
  end

  local phase = resolveTouchPhase(category, value)
  if not phase then
    return false
  end

  local headerConsumed = handleHeaderTap(widget, phase, x, y)
  if headerConsumed then
    return true
  end

  return handleTouchScroll(widget, phase, x, y, category, value)
end

local function name()
  return WIDGET_NAME
end

-- Ethos widget API registration entrypoint.
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

local function testExports()
  return {
    toInt = toInt,
    formatHex = formatHex,
    normalizeSensors = normalizeSensors,
    buildPhysicalGroups = buildPhysicalGroups,
    buildSignature = buildSignature,
    applyScroll = applyScroll,
    clampOffset = clampOffset,
    getSensorList = getSensorList,
    event = event,
    resolveTouchPhase = resolveTouchPhase,
  }
end

local module = { init = init }
if rawget(_G, "__SENSORLIST_TEST__") then
  module._test = testExports()
end

return module
