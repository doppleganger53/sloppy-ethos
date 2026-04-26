local WIDGET_NAME = "BoundryMap"
local bitmapsPath = "/bitmaps/GPS"
local metadataDir = "/documents/user"

local floor = math.floor
local min = math.min
local max = math.max
local abs = math.abs
local sqrt = math.sqrt
local sin = math.sin
local cos = math.cos
local asin = math.asin
local atan = math.atan
local log = math.log
local tan = math.tan
local deg = math.deg
local exp = math.exp
local pi = math.pi
local sformat = string.format
local tconcat = table.concat

local DEG_TO_RAD = pi / 180
local EARTH_R = 6371000
local HOME_STABLE_FRAMES = 100
local HOME_STABLE_DEG = 0.001
local DIST_JITTER_M = 5
local DRAW_PREVIEW_INTERVAL = 0.2
local WARNING_REPEAT_SECONDS = 2
local MAX_BOUNDARIES = 6
local DELETE_TOLERANCE_PX = 12
local MIN_BOUNDARY_LENGTH_PX = 3
local CFG_SEP = "|"

local WARNING_MODE_NONE = 0
local WARNING_MODE_AUDIO = 1
local WARNING_MODE_HAPTIC = 2
local WARNING_MODE_BOTH = 3
local WARNING_TYPE_MOMENTARY = 0
local WARNING_TYPE_CONSTANT = 1

local CONTROL_BUTTON_WIDTH = 72
local CONTROL_BUTTON_HEIGHT = 24
local CONTROL_BUTTON_GAP = 4
local CONTROL_MARGIN = 6

local gpsLatQuery = { name = "GPS", options = nil }
local gpsLonQuery = { name = "GPS", options = nil }
local gpsQueriesReady = false
local gpsSensorQuery = { name = "" }
local altSensorQuery = { name = "" }

local function safeCall(fn, ...)
  if type(fn) ~= "function" then
    return nil
  end
  local ok, result, extra = pcall(fn, ...)
  if not ok then
    return nil
  end
  if extra ~= nil then
    return result, extra
  end
  return result
end

local function safeInvoke(fn, ...)
  if type(fn) ~= "function" then
    return false
  end
  return pcall(fn, ...)
end

local function logRuntimeError(context, err)
  if type(print) == "function" then
    print("BMERR " .. tostring(context) .. ": " .. tostring(err))
  end
end

local function setWidgetError(widget, context, err)
  logRuntimeError(context, err)
  if type(widget) ~= "table" then
    return
  end
  widget.lastError = tostring(context) .. ": " .. tostring(err)
  widget.needsInvalidate = true
end

local function clearWidgetError(widget)
  if type(widget) == "table" then
    widget.lastError = nil
  end
end

local function clamp(value, low, high)
  if value < low then
    return low
  end
  if value > high then
    return high
  end
  return value
end

local function haversine(lat1, lon1, lat2, lon2)
  local dLat = (lat2 - lat1) * DEG_TO_RAD
  local dLon = (lon2 - lon1) * DEG_TO_RAD
  local r1 = lat1 * DEG_TO_RAD
  local r2 = lat2 * DEG_TO_RAD
  local sdLat = sin(dLat * 0.5)
  local sdLon = sin(dLon * 0.5)
  local a = sdLat * sdLat + cos(r1) * cos(r2) * sdLon * sdLon
  return EARTH_R * 2 * asin(sqrt(a))
end

local function mercatorY(lat)
  local latRad = lat * DEG_TO_RAD
  return log(tan(pi / 4 + latRad * 0.5))
end

local function extractNumber(content, key)
  if type(content) ~= "string" or type(key) ~= "string" then
    return nil
  end
  local value = content:match('"' .. key .. '"%s*:%s*([%-]?%d+%.?%d*)')
  return tonumber(value)
end

local function normalizeBoolean(value, defaultValue)
  if type(value) == "boolean" then
    return value
  end
  if type(value) == "number" then
    return value ~= 0
  end
  if type(value) == "string" then
    local normalized = value:lower()
    if normalized == "1" or normalized == "true" or normalized == "yes" or normalized == "on" then
      return true
    end
    if normalized == "0" or normalized == "false" or normalized == "no" or normalized == "off" then
      return false
    end
  end
  return defaultValue and true or false
end

local function mapStem(bitmapFile)
  if type(bitmapFile) ~= "string" or bitmapFile == "" then
    return nil
  end
  local fileName = bitmapFile:match("([^/]+)$") or bitmapFile
  return fileName:match("^(.+)%.[^.]+$") or fileName
end

local function boundarySignature(boundaries)
  local out = {}
  if type(boundaries) ~= "table" then
    return ""
  end
  for _, boundary in ipairs(boundaries) do
    out[#out + 1] = tconcat({
      tostring(boundary.x1 or ""),
      tostring(boundary.y1 or ""),
      tostring(boundary.x2 or ""),
      tostring(boundary.y2 or ""),
      tostring(boundary.lat1 or ""),
      tostring(boundary.lon1 or ""),
      tostring(boundary.lat2 or ""),
      tostring(boundary.lon2 or ""),
    }, ",")
  end
  return tconcat(out, ";")
end

local function loadMapMetadata(bitmapFile)
  local stem = mapStem(bitmapFile)
  if not stem then
    return nil, "missing map stem"
  end
  local path = metadataDir .. "/" .. stem .. ".json"
  local file = io and io.open and io.open(path, "r")
  if not file then
    return nil, "metadata not found"
  end
  local content = file:read(8192)
  file:close()
  if type(content) ~= "string" or content == "" then
    return nil, "metadata empty"
  end

  local topLat = extractNumber(content, "topLat")
  local bottomLat = extractNumber(content, "bottomLat")
  local leftLon = extractNumber(content, "leftLon")
  local rightLon = extractNumber(content, "rightLon")
  if not topLat or not bottomLat or not leftLon or not rightLon or rightLon == leftLon then
    return nil, "metadata invalid"
  end

  local topMercY = mercatorY(topLat)
  local bottomMercY = mercatorY(bottomLat)
  local mercYRange = topMercY - bottomMercY
  if mercYRange == 0 then
    return nil, "metadata mercator range invalid"
  end

  return {
    topLat = topLat,
    bottomLat = bottomLat,
    leftLon = leftLon,
    rightLon = rightLon,
    topMercY = topMercY,
    mercYRange = mercYRange,
    lonRange = rightLon - leftLon,
  }
end

