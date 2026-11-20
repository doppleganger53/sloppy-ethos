local WIDGET_NAME = "BoundryMap"
local mapAssetsPath = "/scripts/BoundryMap/assets/maps"
local bitmapsPath = mapAssetsPath
local metadataDir = mapAssetsPath
local sidecarDir = mapAssetsPath

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
local MAX_PREDICTION_SECONDS = 10
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
local CONTROL_RELEASE_SLOP = 8
local END_ONLY_CONTROL_DEBOUNCE = 0.2
local TOUCH_CONTENT_Y_OFFSET = 18
local ICON_PATH = "assets/icons"
local COORDS_TEXT_X = 4
local COORDS_TEXT_BOTTOM = 40
local DIST_TEXT_BOTTOM = 24
local OVERLAY_TEXT_CHAR_WIDTH = 6
local OVERLAY_TEXT_HEIGHT = 10
local OVERLAY_TEXT_CONTROL_GAP = 2

local latOption = { options = OPTION_LATITUDE }
local lonOption = { options = OPTION_LONGITUDE }
local GPS_SLACK_M = 25
local GPS_MAX_SPEED_MPS = 150
local REACQUIRE_SECONDS = 3
local saveBoundaries
local saveCurrentBoundaries
local setColor

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

local function finite(value)
  return type(value) == "number" and value > -math.huge and value < math.huge
end

local function validCoordinates(lat, lon)
  -- Mercator has no finite projection at either pole.
  return finite(lat) and finite(lon) and abs(lat) < 90 and abs(lon) <= 180
end

local function haversine(lat1, lon1, lat2, lon2)
  local dLat = (lat2 - lat1) * DEG_TO_RAD
  local dLon = (lon2 - lon1) * DEG_TO_RAD
  local r1 = lat1 * DEG_TO_RAD
  local r2 = lat2 * DEG_TO_RAD
  local sdLat = sin(dLat * 0.5)
  local sdLon = sin(dLon * 0.5)
  local a = sdLat * sdLat + cos(r1) * cos(r2) * sdLon * sdLon
  return EARTH_R * 2 * asin(sqrt(clamp(a, 0, 1)))
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
  if not validCoordinates(topLat, leftLon) or not validCoordinates(bottomLat, rightLon) or rightLon == leftLon then
    return nil, "metadata invalid"
  end

  local topMercY = mercatorY(topLat)
  local bottomMercY = mercatorY(bottomLat)
  local mercYRange = topMercY - bottomMercY
  if not finite(mercYRange) or mercYRange == 0 then
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
  if not validCoordinates(lat, lon) or widget.bmpW <= 0 or widget.bmpH <= 0 then
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
  if (widget.displayAngle or 0) == 0 then
    return (widget.offX or 0) + x, (widget.offY or 0) + y
  end
  local dx, dy = x - widget.bmpW / 2, y - widget.bmpH / 2
  return widget.centerX + dx * widget.angleCos - dy * widget.angleSin,
    widget.centerY + dx * widget.angleSin + dy * widget.angleCos
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
  if (widget.displayAngle or 0) ~= 0 then
    local dx, dy = clampedX - widget.centerX, clampedY - widget.centerY
    local bx = widget.bmpW / 2 + dx * widget.angleCos + dy * widget.angleSin
    local by = widget.bmpH / 2 - dx * widget.angleSin + dy * widget.angleCos
    -- The corners of an oblique rotation can be outside the source image.
    if bx < -0.0001 or by < -0.0001 or bx > widget.bmpW + 0.0001 or by > widget.bmpH + 0.0001 then
      return nil, nil
    end
    return clamp(bx, 0, widget.bmpW), clamp(by, 0, widget.bmpH)
  end
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

local function normalizeTouchPoint(x, y)
  if type(y) ~= "number" then
    return x, y
  end
  return x, y - TOUCH_CONTENT_Y_OFFSET
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

local function hitRectSlop(x, y, rect, slop)
  if type(x) ~= "number" or type(y) ~= "number" or type(rect) ~= "table" then
    return false
  end
  local pad = slop or 0
  return x >= rect.left - pad and x <= rect.right + pad and y >= rect.top - pad and y <= rect.bottom + pad
end

local function controlAtPoint(widget, x, y, slop)
  local rects = controlRects(widget)
  if hitRect(x, y, rects.draw) then
    return "draw", rects.draw
  end
  if hitRect(x, y, rects.delete) then
    return "delete", rects.delete
  end
  if hitRect(x, y, rects.save) then
    return "save", rects.save
  end
  if hitRectSlop(x, y, rects.draw, slop) then
    return "draw", rects.draw
  end
  if hitRectSlop(x, y, rects.delete, slop) then
    return "delete", rects.delete
  end
  if hitRectSlop(x, y, rects.save, slop) then
    return "save", rects.save
  end
  return nil, nil
end

local function activateControl(widget, control)
  if control == "draw" then
    widget.drawMode = not widget.drawMode
    if widget.drawMode then
      widget.deleteMode = false
    end
    widget.needsInvalidate = true
    return true
  end
  if control == "delete" then
    widget.deleteMode = not widget.deleteMode
    if widget.deleteMode then
      widget.drawMode = false
    end
    widget.needsInvalidate = true
    return true
  end
  if control == "save" then
    local okSave, saveErr = saveCurrentBoundaries(widget)
    if not okSave then
      logRuntimeError("sidecar-save", saveErr)
    end
    widget.needsInvalidate = true
    return true
  end
  return false
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
  local angle = widget.mapAngle or 0
  local mapChanged = widget.loadedFile ~= bmpFile
  local retryDue = not widget.loadedBitmap and bmpFile ~= "" and os.clock() >= (widget.nextBitmapRetry or 0)
  if not mapChanged and widget.loadedAngle == angle and not retryDue then
    return
  end

  -- A north-up display already holds the source. An angle change from a
  -- rotated display needs the original, never a rotation of a rotation.
  local bitmap = not mapChanged and widget.displayAngle == 0 and widget.loadedBitmap or nil
  if widget.loadedBitmap and not bitmap then
    widget.loadedBitmap = nil
    safeCall(collectgarbage, "collect")
  end
  widget.loadedFile = bmpFile
  widget.loadedAngle = angle
  widget.displayAngle = 0
  widget.needsInvalidate = true
  if mapChanged then
    widget.mapMeta, widget.mapMetaError = loadMapMetadata(bmpFile)
    widget.boundaryLoadPending = true
  end
  if not bitmap and bmpFile ~= "" then
    bitmap = safeCall(lcd.loadBitmap, bitmapsPath .. "/" .. bmpFile)
  end
  widget.bmpW = bitmap and safeCall(bitmap.width, bitmap) or 0
  widget.bmpH = bitmap and safeCall(bitmap.height, bitmap) or 0
  widget.drawW, widget.drawH = widget.bmpW, widget.bmpH
  if widget.bmpW <= 0 or widget.bmpH <= 0 then
    -- An undecodable file can return a zero-size native Bitmap.
    widget.loadedBitmap = nil
    widget.nextBitmapRetry = os.clock() + 2
    widget.aircraftX, widget.aircraftY = nil, nil
    widget.homeX, widget.homeY = nil, nil
    return
  end
  if mapChanged or retryDue then
    widget.homeX, widget.homeY = latLonToBitmapLocal(widget, widget.homeLat, widget.homeLon)
    widget.aircraftX, widget.aircraftY = latLonToBitmapLocal(widget, widget.prevLat, widget.prevLon)
  end
  if angle ~= 0 then
    -- One equivalent full turn selects the centered rotation path on both
    -- target runtimes. The 1.6.8 quarter-turn path can fail on large maps.
    local nativeAngle = angle % 90 == 0 and angle + 360 or angle
    local memory = safeCall(system and system.getMemoryUsage)
    local available = type(memory) == "table" and memory.luaBitmapsRamAvailable
    local rotated
    if type(available) ~= "number" or available >= 2 * widget.bmpW * widget.bmpH then
      rotated = safeCall(bitmap.rotate, bitmap, nativeAngle)
    end
    local rw = rotated and safeCall(rotated.width, rotated) or 0
    local rh = rotated and safeCall(rotated.height, rotated) or 0
    if rw > 0 and rh > 0 then
      bitmap = rotated
      widget.displayAngle = angle
      widget.drawW, widget.drawH = rw, rh
    end
  end
  widget.loadedBitmap = bitmap
  widget.angleCos = cos(widget.displayAngle * DEG_TO_RAD)
  widget.angleSin = sin(widget.displayAngle * DEG_TO_RAD)
