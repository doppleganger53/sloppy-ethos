local WIDGET_NAME = "SensorList"

local FONT_BODY = _G.FONT_XS or _G.SMLSIZE or 0
local FONT_HEADER = _G.FONT_STD_BOLD or _G.BOLDSIZE or FONT_BODY

local COLOR_PALETTE = {
  { 255, 235, 59 },  -- yellow
  { 129, 212, 250 }, -- cyan
  { 255, 171, 145 }, -- orange
  { 179, 157, 219 }, -- violet
  { 165, 214, 167 }, -- green
  { 244, 143, 177 }, -- pink
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

local function nowTicks()
  local ticks = safeCall(_G.getTime)
  if type(ticks) == "number" then
    return ticks
  end

  if _G.system and type(_G.system.getTimeCounter) == "function" then
    ticks = safeCall(_G.system.getTimeCounter)
    if type(ticks) == "number" then
      return ticks
    end
  end

  return math.floor(os.clock() * 100)
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
  if not (_G.lcd and type(_G.lcd.RGB) == "function") then
    return nil
  end

  return safeCall(_G.lcd.RGB, rgb[1], rgb[2], rgb[3])
end

local function drawText(x, y, text, flags, color)
  if not (_G.lcd and type(_G.lcd.drawText) == "function") then
    return
  end

  local drawFlags = flags or 0
  if color and _G.CUSTOM_COLOR and type(_G.CUSTOM_COLOR) == "number" and type(_G.lcd.setColor) == "function" then
    safeCall(_G.lcd.setColor, _G.CUSTOM_COLOR, color)
    drawFlags = drawFlags + _G.CUSTOM_COLOR
  end

  safeCall(_G.lcd.drawText, x, y, text, drawFlags)
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

local function getSensorList()
  if _G.system and type(_G.system.getSensors) == "function" then
    local sensors = safeCall(_G.system.getSensors)
    if type(sensors) == "table" then
      return sensors
    end
  end

  if _G.model and type(_G.model.getSensors) == "function" then
    local sensors = safeCall(_G.model.getSensors)
    if type(sensors) == "table" then
      return sensors
    end
  end

  return {}
end

local function normalizeSensors(rawSensors)
  local normalized = {}

  for idx, sensor in ipairs(rawSensors) do
    local name = sensor.label or sensor.name or sensor.text or ("Sensor " .. tostring(idx))

    local physical = toInt(sensor.physicalId) or toInt(sensor.physicalID) or toInt(sensor.id) or toInt(sensor.physId)
    local application = toInt(sensor.applicationId) or toInt(sensor.appId) or toInt(sensor.param) or toInt(sensor.subId)

    if physical ~= nil or application ~= nil then
      normalized[#normalized + 1] = {
        name = tostring(name),
        physical = physical or -1,
        application = application or -1,
        physicalText = formatHex(physical, 2),
        applicationText = formatHex(application, 4),
      }
    end
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
    counts[sensor.physical] = (counts[sensor.physical] or 0) + 1
  end

  local groups = {}
  local nextIndex = 1
  for _, sensor in ipairs(sensors) do
    if counts[sensor.physical] and counts[sensor.physical] > 1 and not groups[sensor.physical] then
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

local function refreshSensors(widget)
  widget.sensors = normalizeSensors(getSensorList())
  widget.groups = buildPhysicalGroups(widget.sensors)
  widget.colorCache = {}
  widget.lastPoll = nowTicks()

  if widget.scrollOffset > #widget.sensors - 1 then
    widget.scrollOffset = 0
  end
end

local function drawHeader(zone, colNameX, colPhysX, colAppX)
  drawText(colNameX, zone.y, "Name", FONT_HEADER)
  drawText(colPhysX, zone.y, "Physical ID", FONT_HEADER)
  drawText(colAppX, zone.y, "Application ID", FONT_HEADER)
end

local function drawEmptyState(widget)
  local zone = widget.zone
  drawText(zone.x, zone.y + 18, "No sensors configured.", FONT_BODY)
end

local function drawSensorRows(widget)
  local zone = widget.zone
  local rowHeight = 16
  local headerHeight = 18

  local colNameX = zone.x
  local colPhysX = zone.x + math.floor(zone.w * 0.56)
  local colAppX = zone.x + math.floor(zone.w * 0.76)
  local rowsY = zone.y + headerHeight
  local visibleRows = math.max(1, math.floor((zone.h - headerHeight) / rowHeight))

  drawHeader(zone, colNameX, colPhysX, colAppX)

  if #widget.sensors == 0 then
    drawEmptyState(widget)
    return
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
end

local function stepScroll(widget)
  local zone = widget.zone
  local headerHeight = 18
  local rowHeight = 16
  local visibleRows = math.max(1, math.floor((zone.h - headerHeight) / rowHeight))
  local totalRows = #widget.sensors

  if totalRows <= visibleRows then
    widget.scrollOffset = 0
    return
  end

  local maxOffset = totalRows - visibleRows
  if widget.scrollOffset >= maxOffset then
    widget.scrollOffset = 0
  else
    widget.scrollOffset = widget.scrollOffset + 1
  end
end

local function maybeUpdate(widget)
  local ticks = nowTicks()
  if ticks - widget.lastPoll >= 100 then
    refreshSensors(widget)
  end

  if ticks - widget.lastScroll >= 80 then
    stepScroll(widget)
    widget.lastScroll = ticks
  end
end

local function create(zone, options)
  local widget = {
    zone = zone,
    options = options,
    sensors = {},
    groups = {},
    colorCache = {},
    scrollOffset = 0,
    lastPoll = 0,
    lastScroll = 0,
  }

  refreshSensors(widget)
  widget.lastScroll = nowTicks()
  return widget
end

local function update(widget, options)
  widget.options = options
end

local function background(widget)
  maybeUpdate(widget)
end

local function refresh(widget)
  maybeUpdate(widget)
  drawSensorRows(widget)
end

return {
  name = WIDGET_NAME,
  create = create,
  update = update,
  background = background,
  refresh = refresh,
}