local function latLonToBitmapLocal(widget, lat, lon)
  if type(widget) ~= "table" or type(widget.mapMeta) ~= "table" then
    return nil, nil
  end
  if type(lat) ~= "number" or type(lon) ~= "number" or widget.bmpW <= 0 or widget.bmpH <= 0 then
    return nil, nil
  end
  local x = ((lon - widget.mapMeta.leftLon) / widget.mapMeta.lonRange) * widget.bmpW
  local y = ((widget.mapMeta.topMercY - mercatorY(lat)) / widget.mapMeta.mercYRange) * widget.bmpH
  return x, y
end

local function bitmapLocalToLatLon(widget, x, y)
  if type(widget) ~= "table" or type(widget.mapMeta) ~= "table" then
    return nil, nil
  end
  if type(x) ~= "number" or type(y) ~= "number" or widget.bmpW <= 0 or widget.bmpH <= 0 then
    return nil, nil
  end
  local lon = widget.mapMeta.leftLon + ((x / widget.bmpW) * widget.mapMeta.lonRange)
  local merc = widget.mapMeta.topMercY - ((y / widget.bmpH) * widget.mapMeta.mercYRange)
  local lat = deg((2 * atan(exp(merc))) - (pi / 2))
  return lat, lon
end

local function bitmapLocalToScreen(widget, x, y)
  if type(widget) ~= "table" then
    return nil, nil
  end
  return (widget.offX or 0) + x, (widget.offY or 0) + y
end

local function screenToBitmapLocal(widget, x, y)
  if type(widget) ~= "table" or type(x) ~= "number" or type(y) ~= "number" then
    return nil, nil
  end
  if type(widget.mapRect) ~= "table" then
    return nil, nil
  end
  local clampedX = clamp(x, widget.mapRect.left, widget.mapRect.right)
  local clampedY = clamp(y, widget.mapRect.top, widget.mapRect.bottom)
  local localX = clamp(clampedX - (widget.offX or 0), 0, widget.bmpW)
  local localY = clamp(clampedY - (widget.offY or 0), 0, widget.bmpH)
  return localX, localY
end

local function boundaryToScreen(widget, boundary)
  if type(widget) ~= "table" or type(boundary) ~= "table" then
    return nil
  end
  local sx1, sy1 = bitmapLocalToScreen(widget, boundary.x1, boundary.y1)
  local sx2, sy2 = bitmapLocalToScreen(widget, boundary.x2, boundary.y2)
  if not sx1 or not sy1 or not sx2 or not sy2 then
    return nil
  end
  return { x1 = sx1, y1 = sy1, x2 = sx2, y2 = sy2 }
end

local function pointSegmentDistance(px, py, x1, y1, x2, y2)
  local dx = x2 - x1
  local dy = y2 - y1
  if dx == 0 and dy == 0 then
    local ddx = px - x1
    local ddy = py - y1
    return sqrt(ddx * ddx + ddy * ddy)
  end
  local t = ((px - x1) * dx + (py - y1) * dy) / ((dx * dx) + (dy * dy))
  t = clamp(t, 0, 1)
  local cx = x1 + t * dx
  local cy = y1 + t * dy
  local ddx = px - cx
  local ddy = py - cy
  return sqrt(ddx * ddx + ddy * ddy)
end

local function orientation(ax, ay, bx, by, cx, cy)
  local value = ((by - ay) * (cx - bx)) - ((bx - ax) * (cy - by))
  if abs(value) < 0.000001 then
    return 0
  end
  return value > 0 and 1 or 2
end

local function onSegment(ax, ay, bx, by, cx, cy)
  return bx <= max(ax, cx) and bx >= min(ax, cx) and by <= max(ay, cy) and by >= min(ay, cy)
end

local function segmentsIntersect(x1, y1, x2, y2, x3, y3, x4, y4)
  local o1 = orientation(x1, y1, x2, y2, x3, y3)
  local o2 = orientation(x1, y1, x2, y2, x4, y4)
  local o3 = orientation(x3, y3, x4, y4, x1, y1)
  local o4 = orientation(x3, y3, x4, y4, x2, y2)

  if o1 ~= o2 and o3 ~= o4 then
    return true
  end
  if o1 == 0 and onSegment(x1, y1, x3, y3, x2, y2) then
    return true
  end
  if o2 == 0 and onSegment(x1, y1, x4, y4, x2, y2) then
    return true
  end
  if o3 == 0 and onSegment(x3, y3, x1, y1, x4, y4) then
    return true
  end
  if o4 == 0 and onSegment(x3, y3, x2, y2, x4, y4) then
    return true
  end
  return false
end

local function resolveTouchPhase(category, value)
  local evtTouch = rawget(_G, "EVT_TOUCH")
  local function matchesCode(rawValue, names)
    if type(rawValue) ~= "number" then
      return false
    end
    for _, key in ipairs(names) do
      local code = rawget(_G, key)
      if type(code) == "number" and rawValue == code then
        return true
      end
    end
    return false
  end

  if type(evtTouch) == "number" then
    if category ~= evtTouch then
      return nil
    end
    if value == 16640 or matchesCode(value, { "TOUCH_START", "EVT_TOUCH_FIRST", "EVT_TOUCH_START", "EVT_TOUCH_TAP" }) then
      return "start"
    end
    if value == 16642 or matchesCode(value, { "TOUCH_MOVE", "EVT_TOUCH_MOVE", "EVT_TOUCH_SLIDE", "EVT_TOUCH_REPT" }) then
      return "move"
    end
    if value == 16641 or matchesCode(value, { "TOUCH_END", "EVT_TOUCH_BREAK", "EVT_TOUCH_END" }) then
      return "end"
    end
    return nil
  end

  if matchesCode(category, { "TOUCH_START", "EVT_TOUCH_FIRST", "EVT_TOUCH_START", "EVT_TOUCH_TAP" }) then
    return "start"
  end
  if matchesCode(category, { "TOUCH_MOVE", "EVT_TOUCH_MOVE", "EVT_TOUCH_SLIDE", "EVT_TOUCH_REPT" }) then
    return "move"
  end
  if matchesCode(category, { "TOUCH_END", "EVT_TOUCH_BREAK", "EVT_TOUCH_END" }) then
    return "end"
  end
  return nil
end

local function controlRects(widget)
  local w = widget.windowW or 480
  local totalHeight = (CONTROL_BUTTON_HEIGHT * 3) + (CONTROL_BUTTON_GAP * 2)
  local left = w - CONTROL_MARGIN - CONTROL_BUTTON_WIDTH
  local top = (widget.windowH or 272) - CONTROL_MARGIN - totalHeight
  return {
    draw = { left = left, top = top, right = left + CONTROL_BUTTON_WIDTH, bottom = top + CONTROL_BUTTON_HEIGHT },
    delete = {
      left = left,
      top = top + CONTROL_BUTTON_HEIGHT + CONTROL_BUTTON_GAP,
      right = left + CONTROL_BUTTON_WIDTH,
      bottom = top + (CONTROL_BUTTON_HEIGHT * 2) + CONTROL_BUTTON_GAP,
    },
    save = {
      left = left,
      top = top + (CONTROL_BUTTON_HEIGHT * 2) + (CONTROL_BUTTON_GAP * 2),
      right = left + CONTROL_BUTTON_WIDTH,
      bottom = top + (CONTROL_BUTTON_HEIGHT * 3) + (CONTROL_BUTTON_GAP * 2),
    },
  }