end

local function updateMapRect(widget)
  local w, h = 480, 272
  if type(lcd) == "table" and type(lcd.getWindowSize) == "function" then
    local ok, ww, hh = pcall(lcd.getWindowSize)
    if ok and type(ww) == "number" and type(hh) == "number" then
      w, h = ww, hh
    end
  end
  local dw, dh = widget.drawW or 0, widget.drawH or 0
  if widget.rectW == dw and widget.rectH == dh and widget.windowW == w and widget.windowH == h then
    return
  end
  widget.windowW, widget.windowH = w, h
  widget.rectW, widget.rectH = dw, dh
  widget.offX, widget.offY = floor((w - dw) / 2), floor((h - dh) / 2)
  widget.centerX, widget.centerY = widget.offX + dw / 2, widget.offY + dh / 2
  widget.needsInvalidate = true
  if dw > 0 and dh > 0 then
    widget.mapRect = {
      left = max(0, widget.offX),
      top = max(0, widget.offY),
      right = min(w, widget.offX + dw),
      bottom = min(h, widget.offY + dh),
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
  return sidecarDir .. "/" .. stem .. ".boundries.json"
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

saveBoundaries = function(widget)
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
    if localX then
      draft.x2, draft.y2 = localX, localY
    end
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

saveCurrentBoundaries = function(widget)
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

local function drawShadowText(widget, x, y, text, colorKey, flags)
  if type(lcd) ~= "table" or type(lcd.drawText) ~= "function" then
    return
  end
  if type(lcd.color) ~= "function" or type(widget) ~= "table" or type(widget.colors) ~= "table" or not widget.colors.black then
    drawTextSafe(x, y, text, flags)
    return
  end
  setColor(widget, "black")
  drawTextSafe(x - 1, y - 1, text, flags)
  drawTextSafe(x + 1, y - 1, text, flags)
  drawTextSafe(x - 1, y + 1, text, flags)
  drawTextSafe(x + 1, y + 1, text, flags)
  setColor(widget, colorKey or "white")
  drawTextSafe(x, y, text, flags)
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
    lcd.drawLine(floor(x1 + 0.5), floor(y1 + 0.5), floor(x2 + 0.5), floor(y2 + 0.5))
  end
end

local function drawBitmapSafe(x, y, bitmap)
  if bitmap and type(lcd) == "table" and type(lcd.drawBitmap) == "function" then
    return safeInvoke(lcd.drawBitmap, x, y, bitmap)
  end
  return false
end

local function bitmapDimension(bitmap, methodName, fallback)
  if not bitmap then
    return fallback
  end
  local method = bitmap[methodName]
  local value = safeCall(method, bitmap)
  if type(value) == "number" then
    return value
  end
  return fallback
end

local function loadWidgetIcons(widget)
  if widget.iconsLoaded or type(lcd) ~= "table" or type(lcd.loadBitmap) ~= "function" then
    return
  end
  local function loadIcon(filename)
    local icon = safeCall(lcd.loadBitmap, ICON_PATH .. "/" .. filename)
    if bitmapDimension(icon, "width", 0) > 0 and bitmapDimension(icon, "height", 0) > 0 then
      return icon
    end
  end
  widget.homeIcon = loadIcon("home.png")
  widget.arrowIcon = loadIcon("arrow.png")
  widget.arrowRedIcon = loadIcon("arrow_red.png")
  widget.iconsLoaded = true
end

setColor = function(widget, key)
  if type(lcd) ~= "table" or type(lcd.color) ~= "function" then
    return
  end
  local value = widget.colors[key]
  if value then
    lcd.color(value)
  end
end

local function measureText(text)
  if type(lcd.getTextSize) == "function" then
    local width, height = lcd.getTextSize(text)
    if type(width) == "number" and type(height) == "number" then
      return width, height
    end
  end
  return #text * OVERLAY_TEXT_CHAR_WIDTH, OVERLAY_TEXT_HEIGHT
end

local function initializeColors(widget)
  if widget.colorsInitialized or type(lcd) ~= "table" or type(lcd.RGB) ~= "function" then
    return
  end
  widget.colors = {
    black = lcd.RGB(0, 0, 0),
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

-- ETHOS Source objects may have method-backed fields, including handles that
-- became invalid after a sensor was removed. Protect lookup and invocation.
local function readSourceMember(source, key, ...)
  local member = source[key]
  if type(member) == "function" then
    return member(source, ...)
  end
  return member
end

local function getSourceValue(source, ...)
  if source then
    return safeCall(readSourceMember, source, "value", ...)
  end
end

local function getSourceAge(source)
  if source and safeCall(readSourceMember, source, "state") ~= false then
    return safeCall(readSourceMember, source, "age")
  end
end

local function getSourceUnit(source)
  if source then
    return safeCall(readSourceMember, source, "unit")
  end
end

local function getSourceStringUnit(source)
  if source then
    return safeCall(readSourceMember, source, "stringUnit")
  end
end

local function unitEquals(unit, globalName)
  local expected = rawget(_G, globalName)
  return expected ~= nil and unit == expected
end

local function speedUnitScale(source)
  local unit = getSourceUnit(source)
  if unitEquals(unit, "UNIT_CENTIMETER_PER_SECOND") then
    return 0.01
  end
  if unitEquals(unit, "UNIT_METER_PER_SECOND") then
    return 1
  end
  if unitEquals(unit, "UNIT_FOOT_PER_SECOND") then
    return 0.3048
  end
  if unitEquals(unit, "UNIT_METER_PER_MINUTE") then
    return 1 / 60
  end
  if unitEquals(unit, "UNIT_FOOT_PER_MINUTE") then
    return 0.00508
  end
  if unitEquals(unit, "UNIT_KILOMETER_PER_HOUR") then
    return 1 / 3.6
  end
  if unitEquals(unit, "UNIT_MILE_PER_HOUR") then
    return 0.44704
  end
  if unitEquals(unit, "UNIT_KNOT") then
    return 0.514444
  end

  local unitText = getSourceStringUnit(source)
  if type(unitText) ~= "string" then
    return nil
  end
  local normalized = unitText:lower():gsub("%s+", "")
  if normalized == "cm/s" or normalized == "cmps" then
    return 0.01
  end
  if normalized == "m/s" or normalized == "mps" then
    return 1
  end
  if normalized == "ft/s" or normalized == "fps" then
    return 0.3048
  end
  if normalized == "m/min" or normalized == "mpm" then
    return 1 / 60
  end
  if normalized == "ft/min" or normalized == "fpm" then
    return 0.00508
  end
  if normalized == "km/h" or normalized == "kmh" or normalized == "kph" then
    return 1 / 3.6
  end
  if normalized == "mi/h" or normalized == "mph" then
    return 0.44704
  end
  if normalized == "kt" or normalized == "kts" or normalized == "knot" or normalized == "knots" then
    return 0.514444
  end
  return nil
end

local function speedMetersPerSecond(source)
  local value = getSourceValue(source)
  local scale = speedUnitScale(source)
  if type(value) ~= "number" or value ~= value or value <= 0 or value == math.huge or not scale then
    return nil
  end
  return value * scale
end

local function validAge(age, staleMs)
  return finite(age) and age >= 0 and age <= staleMs
end

-- Runtime samples and settings belong to a single widget instance.
local flight = {}
local flightMetrics = {
  { key = "alt", label = "Altitude", short = "Alt", name = "altSensorName", source = "altSrc" },
  { key = "speed", label = "Speed", short = "Spd", name = "speedSensorName", source = "speedSrc" },
  { key = "field1", label = "Field 1" },
  { key = "field2", label = "Field 2" },
  { key = "field3", label = "Field 3" },
}
local textColors = { "white", "cyan", "green", "yellow", "red" }

function flight.create()
  return {
    showAlt = false, showSpeed = false, showExtra = true, liveColor = 1, staleColor = 0,
    names = {}, sources = {}, readings = {}, stats = {}, seconds = 0,
  }
end

function flight.reset(data)
  data.stats = {}
  data.seconds = 0
  data.maxDistance = nil
  data.lastMoving = nil
end

local function readMetric(widget, metric)
  local data = widget.flight
  local key = metric.key
  local name = metric.name and widget[metric.name] or data.names[key]
  if not name or name == "" then
    return
  end
  local source = metric.source and widget[metric.source] or data.sources[key]
  if not source and not metric.source and system and system.getSource then
    source = safeCall(system.getSource, { name = name })
    data.sources[key] = source
  end
  local age = getSourceAge(source)
  if not validAge(age, widget.staleMs) then
    data.sources[key] = nil
    return name
  end
  local value = getSourceValue(source)
  if not finite(value) then
    data.sources[key] = nil
    return name
  end
  local unit = getSourceStringUnit(source)
  return name, value, type(unit) == "string" and unit or ""
end

function flight.visible(data, key)
  if key == "alt" then
    return data.showAlt
  elseif key == "speed" then
    return data.showSpeed
  end
  return data.showExtra and data.names[key] ~= nil and data.names[key] ~= ""
end

function flight.collect(widget, now)
  local data = widget.flight
  if data.lastSample and now >= data.lastSample and now - data.lastSample < 1 then
    return
  end
  local elapsed = data.lastSample and now - data.lastSample or 0
  if (data.pages or 1) > 1 and floor(now / 3) ~= floor((data.lastSample or 0) / 3) then
    widget.needsInvalidate = true
  end
  data.lastSample = now
  local moving = false
  for _, metric in ipairs(flightMetrics) do
    local key = metric.key
    local name, value, unit = readMetric(widget, metric)
    local reading = data.readings[key]
    if not reading then
      reading = {}
      data.readings[key] = reading
    end
    if reading.name ~= name or (unit and reading.unit ~= unit) then
      data.stats[key] = nil
    end
    if (reading.name ~= name or reading.value ~= value or (unit and reading.unit ~= unit))
      and flight.visible(data, key) then
      widget.needsInvalidate = true
    end
    if reading.name ~= name then
      reading.unit = nil
    end
    reading.name = name
    reading.value = value
    reading.unit = unit or reading.unit
    if key == "speed" and value then
      local scale = speedUnitScale(widget.speedSrc)
      moving = scale ~= nil and value * scale >= 3
    end
  end
  if moving then
    if data.lastMoving and elapsed >= 0 and elapsed <= 2 then
      data.seconds = data.seconds + elapsed
    end
    if not widget.gpsStale and finite(widget.prevGroundDistance) then
      data.maxDistance = max(data.maxDistance or 0, widget.prevGroundDistance)
    end
    for key, reading in pairs(data.readings) do
      local value = reading.value
      if value then
        local totals = data.stats[key]
        if not totals then
          totals = { low = value, high = value, mean = 0, count = 0 }
          data.stats[key] = totals
        end
        totals.low = min(totals.low, value)
        totals.high = max(totals.high, value)
        totals.count = totals.count + 1
        totals.mean = totals.mean + (value - totals.mean) / totals.count
      end
    end
  end
  data.lastMoving = moving
end

-- Slot 12 is reserved for the map angle. Escape new sensor names so they
-- can contain a literal separator or percent sign without shifting fields.
function flight.read(data, parts)
  data.showAlt = parts[13] == "1"
  data.showSpeed = parts[14] == "1"
  data.showExtra = parts[15] ~= "0"
  local live, stale = tonumber(parts[16]), tonumber(parts[17])
  data.liveColor = textColors[live] and live or 1
  data.staleColor = textColors[stale] and stale or 0
  for i = 1, 3 do
    local key = "field" .. i
    local name = (parts[17 + i] or ""):gsub("%%7[Cc]", "|"):gsub("%%25", "%%")
    if data.names[key] ~= name then
      data.names[key] = name
      data.sources[key], data.readings[key], data.stats[key] = nil, nil, nil
    end
  end
end

function flight.encode(data)
  local fields = {
    data.showAlt and "1" or "0", data.showSpeed and "1" or "0", data.showExtra and "1" or "0",
    tostring(data.liveColor), tostring(data.staleColor),
  }
  for i = 1, 3 do
    local name = data.names["field" .. i] or ""
    fields[#fields + 1] = name:gsub("%%", "%%25"):gsub("|", "%%7C")
  end
  return tconcat(fields, CFG_SEP)
end

local function altitudeScale(source)
  local unit = getSourceUnit(source)
  if unitEquals(unit, "UNIT_METER") then
    return 1
  elseif unitEquals(unit, "UNIT_FOOT") then
    return 0.3048
  end
  local label = getSourceStringUnit(source)
  if label == "m" then
    return 1
  elseif label == "ft" then
    return 0.3048
  end
end

local function altitudeMeters(widget)
  local source = widget.altSrc
  local scale = altitudeScale(source)
  if widget.homeAltitudeM and (source ~= widget.homeAltitudeSource or scale ~= widget.homeAltitudeScale) then
    widget.homeAltitudeM = nil
  end
  if scale and validAge(getSourceAge(source), widget.staleMs) then
    local value = getSourceValue(source)
    if finite(value) then
      return value * scale, scale
    end
  end
end

local function resolveSource(name)
  if name and name ~= "" and system and system.getSource then
    return safeCall(system.getSource, { name = name })
  end
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
  local sx, sy
  if x and y then
    sx, sy = bitmapLocalToScreen(widget, x, y)
  end
  local changed = widget.aircraftScreenX ~= sx or widget.aircraftScreenY ~= sy
  if lat ~= widget.prevLat or lon ~= widget.prevLon then
    widget.prevLat, widget.prevLon = lat, lon
    widget.lastCoordsText = sformat("%.5f, %.5f", lat, lon)
    changed = true
  end
  widget.aircraftX, widget.aircraftY = x, y
  widget.aircraftScreenX, widget.aircraftScreenY = sx, sy
  return changed
end

local function resetHome(widget)
  widget.homeAltitudeM, widget.homeAltitudeSource, widget.homeAltitudeScale = nil, nil, nil
  widget.homeLat = nil
  widget.homeLon = nil
  widget.homeStableLat = nil
  widget.homeStableLon = nil
  widget.homeStableFrames = 0
  widget.homeX = nil
  widget.homeY = nil
  widget.distFromHome = nil
  widget.distText = nil
  widget.lastGroundDistText = nil
  widget.prevGroundDistance = nil
  widget.prevHomeDistance = nil
  widget.wasExceeded = false
  widget.wasWarningCondition = false
  widget.lastWarningAt = nil
  widget.warningActive = false
  widget.warningImminent = false
end

local function resetGps(widget, name)
  resetHome(widget)
  widget.telemetrySourceName = name
  widget.gpsSensor = nil
  widget.nextGpsLookup = nil
  widget.lastFixAt, widget.jump = nil, nil
  widget.prevLat, widget.prevLon, widget.lastCoordsText = nil, nil, nil
  widget.aircraftX, widget.aircraftY = nil, nil
  widget.aircraftScreenX, widget.aircraftScreenY = nil, nil
  widget.prevHeadingLat, widget.prevHeadingLon = nil, nil
  widget.lastHeading, widget.headingValid = 0, false
  widget.needsInvalidate = true
end

local function refreshTelemetry(widget, now)
  now = now or os.clock()
  local name = widget.gpsSensorName
  if widget.telemetrySourceName ~= name then
    resetGps(widget, name)
  end
  if not name or name == "" or not latOption.options or not lonOption.options then
    return nil
  end
  if not widget.gpsSensor and now >= (widget.nextGpsLookup or 0) then
    widget.gpsSensor = resolveSource(name)
    widget.nextGpsLookup = now + 1
  end
  local source = widget.gpsSensor
  if not source then
    return nil
  end
  local handleName = safeCall(readSourceMember, source, "name")
  if handleName and handleName ~= name then
    widget.gpsSensor = nil
    return nil
  end
  local lat = getSourceValue(source, latOption)
  local lon = getSourceValue(source, lonOption)
  local age = getSourceAge(source)
  if
    (not validCoordinates(lat, lon) or not validAge(age, widget.staleMs) or (lat == 0 and lon == 0))
    and now >= (widget.nextGpsLookup or 0)
  then
    -- A deleted/recreated sensor can leave an unusable handle behind.
    widget.gpsSensor = nil
  end
  return age, lat, lon, source
end

local function plausible(lat, lon, otherLat, otherLon, elapsed)
  local allowance = GPS_SLACK_M + GPS_MAX_SPEED_MPS * clamp(elapsed, 0, 2)
  return haversine(lat, lon, otherLat, otherLon) <= allowance
end

local function acceptFix(widget, now, age, lat, lon)
  if not validAge(age, widget.staleMs) or not validCoordinates(lat, lon) or (lat == 0 and lon == 0) then
    widget.jump = nil
    return false
  end
  local sampleAt = now - age / 1000
  local jump = widget.jump
  if
    not widget.lastFixAt
    or (
      plausible(lat, lon, widget.prevLat, widget.prevLon, sampleAt - widget.lastFixAt)
      and (not jump or lat ~= jump.lat or lon ~= jump.lon)
    )
  then
    widget.jump = nil
    widget.lastFixAt = sampleAt
    return true
  end
  if
    not jump
    or sampleAt - jump.sampleAt > widget.staleMs / 1000
    or not plausible(lat, lon, jump.lat, jump.lon, sampleAt - jump.sampleAt)
  then
    widget.jump = { lat = lat, lon = lon, age = age, firstAt = sampleAt, sampleAt = sampleAt, count = 1 }
    return false
  end
  -- A coordinate change or a younger reading proves the receiver refreshed.
  local newSample = lat ~= jump.lat or lon ~= jump.lon or age < jump.age
  jump.age = age
  if newSample then
    jump.count = jump.count + 1
    jump.lat, jump.lon, jump.sampleAt = lat, lon, sampleAt
    if jump.count >= 3 and sampleAt - jump.firstAt >= REACQUIRE_SECONDS then
      widget.jump, widget.lastFixAt = nil, sampleAt
      return true
    end
  end
  return false
end

local function updateHomeIfStable(widget, lat, lon)
  if widget.homeLat then
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
    widget.homeAltitudeM, widget.homeAltitudeScale = altitudeMeters(widget)
    widget.homeAltitudeSource = widget.homeAltitudeM and widget.altSrc or nil
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
      widget.headingValid = true
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
    widget.distText = nil
    widget.prevGroundDistance = nil
    widget.lastGroundDistText = nil
    return false
  end
  local previousDistance, previousText = widget.distFromHome, widget.distText
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

  local altitude = altitudeMeters(widget)
  if widget.distEnabled then
    if altitude and widget.homeAltitudeM then
      local height = altitude - widget.homeAltitudeM
      widget.distFromHome = sqrt(groundDist * groundDist + height * height)
    else
      widget.distFromHome = groundDist
    end
  else
    widget.distFromHome = nil
  end

  if widget.distFromHome and previousDistance ~= widget.distFromHome then
    if widget.distFromHome >= 1000 then
      widget.distText = sformat("Distance: %.1f km", widget.distFromHome / 1000)
    else
      widget.distText = sformat("Distance: %.0f m", widget.distFromHome)
    end
  elseif not widget.distFromHome then
    widget.distText = nil
  end
  return previousText ~= widget.distText
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

local function projectLatLon(lat, lon, bearingDegrees, distanceMeters)
  if
    not validCoordinates(lat, lon)
    or not finite(bearingDegrees)
    or not finite(distanceMeters)
    or distanceMeters < 0
  then
    return nil, nil
  end

  local angularDistance = distanceMeters / EARTH_R
  local bearing = bearingDegrees * DEG_TO_RAD
  local lat1 = lat * DEG_TO_RAD
  local lon1 = lon * DEG_TO_RAD
  local sinLat2 = sin(lat1) * cos(angularDistance)
    + cos(lat1) * sin(angularDistance) * cos(bearing)
  local lat2 = asin(clamp(sinLat2, -1, 1))
  local atan2fn = math.atan2 or math.atan
  local lon2 = lon1 + atan2fn(
    sin(bearing) * sin(angularDistance) * cos(lat1),
    cos(angularDistance) - sin(lat1) * sin(lat2)
  )
  return deg(lat2), ((deg(lon2) + 540) % 360) - 180
end

local function predictionCrossesBoundary(widget, lat, lon)
  local predictionSeconds = tonumber(widget.predictionSeconds) or 0
  if predictionSeconds < 1 or predictionSeconds > MAX_PREDICTION_SECONDS
    or widget.gpsStale or not widget.headingValid then
    return false
  end
  if type(widget.homeLat) ~= "number" or type(widget.homeLon) ~= "number"
    or type(widget.aircraftX) ~= "number" or type(widget.aircraftY) ~= "number"
    or type(widget.boundaries) ~= "table" or #widget.boundaries == 0 then
    return false
  end

  local speedAge = getSourceAge(widget.speedSrc)
  if not validAge(speedAge, widget.staleMs) then
    return false
  end
  local speed = speedMetersPerSecond(widget.speedSrc)
  if not speed then
    return false
  end

  local projectedLat, projectedLon = projectLatLon(lat, lon, widget.lastHeading, speed * predictionSeconds)
  if not projectedLat or not projectedLon then
    return false
  end
  local projectedX, projectedY = latLonToBitmapLocal(widget, projectedLat, projectedLon)
  if not projectedX or not projectedY then
    return false
  end

  local currentHomeDistance = haversine(widget.homeLat, widget.homeLon, lat, lon)
  local projectedHomeDistance = haversine(widget.homeLat, widget.homeLon, projectedLat, projectedLon)
  if projectedHomeDistance <= currentHomeDistance then
    return false
  end

  for _, boundary in ipairs(widget.boundaries) do
    if segmentsIntersect(
      widget.aircraftX,
      widget.aircraftY,
      projectedX,
      projectedY,
      boundary.x1,
      boundary.y1,
      boundary.x2,
      boundary.y2
    ) then
      return true, projectedLat, projectedLon
    end
  end
  return false, projectedLat, projectedLon
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

  local imminent = false
  if not exceeded and lat and lon then
    imminent = predictionCrossesBoundary(widget, lat, lon)
  end
  local warningCondition = exceeded or imminent
  local wasWarningCondition = widget.wasWarningCondition or false

  local warningMode = widget.boundryWarningMode or WARNING_MODE_NONE
  local warningType = widget.warningType or WARNING_TYPE_MOMENTARY

  if warningMode ~= WARNING_MODE_NONE then
    if warningType == WARNING_TYPE_MOMENTARY then
      if warningCondition and not wasWarningCondition and (imminent or movingAway) then
        triggerWarningFeedback(widget)
      end
    else
      if warningCondition then
        local shouldReplay = false
        if not wasWarningCondition then
          shouldReplay = imminent or movingAway
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
  widget.warningImminent = imminent
  widget.wasExceeded = exceeded
  widget.wasWarningCondition = warningCondition
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
  if type(widget.aircraftX) ~= "number" or type(widget.aircraftY) ~= "number" then
    return
  end
  local x, y = bitmapLocalToScreen(widget, widget.aircraftX, widget.aircraftY)
  if widget.displayAngle ~= 0 and not hitRect(x, y, widget.mapRect) then
    return
  end
  x = clamp(floor(x + 0.5), 6, (widget.windowW or 480) - 6)
  y = clamp(floor(y + 0.5), 6, (widget.windowH or 272) - 6)
  local heading = ((widget.lastHeading or 0) + (widget.displayAngle or 0)) % 360
  if widget.indicatorType == 1 then
    local icon = isStale and widget.arrowRedIcon or widget.arrowIcon
    if icon and type(icon.rotate) == "function" then
      if widget.rotatedIconSource ~= icon or widget.rotatedIconHeading ~= heading then
        widget.rotatedIcon = nil
        widget.rotatedIcon = safeCall(icon.rotate, icon, heading)
        widget.rotatedIconSource, widget.rotatedIconHeading = icon, heading
      end
      local rotated = widget.rotatedIcon
      if rotated then
        local rw = bitmapDimension(rotated, "width", 16)
        local rh = bitmapDimension(rotated, "height", 16)
        if drawBitmapSafe(x - floor(rw * 0.5), y - floor(rh * 0.5), rotated) then
          return
        end
      end
    end
  end
  if widget.indicatorType == 1 and type(lcd) == "table" and type(lcd.drawLine) == "function" then
    local length = 10
    local angle = heading * DEG_TO_RAD
    local tipX = x + (sin(angle) * length)
    local tipY = y - (cos(angle) * length)
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
  sx, sy = floor(sx + 0.5), floor(sy + 0.5)
  if widget.homeIcon then
    local iw = bitmapDimension(widget.homeIcon, "width", 16)
    local ih = bitmapDimension(widget.homeIcon, "height", 20)
    if drawBitmapSafe(sx - floor(iw * 0.5), sy - floor(ih * 0.5), widget.homeIcon) then
      return
    end
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
  local compact = widget.windowW < 400 or widget.windowH < 200
  if type(lcd.font) == "function" then
    lcd.font((widget.windowW >= 600 and FONT_S) or FONT_XS or FONT_STD)
  end
  local _, height = measureText("Ag")
  local step = max(14, height + 2)
  local data = widget.flight
  local live = textColors[data.liveColor] or "white"
  local stale = textColors[data.staleColor] or "red"
  local status = widget.boundaryDirty and "Unsaved *" or sformat("%d/%d lines", #widget.boundaries, MAX_BOUNDARIES)
  drawShadowText(widget, 4, 4, status, "white")
  local nextY = 4 + step
  if widget.warningActive or widget.warningImminent then
    local warning = widget.warningActive and "Boundary exceeded" or "Boundary ahead"
    drawShadowText(widget, 4, nextY, warning, widget.warningActive and "red" or "yellow")
    nextY = nextY + step
  end

  local leftWidth = rects.draw.left - 8
  local function bottomRow(text, requestedY, color)
    local y = requestedY
    if y + height >= rects.draw.top and measureText(text) > leftWidth then
      y = min(y, rects.draw.top - height - OVERLAY_TEXT_CONTROL_GAP)
    end
    drawShadowText(widget, COORDS_TEXT_X, y, text, color)
    return y
  end
  local bottom = widget.windowH - 2
  local distance
  if widget.gpsStale then
    distance = widget.lastGroundDistText
  elseif widget.distEnabled then
    distance = widget.distText
  end
  if distance then
    if compact then
      distance = distance:gsub("^Distance: ", "")
    end
    bottom = bottomRow(distance, widget.windowH - max(DIST_TEXT_BOTTOM, height + 2), widget.gpsStale and stale or live)
  end
  if widget.coordsEnabled and widget.lastCoordsText then
    local color = widget.gpsStale and (textColors[data.staleColor] or "white") or live
    bottom = bottomRow(widget.lastCoordsText, min(widget.windowH - COORDS_TEXT_BOTTOM, bottom - step), color)
  end

  local visible = {}
  for _, metric in ipairs(flightMetrics) do
    if flight.visible(data, metric.key) then
      visible[#visible + 1] = metric
    end
  end
  local slots = min(#visible, max(0, floor((bottom - nextY) / step)))
  local pages = slots > 0 and math.ceil(#visible / slots) or 1
  data.pages = pages
  local first = (floor((data.lastSample or 0) / 3) % pages) * slots + 1
  for index = first, min(#visible, first + slots - 1) do
    local metric = visible[index]
    local reading = data.readings[metric.key] or {}
    local value = reading.value
    local unit = reading.unit or ""
    local suffix = value and (sformat("%.1f", value):gsub("%.0$", "") .. (unit ~= "" and " " .. unit or "")) or "--"
    local label = metric.short and (compact and metric.short or metric.label) or data.names[metric.key]
    local width = nextY + height < rects.draw.top and widget.windowW - 8 or leftWidth
    local line = (label or metric.label) .. " " .. suffix
    if measureText(line) > width then
      line = (metric.short or ("F" .. metric.key:sub(-1))) .. " " .. suffix
    end
    if measureText(line) <= width then
      drawShadowText(widget, 4, nextY, line, value and live or stale)
    end
    nextY = nextY + step
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
    mapAngle = 0,
    gpsSensor = nil,
    gpsSensorName = "",
    distEnabled = false,
    altSensorName = "",
    altSrc = nil,
    speedSensorName = "",
    speedSrc = nil,
    flight = flight.create(),
    predictionSeconds = 0,
    indicatorType = 0,
    signalTimeout = 2,
    coordsEnabled = false,
    boundryWarningMode = WARNING_MODE_NONE,
    warningType = WARNING_TYPE_MOMENTARY,
    loadedBitmap = nil,
    loadedFile = "",
    iconsLoaded = false,
    homeIcon = nil,
    arrowIcon = nil,
    arrowRedIcon = nil,
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
    mapTouchActive = false,
    pendingDraftPoint = nil,
    lastDraftPreviewAt = 0,
    draftBoundary = nil,
    lastWarningAt = nil,
    warningActive = false,
    warningImminent = false,
    wasExceeded = false,
    wasWarningCondition = false,
    prevHomeDistance = nil,
    prevGroundDistance = nil,
    prevLat = nil,
    prevLon = nil,
    prevHeadingLat = nil,
    prevHeadingLon = nil,
    lastHeading = 0,
    headingValid = false,
    telemetrySourceName = nil,
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
    lastEndOnlyControl = nil,
    lastEndOnlyAt = nil,
    aircraftX = nil,
    aircraftY = nil,
    lastError = nil,
    colors = {},
    colorsInitialized = false,
    windowW = 480,
    windowH = 272,
  }
end

local buildMainForm, buildDiagForm

local function addStaticLine(label, text)
  local line = form.addLine(label)
  if type(form.addStaticText) == "function" then
    form.addStaticText(line, nil, tostring(text or ""))
  end
  return line
end

local function displayFileName(path)
  if type(path) ~= "string" or path == "" then
    return ""
  end
  return path:match("([^/]+)$") or path
end

local function fileStatus(status, path)
  return status .. " (" .. displayFileName(path) .. ")"
end

local function fileStatusDetail(status, path, detail)
  return status .. " (" .. displayFileName(path) .. ": " .. tostring(detail or "unknown") .. ")"
end

local function gpsDiagnostics(widget)
  local name = widget.gpsSensorName or ""
  if name == "" then
    return "Not configured", "-"
  end
  local age, lat, lon, source = refreshTelemetry(widget)
  if not source then
    return "Not found (" .. name .. ")", "-"
  end
  if validCoordinates(lat, lon) and (lat ~= 0 or lon ~= 0) then
    local status = validAge(age, widget.signalTimeout * 1000) and "Found" or "Stale"
    return status .. " (" .. name .. ")", sformat("%.5f, %.5f", lat, lon)
  end
  return "Found, no fix (" .. name .. ")", "-"
end

local function bitmapDiagnostics(widget)
  local bmpFile = widget.bitmapFile or ""
  if bmpFile == "" then
    return "No map selected"
  end
  local path = bitmapsPath .. "/" .. bmpFile
  if widget.loadedBitmap and widget.loadedFile == bmpFile then
    return fileStatus("Loaded", path)
  end
  local bitmap = nil
  if type(lcd) == "table" and type(lcd.loadBitmap) == "function" then
    bitmap = safeCall(lcd.loadBitmap, path)
  end
  if bitmapDimension(bitmap, "width", 0) > 0 and bitmapDimension(bitmap, "height", 0) > 0 then
    return fileStatus("Available, not loaded", path)
  end
  return fileStatus("Missing", path)
end

local function metadataDiagnostics(widget)
  local bmpFile = widget.bitmapFile or ""
  local stem = mapStem(bmpFile)
  if not stem then
    return "No map selected"
  end
  local path = metadataDir .. "/" .. stem .. ".json"
  local meta, metaErr = loadMapMetadata(bmpFile)
  if meta then
    return fileStatus("OK", path)
  end
  if metaErr == "metadata not found" then
    return fileStatus("Missing", path)
  end
  return fileStatusDetail("Malformed", path, metaErr or "metadata invalid")
end

local function sidecarDiagnostics(widget)
  local path = sidecarPath(widget.bitmapFile)
  if not path then
    return "No map selected"
  end
  if not io or type(io.open) ~= "function" then
    return "Unavailable (" .. path .. ")"
  end
  local file = io.open(path, "r")
  if not file then
    return fileStatus("Missing", path)
  end
  local content = file:read(65535)
  file:close()
  if type(content) ~= "string" or content == "" then
    return fileStatusDetail("Malformed", path, "sidecar empty")
  end
  local boundaries, parseErr = parseBoundaryObjects(content)
  if parseErr and parseErr ~= "" then
    return fileStatusDetail("Malformed", path, parseErr)
  end
  return sformat("Loaded %d lines (%s)", #boundaries, mapStem(widget.bitmapFile) or displayFileName(path))
end

local function buildDiagnostics(widget)
  local gpsStatus, gpsCoords = gpsDiagnostics(widget)
  return {
    gpsStatus = gpsStatus,
    gpsCoords = gpsCoords,
    bitmapStatus = bitmapDiagnostics(widget),
    metadataStatus = metadataDiagnostics(widget),
    sidecarStatus = sidecarDiagnostics(widget),
    errorStatus = tostring(widget.errorText or widget.lastError or "None"),
  }
end

buildDiagForm = function(widget)
  if type(widget) ~= "table" or type(form) ~= "table" or type(form.addLine) ~= "function" then
    return
  end

  if type(form.clear) == "function" then
    form.clear()
  end

  local diagnostics = buildDiagnostics(widget)
  addStaticLine("GPS Source", diagnostics.gpsStatus)
  addStaticLine("GPS Lat/Lon", diagnostics.gpsCoords)
  addStaticLine("Map Bitmap", diagnostics.bitmapStatus)
  addStaticLine("JSON Metadata", diagnostics.metadataStatus)
  addStaticLine("Boundary Sidecar", diagnostics.sidecarStatus)
  addStaticLine("Last Error", diagnostics.errorStatus)

  form.addLine("")
  local lineBack = form.addLine("")
  if type(form.addTextButton) == "function" then
    form.addTextButton(lineBack, nil, "Back", function()
      buildMainForm(widget)
    end)
  end
end

local function pageButton(label, text, callback)
  form.addTextButton(form.addLine(label), nil, text, callback)
end

local function showStats(widget)
  local data = widget.flight
  form.clear()
  addStaticLine("Moving time", sformat("%d:%02d", floor(data.seconds / 60), floor(data.seconds % 60)))
  addStaticLine("Max ground distance", data.maxDistance and sformat("%.0f m", data.maxDistance) or "--")
  addStaticLine("Samples", "Min / Mean / Max")
  for _, metric in ipairs(flightMetrics) do
    local reading = data.readings[metric.key] or {}
    local label = metric.short and metric.label or data.names[metric.key]
    if label and label ~= "" then
      local totals = data.stats[metric.key]
      local unit = reading.unit or ""
      local values = totals and sformat("%.1f / %.1f / %.1f", totals.low, totals.mean, totals.high) or "--"
      addStaticLine(label, values .. (unit ~= "" and " " .. unit or ""))
    end
  end
  pageButton("", "Refresh", function() showStats(widget) end)
  pageButton("Reset statistics", "Reset", function()
    flight.reset(data)
    showStats(widget)
  end)
  pageButton("", "Back", function() buildMainForm(widget) end)
end

local function showDisplaySettings(widget)
  local data = widget.flight
  form.clear()
  local function choice(label, key, items, toggle)
    form.addChoiceField(form.addLine(label), nil, items, function()
      if toggle then
        return data[key] and 1 or 0
      end
      return data[key]
    end, function(value)
      if toggle then
        data[key] = value == 1
      else
        data[key] = value
      end
      widget.needsInvalidate = true
    end)
  end
  local onOff = { { "Off", 0 }, { "On", 1 } }
  choice("Show altitude", "showAlt", onOff, true)
  choice("Show speed", "showSpeed", onOff, true)
  choice("Show extra fields", "showExtra", onOff, true)
  for _, metric in ipairs(flightMetrics) do
    if not metric.source then
      local key = metric.key
      form.addSensorField(form.addLine(metric.label), nil, function()
        if not data.sources[key] and data.names[key] and data.names[key] ~= "" then
          data.sources[key] = safeCall(system.getSource, { name = data.names[key] })
        end
        return data.sources[key]
      end, function(value)
        local name = value and safeCall(value.name, value)
        data.names[key] = name ~= "---" and name or ""
        data.sources[key] = value
        data.readings[key] = nil
        data.stats[key] = nil
        data.lastSample = nil
        widget.needsInvalidate = true
      end, nil)
    end
  end
  local colors = { { "White", 1 }, { "Cyan", 2 }, { "Green", 3 }, { "Yellow", 4 }, { "Red", 5 } }
  choice("Live text", "liveColor", colors)
  local staleColors = { { "Default", 0 } }
  for _, item in ipairs(colors) do
    staleColors[#staleColors + 1] = item
  end
  choice("Stale text", "staleColor", staleColors)
  pageButton("", "Back", function() buildMainForm(widget) end)
end

-- Use the stored Boolean on either form control.
local function addToggle(widget, label, key)
  local line = form.addLine(label)
  local function setValue(value)
    widget[key] = normalizeBoolean(value, false)
    widget.needsInvalidate = true
  end
  if type(form.addBooleanField) == "function" then
    form.addBooleanField(line, nil, function()
      return widget[key]
    end, setValue)
  elseif type(form.addChoiceField) == "function" then
    form.addChoiceField(line, nil, { { "Off", 0 }, { "On", 1 } }, function()
      return widget[key] and 1 or 0
    end, setValue)
  end
end

buildMainForm = function(widget)
  if type(widget) ~= "table" or type(form) ~= "table" or type(form.addLine) ~= "function" then
    return
  end

  if type(form.clear) == "function" then
    form.clear()
  end

  local line1 = form.addLine("Map")
  if type(form.addBitmapField) == "function" then
    form.addBitmapField(line1, nil, bitmapsPath, function()
      return widget.bitmapFile
    end, function(value)
      if widget.bitmapFile == value and widget.loadedBitmap then
        return
      end
      widget.bitmapFile = value or ""
      widget.loadedFile = nil
      widget.boundaryLoadPending = true
      widget.needsInvalidate = true
    end)
  end

  if type(form.addNumberField) == "function" then
    local line = form.addLine("Map rotation (degrees CW)")
    form.addNumberField(line, nil, 0, 359, function()
      return widget.mapAngle
    end, function(value)
      widget.mapAngle = floor(clamp(tonumber(value) or 0, 0, 359))
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
      resetGps(widget, widget.gpsSensorName)
      widget.gpsSensor = value
    end, nil)
  end

  local line2a = form.addLine("Heading Indicator")
  if type(form.addChoiceField) == "function" then
    form.addChoiceField(line2a, nil, { { "Dot", 0 }, { "Arrow", 1 } }, function()
      return widget.indicatorType
    end, function(value)
      widget.indicatorType = value
      widget.needsInvalidate = true
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

  addToggle(widget, "Coordinates", "coordsEnabled")
  addToggle(widget, "Distance", "distEnabled")

  local line3a = form.addLine("  Altitude Source")
  if type(form.addSensorField) == "function" then
    form.addSensorField(line3a, nil, function()
      return widget.altSrc
    end, function(value)
      widget.homeAltitudeM, widget.homeAltitudeSource, widget.homeAltitudeScale = nil, nil, nil
      widget.altSrc = value
      if value then
        local name = safeCall(value.name, value)
        widget.altSensorName = (type(name) == "string" and name ~= "---") and name or ""
      else
        widget.altSensorName = ""
      end
    end, nil)
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

  local line6a = form.addLine("Speed Source")
  if type(form.addSensorField) == "function" then
    form.addSensorField(line6a, nil, function()
      return widget.speedSrc
    end, function(value)
      widget.speedSrc = value
      if value then
        local name = safeCall(value.name, value)
        widget.speedSensorName = (type(name) == "string" and name ~= "---") and name or ""
      else
        widget.speedSensorName = ""
      end
    end, nil)
  end

  local line6b = form.addLine("Pre-warning Time")
  if type(form.addChoiceField) == "function" then
    form.addChoiceField(line6b, nil, {
      { "Off", 0 },
      { "1 s", 1 },
      { "2 s", 2 },
      { "3 s", 3 },
      { "4 s", 4 },
      { "5 s", 5 },
      { "6 s", 6 },
      { "7 s", 7 },
      { "8 s", 8 },
      { "9 s", 9 },
      { "10 s", 10 },
    }, function()
      return widget.predictionSeconds
    end, function(value)
      widget.predictionSeconds = clamp(tonumber(value) or 0, 0, MAX_PREDICTION_SECONDS)
    end)
  end

  pageButton("Telemetry overlays", "Set up", function() showDisplaySettings(widget) end)
  pageButton("Flight statistics", "View", function() showStats(widget) end)

  local line7 = form.addLine("Diagnostics")
  if type(form.addTextButton) == "function" then
    form.addTextButton(line7, nil, "Run", function()
      buildDiagForm(widget)
    end)
  end
end

local function configure(widget)
  buildMainForm(widget)
end

local function paint(widget)
  if type(widget) ~= "table" then
    return
  end
  local ok, err = pcall(function()
    refreshMapState(widget)
    initializeColors(widget)
    loadWidgetIcons(widget)

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
      drawTextSafe(8, 8, widget.bitmapFile ~= "" and "Map bitmap unavailable" or "No map selected")
      return
    end

    if not widget.mapMeta then
      setColor(widget, "white")
      drawTextSafe(8, 8, "Map metadata unavailable")
      return
    end

    local clip = widget.displayAngle ~= 0 and widget.mapRect and lcd.setClipping
    if clip then
      local r = widget.mapRect
      clip(r.left, r.top, r.right - r.left, r.bottom - r.top)
    end
    drawBoundaries(widget)
    drawHome(widget)
    drawIndicator(widget, widget.gpsStale)
    if clip then
      clip()
    end
    drawOverlay(widget)
    if widget.mapAngle ~= 0 then
      local label = widget.displayAngle == 0 and "N up!" or sformat("N %d°", widget.displayAngle)
      local rect = controlRects(widget).draw
      local right = rect.top < 24 and rect.left - 4 or widget.windowW - 4
      drawShadowText(widget, right, 2, label, "white", RIGHT)
    end
  end)
  if not ok then
    setWidgetError(widget, "paint", err)
    drawErrorState(widget)
  end
end

local function handleControlTouch(widget, phase, x, y)
  local exactControl = controlAtPoint(widget, x, y, 0)
  local releaseControl = controlAtPoint(widget, x, y, CONTROL_RELEASE_SLOP)
  if phase == "start" then
    if exactControl then
      widget.touchArmed = exactControl
      widget.touchActivatedOnStart = false
      if exactControl == "save" then
        activateControl(widget, exactControl)
        widget.touchActivatedOnStart = true
      end
      return true
    end
    return false
  end

  local armed = widget.touchArmed
  if not armed then
    if phase == "end" and exactControl then
      if exactControl == "save" and widget.mapTouchActive then
        return false
      end
      local now = os.clock()
      if widget.lastEndOnlyControl == exactControl and widget.lastEndOnlyAt and (now - widget.lastEndOnlyAt) < END_ONLY_CONTROL_DEBOUNCE then
        return true
      end
      widget.lastEndOnlyControl = exactControl
      widget.lastEndOnlyAt = now
      activateControl(widget, exactControl)
      return true
    end
    return false
  end

  if phase == "move" then
    return true
  end

  if phase == "end" then
    widget.touchArmed = nil
    if widget.touchActivatedOnStart then
      widget.touchActivatedOnStart = false
      return true
    end
    if armed == releaseControl then
      activateControl(widget, armed)
      return true
    end
  end
  return true
end

local function handleMapTouch(widget, phase, x, y)
  local localX, localY
  if hitRect(x, y, widget.mapRect) then
    localX, localY = screenToBitmapLocal(widget, x, y)
  end
  if not localX then
    if phase == "end" then
      widget.mapTouchActive = false
      widget.draftBoundary = nil
      widget.pendingDraftPoint = nil
    end
    return false
  end

  if widget.drawMode then
    if phase == "start" then
      widget.mapTouchActive = true
      widget.pendingDraftPoint = { x = localX, y = localY }
      widget.draftBoundary = {
        x1 = localX, y1 = localY, x2 = localX, y2 = localY,
        lat1 = 0, lon1 = 0, lat2 = 0, lon2 = 0,
      }
      widget.needsInvalidate = true
      return true
    end
    if phase == "move" and widget.draftBoundary then
      widget.pendingDraftPoint = { x = localX, y = localY }
      return true
    end
    if phase == "end" and widget.draftBoundary then
      widget.mapTouchActive = false
      local startX = widget.draftBoundary.x1
      local startY = widget.draftBoundary.y1
      widget.draftBoundary = nil
      widget.pendingDraftPoint = nil
      addBoundary(widget, startX, startY, localX, localY)
      return true
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

    widget.altSrc = widget.altSrc or resolveSource(widget.altSensorName)
    widget.speedSrc = widget.speedSrc or resolveSource(widget.speedSensorName)
    widget.staleMs = widget.signalTimeout * 1000
    local wasStale, wasWarning, wasImminent = widget.gpsStale, widget.warningActive, widget.warningImminent
    local age, lat, lon = refreshTelemetry(widget, now)
    local accepted = acceptFix(widget, now, age, lat, lon)
    widget.gpsStale = widget.gpsSensorName ~= "" and not accepted
    if accepted then
      if updateAircraftPosition(widget, lat, lon) then
        widget.needsInvalidate = true
      end
      local headingChanged = updateHeading(widget, lat, lon)
      if updateHomeIfStable(widget, lat, lon) or headingChanged then
        widget.needsInvalidate = true
      end
      if updateDistanceTexts(widget, lat, lon) then
        widget.needsInvalidate = true
      end
      updateWarnings(widget, now, lat, lon)
    else
      if widget.prevLat and updateAircraftPosition(widget, widget.prevLat, widget.prevLon) then
        widget.needsInvalidate = true
      end
      widget.prevHeadingLat, widget.prevHeadingLon, widget.headingValid = nil, nil, false
      widget.homeStableLat, widget.homeStableLon, widget.homeStableFrames = nil, nil, 0
      updateWarnings(widget, now, nil, nil)
    end
    if wasStale ~= widget.gpsStale or wasWarning ~= widget.warningActive or wasImminent ~= widget.warningImminent then
      widget.needsInvalidate = true
    end

    flight.collect(widget, now)

    if widget.pendingDraftPoint and widget.draftBoundary and (now - (widget.lastDraftPreviewAt or 0)) >= DRAW_PREVIEW_INTERVAL then
      widget.lastDraftPreviewAt = now
      widget.draftBoundary.x2 = widget.pendingDraftPoint.x
      widget.draftBoundary.y2 = widget.pendingDraftPoint.y
      widget.pendingDraftPoint = nil
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
  local phase
  local ok, result = pcall(function()
    phase = resolveTouchPhase(category, value)
    if not phase then
      return false
    end
    -- Controls and map geometry share content coordinates. Raw hits can select
    -- the next button down before the content offset has been removed.
    local contentX, contentY = normalizeTouchPoint(x, y)
    if handleControlTouch(widget, phase, contentX, contentY) then
      return true
    end
    return handleMapTouch(widget, phase, contentX, contentY)
  end)
  -- Keep the drag flag during dispatch to guard Save, then end the gesture
  -- even when a control consumed the release or a handler failed.
  if phase == "end" then
    widget.mapTouchActive = false
  end
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
  flight.read(widget.flight, parts)
  widget.bitmapFile = parts[1] or ""
  widget.gpsSensorName = parts[2] or ""
  widget.distEnabled = parts[3] == "1"
  widget.altSensorName = parts[4] or ""
  widget.indicatorType = (parts[5] == "1") and 1 or 0
  local signalTimeout = tonumber(parts[6]) or 2
  widget.signalTimeout = clamp(signalTimeout, 2, 30)
  widget.boundryWarningMode = tonumber(parts[7]) or WARNING_MODE_NONE
  widget.warningType = tonumber(parts[8]) or WARNING_TYPE_MOMENTARY
  widget.coordsEnabled = parts[9] == "1"
  widget.speedSensorName = parts[10] or ""
  widget.predictionSeconds = floor(clamp(tonumber(parts[11]) or 0, 0, MAX_PREDICTION_SECONDS))
  widget.mapAngle = floor(clamp(tonumber(parts[12]) or 0, 0, 359))
  resetGps(widget, widget.gpsSensorName)
  widget.altSrc = nil
  widget.speedSrc = nil

  if widget.gpsSensorName ~= "" and type(system) == "table" and type(system.getSource) == "function" then
    widget.gpsSensor = safeCall(system.getSource, { name = widget.gpsSensorName })
  end
  if widget.altSensorName ~= "" and type(system) == "table" and type(system.getSource) == "function" then
    widget.altSrc = safeCall(system.getSource, { name = widget.altSensorName })
  end
  if widget.speedSensorName ~= "" and type(system) == "table" and type(system.getSource) == "function" then
    widget.speedSrc = safeCall(system.getSource, { name = widget.speedSensorName })
  end
  widget.loadedFile = nil
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
    .. CFG_SEP
    .. (widget.coordsEnabled and "1" or "0")
    .. CFG_SEP
    .. (widget.speedSensorName or "")
    .. CFG_SEP
    .. tostring(floor(clamp(tonumber(widget.predictionSeconds) or 0, 0, MAX_PREDICTION_SECONDS)))
    .. CFG_SEP
    .. tostring(widget.mapAngle or 0)
    .. CFG_SEP
    .. flight.encode(widget.flight)
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
    bitmapLocalToScreen = bitmapLocalToScreen,
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
    controlAtPoint = controlAtPoint,
    pointSegmentDistance = pointSegmentDistance,
    updateDistanceTexts = updateDistanceTexts,
    updateWarnings = updateWarnings,
    refreshTelemetry = refreshTelemetry,
    projectLatLon = projectLatLon,
    predictionCrossesBoundary = predictionCrossesBoundary,
    speedMetersPerSecond = speedMetersPerSecond,
    resetHome = resetHome,
    markBoundariesDirty = markBoundariesDirty,
  }
end

local module = { init = init }
if rawget(_G, "__BOUNDRYMAP_TEST__") then
  module._test = testExports()
end

return module