end

local function hitRect(x, y, rect)
  if type(x) ~= "number" or type(y) ~= "number" or type(rect) ~= "table" then
    return false
  end
  return x >= rect.left and x <= rect.right and y >= rect.top and y <= rect.bottom
end

local function triggerWarningFeedback(widget)
  if type(widget) ~= "table" or type(system) ~= "table" then
    return false
  end
  local mode = widget.boundryWarningMode or WARNING_MODE_NONE
  local didAnything = false
  if mode == WARNING_MODE_AUDIO or mode == WARNING_MODE_BOTH then
    if safeInvoke(system.playTone, 1800, 80, 0) or safeInvoke(system.playTone, 1800, 80) then
      didAnything = true
    end
  end
  if mode == WARNING_MODE_HAPTIC or mode == WARNING_MODE_BOTH then
    if safeInvoke(system.playHaptic, 200) or safeInvoke(system.playHaptic, ".") then
      didAnything = true
    end
  end
  return didAnything
end

local function loadBitmapForWidget(widget)
  local bmpFile = widget.bitmapFile or ""
  if bmpFile == "" then
    widget.loadedBitmap = nil
    widget.loadedFile = ""
    widget.bmpW = 0
    widget.bmpH = 0
    widget.mapMeta = nil
    widget.boundaries = {}
    widget.savedBoundarySignature = ""
    widget.boundaryDirty = false
    widget.mapRect = nil
    return
  end

  if widget.loadedFile == bmpFile and widget.loadedBitmap ~= nil then
    return
  end

  local bitmap = safeCall(lcd.loadBitmap, bitmapsPath .. "/" .. bmpFile)
  widget.loadedBitmap = bitmap
  widget.loadedFile = bmpFile
  if bitmap then
    widget.bmpW = safeCall(bitmap.width, bitmap) or 0
    widget.bmpH = safeCall(bitmap.height, bitmap) or 0
  else
    widget.bmpW = 0
    widget.bmpH = 0
  end

  local meta, metaErr = loadMapMetadata(bmpFile)
  widget.mapMeta = meta
  widget.mapMetaError = metaErr
  widget.boundaryLoadPending = true
end

local function updateMapRect(widget)
  local w, h = 480, 272
  if type(lcd) == "table" and type(lcd.getWindowSize) == "function" then
    local ok, ww, hh = pcall(lcd.getWindowSize)
    if ok and type(ww) == "number" and type(hh) == "number" then
      w, h = ww, hh
    end
  end
  widget.windowW = w
  widget.windowH = h
  widget.offX = floor((w - (widget.bmpW or 0)) / 2)
  widget.offY = floor((h - (widget.bmpH or 0)) / 2)
  if (widget.bmpW or 0) > 0 and (widget.bmpH or 0) > 0 then
    widget.mapRect = {
      left = max(0, widget.offX),
      top = max(0, widget.offY),
      right = min(w, widget.offX + widget.bmpW),
      bottom = min(h, widget.offY + widget.bmpH),
    }
  else
    widget.mapRect = nil
  end
end

local function sidecarPath(bitmapFile)
  local stem = mapStem(bitmapFile)
  if not stem then
    return nil
  end
  return metadataDir .. "/" .. stem .. ".boundries.json"
end

local function encodeBoundary(boundary)
  return sformat(
    '{"x1":%.3f,"y1":%.3f,"x2":%.3f,"y2":%.3f,"lat1":%.8f,"lon1":%.8f,"lat2":%.8f,"lon2":%.8f}',
    boundary.x1,
    boundary.y1,
    boundary.x2,
    boundary.y2,
    boundary.lat1,
    boundary.lon1,
    boundary.lat2,
    boundary.lon2
  )
end

local function saveBoundaries(widget)
  local path = sidecarPath(widget.bitmapFile)
  if not path or not io or type(io.open) ~= "function" then
    return false, "sidecar path unavailable"
  end
  local parts = {}
  for _, boundary in ipairs(widget.boundaries or {}) do
    parts[#parts + 1] = encodeBoundary(boundary)
  end
  local payload = sformat(
    '{"schemaVersion":1,"mapFile":"%s","boundaries":[%s]}',
    tostring(widget.bitmapFile or ""),
    tconcat(parts, ",")
  )
  local file = io.open(path, "w")
  if not file then
    return false, "failed to open sidecar for write"
  end
  file:write(payload)
  file:close()
  widget.savedBoundarySignature = boundarySignature(widget.boundaries)
  widget.boundaryDirty = false
  return true
end

local function parseBoundaryObjects(content)
  local boundaries = {}
  if type(content) ~= "string" then
    return boundaries, "sidecar empty"
  end
  for x1, y1, x2, y2, lat1, lon1, lat2, lon2 in content:gmatch(
    '{"x1":([%-]?[%d%.]+),"y1":([%-]?[%d%.]+),"x2":([%-]?[%d%.]+),"y2":([%-]?[%d%.]+),"lat1":([%-]?[%d%.]+),"lon1":([%-]?[%d%.]+),"lat2":([%-]?[%d%.]+),"lon2":([%-]?[%d%.]+)}'
  ) do
    boundaries[#boundaries + 1] = {
      x1 = tonumber(x1),
      y1 = tonumber(y1),
      x2 = tonumber(x2),
      y2 = tonumber(y2),
      lat1 = tonumber(lat1),
      lon1 = tonumber(lon1),
      lat2 = tonumber(lat2),
      lon2 = tonumber(lon2),
    }
  end
  if content:find('"boundaries"%s*:%s*%[') and #boundaries == 0 then
    return {}, "sidecar malformed"
  end
  return boundaries
end

local function loadBoundaries(widget)
  widget.boundaryLoadPending = false
  widget.boundaries = {}
  widget.savedBoundarySignature = ""
  widget.boundaryDirty = false
  widget.draftBoundary = nil
  local path = sidecarPath(widget.bitmapFile)
  if not path or not io or type(io.open) ~= "function" then
    return
  end
  local file = io.open(path, "r")
  if not file then
    return
  end
  local content = file:read(65535)
  file:close()
  local boundaries, parseErr = parseBoundaryObjects(content)
  if parseErr and parseErr ~= "" then
    logRuntimeError("sidecar-load", parseErr)
    widget.boundaries = {}
    widget.savedBoundarySignature = ""
    widget.boundaryDirty = false
    return
  end
  for idx = 1, min(#boundaries, MAX_BOUNDARIES) do
    widget.boundaries[#widget.boundaries + 1] = boundaries[idx]
  end
  widget.savedBoundarySignature = boundarySignature(widget.boundaries)
end

local function markBoundariesDirty(widget)
  widget.boundaryDirty = boundarySignature(widget.boundaries) ~= (widget.savedBoundarySignature or "")
  widget.needsInvalidate = true
end

local function makeBoundary(widget, x1, y1, x2, y2)
  local lat1, lon1 = bitmapLocalToLatLon(widget, x1, y1)
  local lat2, lon2 = bitmapLocalToLatLon(widget, x2, y2)
  if not lat1 or not lon1 or not lat2 or not lon2 then
    return nil
  end
  return {
    x1 = x1,
    y1 = y1,
    x2 = x2,
    y2 = y2,
    lat1 = lat1,
    lon1 = lon1,
    lat2 = lat2,
    lon2 = lon2,
  }
end

local function addBoundary(widget, x1, y1, x2, y2)
  if #widget.boundaries >= MAX_BOUNDARIES then
    return false
  end
  local ddx = x2 - x1
  local ddy = y2 - y1
  if sqrt((ddx * ddx) + (ddy * ddy)) < MIN_BOUNDARY_LENGTH_PX then
    return false
  end
  local boundary = makeBoundary(widget, x1, y1, x2, y2)
  if not boundary then
    return false
  end
  widget.boundaries[#widget.boundaries + 1] = boundary
  markBoundariesDirty(widget)
  return true
end

local function commitDraftBoundary(widget, x, y)
  if type(widget) ~= "table" or type(widget.draftBoundary) ~= "table" then
    return false
  end

  local draft = widget.draftBoundary
  if type(x) == "number" and type(y) == "number" then
    local localX, localY = screenToBitmapLocal(widget, x, y)
    draft.x2 = localX
    draft.y2 = localY
  elseif type(widget.pendingDraftPoint) == "table" then
    if type(widget.pendingDraftPoint.x) == "number" then
      draft.x2 = widget.pendingDraftPoint.x
    end
    if type(widget.pendingDraftPoint.y) == "number" then
      draft.y2 = widget.pendingDraftPoint.y
    end
  end

  widget.draftBoundary = nil
  widget.pendingDraftPoint = nil
  return addBoundary(widget, draft.x1, draft.y1, draft.x2, draft.y2)
end

local function saveCurrentBoundaries(widget)
  -- Save must persist the in-progress draft without turning the save tap into a new endpoint.
  if type(widget) == "table" and type(widget.draftBoundary) == "table" then
    commitDraftBoundary(widget)
  end
  return saveBoundaries(widget)
end

local function removeBoundaryAtPoint(widget, screenX, screenY)
  local bestIndex = nil
  local bestDistance = nil
  for idx, boundary in ipairs(widget.boundaries) do
    local segment = boundaryToScreen(widget, boundary)
    if segment then
      local distance = pointSegmentDistance(screenX, screenY, segment.x1, segment.y1, segment.x2, segment.y2)
      if distance <= DELETE_TOLERANCE_PX and (not bestDistance or distance < bestDistance) then
        bestDistance = distance
        bestIndex = idx
      end
    end
  end
  if not bestIndex then
    return false
  end
  table.remove(widget.boundaries, bestIndex)
  markBoundariesDirty(widget)
  return true
end

local function drawTextSafe(x, y, text, flags)
  if type(lcd) ~= "table" or type(lcd.drawText) ~= "function" then
    return
  end
  lcd.drawText(x, y, text, flags)
end

local function drawFilledRectangleSafe(x, y, w, h)
  if type(lcd) == "table" and type(lcd.drawFilledRectangle) == "function" then
    lcd.drawFilledRectangle(x, y, w, h)
  end
end

local function drawRectangleSafe(x, y, w, h)
  if type(lcd) == "table" and type(lcd.drawRectangle) == "function" then
    lcd.drawRectangle(x, y, w, h)
  end
end

local function drawLineSafe(x1, y1, x2, y2)
  if type(lcd) == "table" and type(lcd.drawLine) == "function" then
    lcd.drawLine(x1, y1, x2, y2)
  end
end

local function setColor(widget, key)
  if type(lcd) ~= "table" or type(lcd.color) ~= "function" then
    return
  end
  local value = widget.colors[key]
  if value then
    lcd.color(value)
  end
end

local function initializeColors(widget)
  if widget.colorsInitialized or type(lcd) ~= "table" or type(lcd.RGB) ~= "function" then
    return
  end
  widget.colors = {
    white = lcd.RGB(255, 255, 255),
    red = lcd.RGB(255, 48, 48),
    orange = lcd.RGB(255, 165, 0),
    yellow = lcd.RGB(255, 215, 0),
    green = lcd.RGB(64, 200, 64),
    blue = lcd.RGB(30, 120, 210),
    gray = lcd.RGB(48, 48, 48),
    cyan = lcd.RGB(80, 220, 240),
  }
  widget.colorsInitialized = true
end

local function getSourceValue(source)
  if not source then
    return nil
  end
  local valueMember = source.value
  if type(valueMember) == "function" then
    return safeCall(valueMember, source)
  end
  if type(valueMember) ~= "nil" then
    return valueMember
  end
  return nil
end

local function getSourceAge(source)
  if not source then
    return nil
  end
  local ageMember = source.age
  if type(ageMember) == "function" then
    return safeCall(ageMember, source)
  end
  return nil
end

local function ensureGpsQueries()
  if gpsQueriesReady then
    return true
  end
  if not OPTION_LATITUDE or not OPTION_LONGITUDE then
    return false
  end
  gpsLatQuery.options = OPTION_LATITUDE
  gpsLonQuery.options = OPTION_LONGITUDE
  gpsQueriesReady = true
  return true
end

local function refreshTelemetry(widget)
  if not ensureGpsQueries() or type(system) ~= "table" or type(system.getSource) ~= "function" then
    return nil, nil
  end
  local srcLat = safeCall(system.getSource, gpsLatQuery)
  local srcLon = safeCall(system.getSource, gpsLonQuery)
  local lat = getSourceValue(srcLat)
  local lon = getSourceValue(srcLon)
  local age = getSourceAge(srcLat)
  return { srcLat = srcLat, srcLon = srcLon, age = age }, lat, lon
end

local function refreshMapState(widget)
  loadBitmapForWidget(widget)
  updateMapRect(widget)
  if widget.boundaryLoadPending then
    loadBoundaries(widget)
  end
end

local function updateAircraftPosition(widget, lat, lon)
  local x, y = latLonToBitmapLocal(widget, lat, lon)
  if not x or not y then
    return false
  end
  widget.prevLat = lat
  widget.prevLon = lon
  widget.aircraftX = x
  widget.aircraftY = y
  widget.lastCoordsText = sformat("%.5f, %.5f", lat, lon)
  local sx, sy = bitmapLocalToScreen(widget, x, y)
  widget.aircraftScreenX = sx
  widget.aircraftScreenY = sy
  return true
end

local function resetHome(widget)
  widget.homeLat = nil
  widget.homeLon = nil
  widget.homeStableLat = nil
  widget.homeStableLon = nil
  widget.homeStableFrames = 0
  widget.homeX = nil
  widget.homeY = nil
  widget.distFromHome = nil
  widget.lastGroundDistText = nil
  widget.prevHomeDistance = nil
  widget.wasExceeded = false
  widget.lastWarningAt = nil
  widget.warningActive = false
end

local function updateHomeIfStable(widget, lat, lon)
  if widget.homeLat or not lat or not lon or lat == 0 or lon == 0 then
    return false
  end
  if not widget.homeStableLat then
    widget.homeStableLat = lat
    widget.homeStableLon = lon
    widget.homeStableFrames = 1
    return false
  end
  if abs(lat - widget.homeStableLat) > HOME_STABLE_DEG or abs(lon - widget.homeStableLon) > HOME_STABLE_DEG then
    widget.homeStableLat = lat
    widget.homeStableLon = lon
    widget.homeStableFrames = 1
    return false
  end
  widget.homeStableFrames = (widget.homeStableFrames or 0) + 1
  if widget.homeStableFrames >= HOME_STABLE_FRAMES then
    widget.homeLat = lat
    widget.homeLon = lon
    widget.homeX, widget.homeY = latLonToBitmapLocal(widget, lat, lon)
    return true
  end
  return false
end

local function updateHeading(widget, lat, lon)
  if not lat or not lon then
    return false
  end
  if widget.prevHeadingLat and widget.prevHeadingLon then
    local dlat = lat - widget.prevHeadingLat
    local dlon = lon - widget.prevHeadingLon
    if abs(dlat) > 0.00001 or abs(dlon) > 0.00001 then
      local y = sin(dlon * DEG_TO_RAD) * cos(lat * DEG_TO_RAD)
      local x = cos(widget.prevHeadingLat * DEG_TO_RAD) * sin(lat * DEG_TO_RAD)
        - sin(widget.prevHeadingLat * DEG_TO_RAD) * cos(lat * DEG_TO_RAD) * cos(dlon * DEG_TO_RAD)
      local atan2fn = math.atan2 or math.atan
      widget.lastHeading = (deg(atan2fn(y, x)) + 360) % 360
      widget.prevHeadingLat = lat
      widget.prevHeadingLon = lon
      return true
    end
  else
    widget.prevHeadingLat = lat
    widget.prevHeadingLon = lon
  end
  return false
end

local function updateDistanceTexts(widget, lat, lon)
  if not widget.homeLat or not widget.homeLon then
    widget.distFromHome = nil
    return false
  end
  local groundDist = haversine(widget.homeLat, widget.homeLon, lat or widget.prevLat, lon or widget.prevLon)
  if groundDist < DIST_JITTER_M then
    groundDist = 0
  end
  if groundDist ~= widget.prevGroundDistance then
    widget.prevGroundDistance = groundDist
    if groundDist >= 1000 then
      widget.lastGroundDistText = sformat("Distance: %.1f km", groundDist / 1000)
    else
      widget.lastGroundDistText = sformat("Distance: %.0f m", groundDist)
    end
  end

  if widget.distEnabled and widget.altSrc then
    local alt = getSourceValue(widget.altSrc)
    if type(alt) == "number" then
      widget.distFromHome = sqrt((groundDist * groundDist) + (alt * alt))
    else
      widget.distFromHome = nil
    end
  else
    widget.distFromHome = nil
  end

  if widget.distFromHome then
    if widget.distFromHome >= 1000 then
      widget.distText = sformat("Distance: %.1f km", widget.distFromHome / 1000)
    else
      widget.distText = sformat("Distance: %.0f m", widget.distFromHome)
    end
  else
    widget.distText = nil
  end
  return true
end

local function isExceeded(widget, aircraftX, aircraftY)
  if type(widget.boundaries) ~= "table" or #widget.boundaries == 0 then
    return false
  end
  if type(widget.homeX) ~= "number" or type(widget.homeY) ~= "number" then
    return false
  end
  for _, boundary in ipairs(widget.boundaries) do
    if segmentsIntersect(widget.homeX, widget.homeY, aircraftX, aircraftY, boundary.x1, boundary.y1, boundary.x2, boundary.y2) then
      return true
    end
  end
  return false
end

local function updateWarnings(widget, now, lat, lon)
  local exceeded = false
  local homeDistance = nil
  if widget.homeLat and widget.homeLon and lat and lon and type(widget.aircraftX) == "number" and type(widget.aircraftY) == "number" then
    exceeded = isExceeded(widget, widget.aircraftX, widget.aircraftY)
    homeDistance = haversine(widget.homeLat, widget.homeLon, lat, lon)
  end

  local movingAway = false
  if homeDistance and widget.prevHomeDistance and homeDistance > widget.prevHomeDistance then
    movingAway = true
  end

  local warningMode = widget.boundryWarningMode or WARNING_MODE_NONE
  local warningType = widget.warningType or WARNING_TYPE_MOMENTARY

  if warningMode ~= WARNING_MODE_NONE then
    if warningType == WARNING_TYPE_MOMENTARY then
      if exceeded and not widget.wasExceeded and movingAway then
        triggerWarningFeedback(widget)
      end
    else
      if exceeded then
        local shouldReplay = false
        if not widget.wasExceeded then
          shouldReplay = movingAway
        elseif not widget.lastWarningAt or (now - widget.lastWarningAt) >= WARNING_REPEAT_SECONDS then
          shouldReplay = true
        end
        if shouldReplay then
          triggerWarningFeedback(widget)
          widget.lastWarningAt = now
        end
      else
        widget.lastWarningAt = nil
      end
    end
  else
    widget.lastWarningAt = nil
  end

  widget.warningActive = exceeded
  widget.wasExceeded = exceeded
  if homeDistance then
    widget.prevHomeDistance = homeDistance
  end
end

local function drawControlButton(widget, rect, label, active)
  setColor(widget, active and "blue" or "gray")
  drawFilledRectangleSafe(rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top)
  setColor(widget, "white")
  drawRectangleSafe(rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top)
  if type(lcd) == "table" and type(lcd.font) == "function" then
    lcd.font(FONT_XS or FONT_STD)
  end
  drawTextSafe(rect.left + 8, rect.top + 4, label)
end

local function drawIndicator(widget, isStale)
  if type(widget.aircraftScreenX) ~= "number" or type(widget.aircraftScreenY) ~= "number" then
    return
  end
  local x = clamp(widget.aircraftScreenX, 6, (widget.windowW or 480) - 6)
  local y = clamp(widget.aircraftScreenY, 6, (widget.windowH or 272) - 6)
  if widget.indicatorType == 1 and type(lcd) == "table" and type(lcd.drawLine) == "function" then
    local length = 10
    local angle = (widget.lastHeading or 0) * DEG_TO_RAD
    local tipX = x + (cos(angle) * length)
    local tipY = y + (sin(angle) * length)
    setColor(widget, isStale and "red" or "orange")
    drawLineSafe(x, y, tipX, tipY)
    drawLineSafe(x - 2, y - 2, x + 2, y + 2)
  else
    setColor(widget, isStale and "red" or "orange")
    if type(lcd) == "table" and type(lcd.drawFilledCircle) == "function" then
      lcd.drawFilledCircle(x, y, 6)
    end
  end
end

local function drawHome(widget)
  if type(widget.homeX) ~= "number" or type(widget.homeY) ~= "number" then
    return
  end
  local sx, sy = bitmapLocalToScreen(widget, widget.homeX, widget.homeY)
  if not sx or not sy then
    return
  end
  setColor(widget, "white")
  if type(lcd) == "table" and type(lcd.font) == "function" then
    lcd.font(FONT_BOLD or FONT_STD_BOLD or FONT_STD)
  end
  drawTextSafe(sx - 5, sy - 8, "H")
end

local function drawBoundaries(widget)
  setColor(widget, "cyan")
  for _, boundary in ipairs(widget.boundaries or {}) do
    local segment = boundaryToScreen(widget, boundary)
    if segment then
      drawLineSafe(segment.x1, segment.y1, segment.x2, segment.y2)
    end
  end
  if type(widget.draftBoundary) == "table" then
    setColor(widget, "yellow")
    local segment = boundaryToScreen(widget, widget.draftBoundary)
    if segment then
      drawLineSafe(segment.x1, segment.y1, segment.x2, segment.y2)
    end
  end
end

local function drawOverlay(widget)
  local rects = controlRects(widget)
  drawControlButton(widget, rects.draw, "Draw", widget.drawMode)
  drawControlButton(widget, rects.delete, "Delete", widget.deleteMode)
  drawControlButton(widget, rects.save, "Save", false)
  setColor(widget, "white")
  if type(lcd) == "table" and type(lcd.font) == "function" then
    lcd.font(FONT_XS or FONT_STD)
  end
  local status = widget.boundaryDirty and "Unsaved *" or sformat("%d/%d lines", #widget.boundaries, MAX_BOUNDARIES)
  drawTextSafe(4, 4, status)
  if widget.warningActive then
    setColor(widget, "red")
    drawTextSafe(4, 18, "Boundary exceeded")
  end
end

local function drawErrorState(widget)
  initializeColors(widget)
  setColor(widget, "gray")
  drawFilledRectangleSafe(0, 0, widget.windowW or 480, widget.windowH or 272)
  setColor(widget, "white")
  if type(lcd) == "table" and type(lcd.font) == "function" then
    lcd.font(FONT_STD_BOLD or FONT_STD)
  end
  drawTextSafe(6, 6, "BoundryMap error")
  if type(lcd) == "table" and type(lcd.font) == "function" then
    lcd.font(FONT_XS or FONT_STD)
  end
  drawTextSafe(6, 24, tostring(widget.lastError or "unknown"))
  drawTextSafe(6, 40, "Check serial log.")
end

local function create()
  return {
    bitmapFile = "",
    gpsSensor = nil,
    gpsSensorName = "",
    distEnabled = false,
    altSensorName = "",
    altSrc = nil,
    indicatorType = 0,
    signalTimeout = 2,
    boundryWarningMode = WARNING_MODE_NONE,
    warningType = WARNING_TYPE_MOMENTARY,
    loadedBitmap = nil,
    loadedFile = "",
    bmpW = 0,
    bmpH = 0,
    offX = 0,
    offY = 0,
    mapMeta = nil,
    mapMetaError = nil,
    mapRect = nil,
    boundaries = {},
    savedBoundarySignature = "",
    boundaryDirty = false,
    boundaryLoadPending = false,
    drawMode = false,
    deleteMode = false,
    touchArmed = nil,
    pendingDraftPoint = nil,
    lastDraftPreviewAt = 0,
    draftBoundary = nil,
    lastWarningAt = nil,
    warningActive = false,
    wasExceeded = false,
    prevHomeDistance = nil,
    prevGroundDistance = nil,
    prevLat = nil,
    prevLon = nil,
    prevHeadingLat = nil,
    prevHeadingLon = nil,
    lastHeading = 0,
    homeLat = nil,
    homeLon = nil,
    homeX = nil,
    homeY = nil,
    homeStableLat = nil,
    homeStableLon = nil,
    homeStableFrames = 0,
    distFromHome = nil,
    distText = nil,
    lastCoordsText = nil,
    lastGroundDistText = nil,
    staleMs = 2000,
    gpsStale = false,
    needsInvalidate = true,
    lastInvalidate = 0,
    aircraftX = nil,
    aircraftY = nil,
    aircraftScreenX = nil,
    aircraftScreenY = nil,
    lastError = nil,
    colors = {},
    colorsInitialized = false,
    windowW = 480,
    windowH = 272,
  }
end

local function configure(widget)
  if type(widget) ~= "table" or type(form) ~= "table" or type(form.addLine) ~= "function" then
    return
  end

  local line1 = form.addLine("Map")
  if type(form.addBitmapField) == "function" then
    form.addBitmapField(line1, nil, bitmapsPath, function()
      return widget.bitmapFile
    end, function(value)
      widget.bitmapFile = value
      widget.loadedFile = ""
      widget.boundaryLoadPending = true
      widget.needsInvalidate = true
    end)
  end

  local line2 = form.addLine("GPS Source")
  if type(form.addSensorField) == "function" then
    form.addSensorField(line2, nil, function()
      return widget.gpsSensor
    end, function(value)
      widget.gpsSensor = value
      if value then
        local name = safeCall(value.name, value)
        widget.gpsSensorName = (type(name) == "string" and name ~= "---") and name or ""
      else
        widget.gpsSensorName = ""
      end
    end)
  end

  local line2a = form.addLine("Heading Indicator")
  if type(form.addChoiceField) == "function" then
    form.addChoiceField(line2a, nil, { { "Dot", 0 }, { "Arrow", 1 } }, function()
      return widget.indicatorType
    end, function(value)
      widget.indicatorType = value
    end)
  end

  local line2b = form.addLine("Signal Timeout (s)")
  if type(form.addNumberField) == "function" then
    form.addNumberField(line2b, nil, 2, 30, function()
      return widget.signalTimeout
    end, function(value)
      widget.signalTimeout = value
    end)
  end

  local line3 = form.addLine("Distance")
  if type(form.addBooleanField) == "function" then
    form.addBooleanField(line3, nil, function()
      return widget.distEnabled
    end, function(value)
      widget.distEnabled = normalizeBoolean(value, false)
    end)
  end

  local line3a = form.addLine("  Altitude Source")
  if type(form.addSensorField) == "function" then
    form.addSensorField(line3a, nil, function()
      return widget.altSrc
    end, function(value)
      widget.altSrc = value
      if value then
        local name = safeCall(value.name, value)
        widget.altSensorName = (type(name) == "string" and name ~= "---") and name or ""
      else
        widget.altSensorName = ""
      end
    end)
  end

  local line4 = form.addLine("Reset Home")
  if type(form.addTextButton) == "function" then
    form.addTextButton(line4, nil, "Reset", function()
      resetHome(widget)
      widget.needsInvalidate = true
    end)
  end

  local line5 = form.addLine("Boundry Warning")
  if type(form.addChoiceField) == "function" then
    form.addChoiceField(line5, nil, {
      { "None", WARNING_MODE_NONE },
      { "Audio", WARNING_MODE_AUDIO },
      { "Haptic", WARNING_MODE_HAPTIC },
      { "Both", WARNING_MODE_BOTH },
    }, function()
      return widget.boundryWarningMode
    end, function(value)
      widget.boundryWarningMode = tonumber(value) or WARNING_MODE_NONE
    end)
  end

  local line6 = form.addLine("Warning Type")
  if type(form.addChoiceField) == "function" then
    form.addChoiceField(line6, nil, {
      { "Momentary", WARNING_TYPE_MOMENTARY },
      { "Constant", WARNING_TYPE_CONSTANT },
    }, function()
      return widget.warningType
    end, function(value)
      widget.warningType = tonumber(value) or WARNING_TYPE_MOMENTARY
    end)
  end
end

local function paint(widget)
  if type(widget) ~= "table" then
    return
  end
  local ok, err = pcall(function()
    refreshMapState(widget)
    initializeColors(widget)

    if widget.lastError then
      drawErrorState(widget)
      return
    end

    if widget.loadedBitmap then
      safeInvoke(lcd.drawBitmap, widget.offX, widget.offY, widget.loadedBitmap)
    else
      setColor(widget, "white")
      if type(lcd) == "table" and type(lcd.font) == "function" then
        lcd.font(FONT_STD_BOLD or FONT_STD)
      end
      drawTextSafe(8, 8, "No map selected")
      return
    end

    if not widget.mapMeta then
      setColor(widget, "white")
      drawTextSafe(8, 8, "Map metadata unavailable")
      return
    end

    drawBoundaries(widget)
    drawHome(widget)
    drawIndicator(widget, widget.gpsStale)
    drawOverlay(widget)

    if type(lcd) == "table" and type(lcd.font) == "function" then
      lcd.font(FONT_XS or FONT_STD)
    end
    setColor(widget, "white")
    if widget.gpsStale then
      if widget.lastCoordsText then
        drawTextSafe((widget.windowW or 480) - 140, (widget.windowH or 272) - 24, widget.lastCoordsText)
      end
      if widget.lastGroundDistText then
        drawTextSafe(4, (widget.windowH or 272) - 24, widget.lastGroundDistText)
      end
    elseif widget.distEnabled and widget.distText then
      drawTextSafe(4, (widget.windowH or 272) - 24, widget.distText)
    end
  end)
  if not ok then
    setWidgetError(widget, "paint", err)
    drawErrorState(widget)
  end
end

local function handleControlTouch(widget, phase, x, y)
  local rects = controlRects(widget)
  if phase == "start" then
    if hitRect(x, y, rects.draw) then
      widget.touchArmed = "draw"
      return true
    end
    if hitRect(x, y, rects.delete) then
      widget.touchArmed = "delete"
      return true
    end
    if hitRect(x, y, rects.save) then
      widget.touchArmed = "save"
      return true
    end
    return false
  end

  local armed = widget.touchArmed
  if not armed then
    return false
  end

  if phase == "move" then
    return true
  end

  if phase == "end" then
    widget.touchArmed = nil
    if armed == "draw" and hitRect(x, y, rects.draw) then
      widget.drawMode = not widget.drawMode
      if widget.drawMode then
        widget.deleteMode = false
      end
      widget.needsInvalidate = true
      return true
    end
    if armed == "delete" and hitRect(x, y, rects.delete) then
      widget.deleteMode = not widget.deleteMode
      if widget.deleteMode then
        widget.drawMode = false
      end
      widget.needsInvalidate = true
      return true
    end
    if armed == "save" then
      local okSave, saveErr = saveCurrentBoundaries(widget)
      if not okSave then
        logRuntimeError("sidecar-save", saveErr)
      end
      widget.needsInvalidate = true
      return true
    end
  end
  return true
end

local function handleMapTouch(widget, phase, x, y)
  if type(widget.mapRect) ~= "table" or not hitRect(x, y, widget.mapRect) then
    if phase == "end" then
      widget.draftBoundary = nil
      widget.pendingDraftPoint = nil
    end
    return false
  end

  if widget.drawMode then
    if phase == "start" then
      local localX, localY = screenToBitmapLocal(widget, x, y)
      widget.pendingDraftPoint = { x = localX, y = localY }
      widget.draftBoundary = {
        x1 = localX, y1 = localY, x2 = localX, y2 = localY,
        lat1 = 0, lon1 = 0, lat2 = 0, lon2 = 0,
      }
      widget.needsInvalidate = true
      return true
    end
    if phase == "move" and widget.draftBoundary then
      local localX, localY = screenToBitmapLocal(widget, x, y)
      widget.pendingDraftPoint = { x = localX, y = localY }
      return true
    end
    if phase == "end" and widget.draftBoundary then
      return commitDraftBoundary(widget, x, y)
    end
  elseif widget.deleteMode then
    if phase == "end" then
      return removeBoundaryAtPoint(widget, x, y)
    end
    return true
  end
  return false
end

local function wakeup(widget)
  if type(widget) ~= "table" then
    return
  end
  local ok, err = pcall(function()
    refreshMapState(widget)
    clearWidgetError(widget)
    local now = os.clock()

    local telemetry, lat, lon = refreshTelemetry(widget)
    if telemetry then
      local staleMs = (tonumber(widget.signalTimeout) or 2) * 1000
      widget.staleMs = staleMs
      local wasStale = widget.gpsStale
      widget.gpsStale = telemetry.age and telemetry.age > staleMs or false
      if wasStale ~= widget.gpsStale then
        widget.needsInvalidate = true
      end
    end

    if lat and lon and widget.mapMeta then
      if updateAircraftPosition(widget, lat, lon) then
        widget.needsInvalidate = true
      end
      if updateHeading(widget, lat, lon) then
        widget.needsInvalidate = true
      end
      if updateHomeIfStable(widget, lat, lon) then
        widget.needsInvalidate = true
      end
      updateDistanceTexts(widget, lat, lon)
      updateWarnings(widget, now, lat, lon)
    else
      widget.warningActive = false
      widget.wasExceeded = false
      widget.lastWarningAt = nil
    end

    if not widget.altSrc and widget.altSensorName ~= "" and type(system) == "table" and type(system.getSource) == "function" then
      altSensorQuery.name = widget.altSensorName
      widget.altSrc = safeCall(system.getSource, altSensorQuery)
    end
    if not widget.gpsSensor and widget.gpsSensorName ~= "" and type(system) == "table" and type(system.getSource) == "function" then
      gpsSensorQuery.name = widget.gpsSensorName
      widget.gpsSensor = safeCall(system.getSource, gpsSensorQuery)
    end

    if widget.pendingDraftPoint and widget.draftBoundary and (now - (widget.lastDraftPreviewAt or 0)) >= DRAW_PREVIEW_INTERVAL then
      widget.lastDraftPreviewAt = now
      widget.draftBoundary.x2 = widget.pendingDraftPoint.x
      widget.draftBoundary.y2 = widget.pendingDraftPoint.y
      widget.needsInvalidate = true
    end

    if widget.needsInvalidate and type(lcd) == "table" and type(lcd.isVisible) == "function" and type(lcd.invalidate) == "function" then
      if lcd.isVisible() and (now - (widget.lastInvalidate or 0)) >= 0.05 then
        lcd.invalidate()
        widget.lastInvalidate = now
        widget.needsInvalidate = false
      end
    end
  end)
  if not ok then
    setWidgetError(widget, "wakeup", err)
  end
end

local function event(widget, category, value, x, y)
  if type(widget) ~= "table" then
    return false
  end
  local ok, result = pcall(function()
    local phase = resolveTouchPhase(category, value)
    if not phase then
      return false
    end
    if handleControlTouch(widget, phase, x, y) then
      return true
    end
    return handleMapTouch(widget, phase, x, y)
  end)
  if not ok then
    setWidgetError(widget, "event", result)
    return false
  end
  return result
end

local function read(widget)
  if type(widget) ~= "table" or type(storage) ~= "table" or type(storage.read) ~= "function" then
    return
  end
  local raw = safeCall(storage.read, "cfg")
  if type(raw) ~= "string" or raw == "" then
    return
  end
  local parts = {}
  for part in (raw .. CFG_SEP):gmatch("(.-)" .. "%|") do
    parts[#parts + 1] = part
  end
  widget.bitmapFile = parts[1] or ""
  widget.gpsSensorName = parts[2] or ""
  widget.distEnabled = parts[3] == "1"
  widget.altSensorName = parts[4] or ""
  widget.indicatorType = (parts[5] == "1") and 1 or 0
  local signalTimeout = tonumber(parts[6]) or 2
  widget.signalTimeout = clamp(signalTimeout, 2, 30)
  widget.boundryWarningMode = tonumber(parts[7]) or WARNING_MODE_NONE
  widget.warningType = tonumber(parts[8]) or WARNING_TYPE_MOMENTARY

  if widget.gpsSensorName ~= "" and type(system) == "table" and type(system.getSource) == "function" then
    widget.gpsSensor = safeCall(system.getSource, { name = widget.gpsSensorName })
  end
  if widget.altSensorName ~= "" and type(system) == "table" and type(system.getSource) == "function" then
    widget.altSrc = safeCall(system.getSource, { name = widget.altSensorName })
  end
  widget.loadedFile = ""
  widget.boundaryLoadPending = true
  widget.needsInvalidate = true
end

local function write(widget)
  if type(widget) ~= "table" or type(storage) ~= "table" or type(storage.write) ~= "function" then
    return
  end
  local payload = (widget.bitmapFile or "")
    .. CFG_SEP
    .. (widget.gpsSensorName or "")
    .. CFG_SEP
    .. (widget.distEnabled and "1" or "0")
    .. CFG_SEP
    .. (widget.altSensorName or "")
    .. CFG_SEP
    .. (widget.indicatorType == 1 and "1" or "0")
    .. CFG_SEP
    .. tostring(widget.signalTimeout or 2)
    .. CFG_SEP
    .. tostring(widget.boundryWarningMode or WARNING_MODE_NONE)
    .. CFG_SEP
    .. tostring(widget.warningType or WARNING_TYPE_MOMENTARY)
  safeInvoke(storage.write, "cfg", payload)
end

local function init()
  system.registerWidget({
    key = "bdymap",
    name = "BoundryMap",
    title = false,
    create = create,
    configure = configure,
    paint = paint,
    wakeup = wakeup,
    event = event,
    read = read,
    write = write,
  })
end

local function testExports()
  return {
    resolveTouchPhase = resolveTouchPhase,
    loadMapMetadata = loadMapMetadata,
    latLonToBitmapLocal = latLonToBitmapLocal,
    bitmapLocalToLatLon = bitmapLocalToLatLon,
    screenToBitmapLocal = screenToBitmapLocal,
    segmentsIntersect = segmentsIntersect,
    boundarySignature = boundarySignature,
    parseBoundaryObjects = parseBoundaryObjects,
    makeBoundary = makeBoundary,
    addBoundary = addBoundary,
    removeBoundaryAtPoint = removeBoundaryAtPoint,
    isExceeded = isExceeded,
    create = create,
    saveBoundaries = saveBoundaries,
    loadBoundaries = loadBoundaries,
    read = read,
    write = write,
    event = event,
    controlRects = controlRects,
    pointSegmentDistance = pointSegmentDistance,
    updateWarnings = updateWarnings,
    markBoundariesDirty = markBoundariesDirty,
  }
end

local module = { init = init }
if rawget(_G, "__BOUNDRYMAP_TEST__") then
  module._test = testExports()
end

return module
