_G.__BOUNDRYMAP_TEST__ = true
_G.RIGHT = 0
_G.FONT_XS = 0
_G.FONT_STD = 0
_G.FONT_STD_BOLD = 0
_G.FONT_BOLD = 0
_G.OPTION_LATITUDE = 1
_G.OPTION_LONGITUDE = 2
_G.UNIT_CENTIMETER_PER_SECOND = 101
_G.UNIT_METER_PER_SECOND = 102
_G.UNIT_FOOT_PER_SECOND = 103
_G.UNIT_METER_PER_MINUTE = 104
_G.UNIT_FOOT_PER_MINUTE = 105
_G.UNIT_KILOMETER_PER_HOUR = 106
_G.UNIT_MILE_PER_HOUR = 107
_G.UNIT_KNOT = 108
_G.EVT_TOUCH = 1
_G.EVT_TOUCH_FIRST = 100
_G.EVT_TOUCH_MOVE = 101
_G.EVT_TOUCH_BREAK = 102
_G.TOUCH_START = 16640
_G.TOUCH_MOVE = 16642
_G.TOUCH_END = 16641

local TOUCH_CONTENT_Y_OFFSET = 18
local storageValues = {}
local ioReads = {}
local ioWrites = {}
local ioWriteCounts = {}
local formRows = {}
local drawTexts = {}
local drawnBitmaps = {}
local currentColor = nil

local function resetFormRows()
  formRows = {}
end

local function rowLabel(row)
  return formRows[row] and formRows[row].label or nil
end

local function rowValue(row)
  return formRows[row] and formRows[row].value or nil
end

local function findFormRow(label)
  for index, row in ipairs(formRows) do
    if row.label == label then
      return index, row
    end
  end
  return nil, nil
end

local function resetDrawCalls()
  drawTexts = {}
  drawnBitmaps = {}
  currentColor = nil
end

_G.system = {
  registerWidget = function(_) end,
  registerSystemTool = function(_) end,
  registerTask = function(_) end,
  getSource = function(_) return nil end,
  getMemoryUsage = function()
    return 0
  end,
  playHaptic = function(_) return true end,
  playTone = function(...) return true end,
}

_G.storage = {
  read = function(key)
    return storageValues[key]
  end,
  write = function(key, value)
    storageValues[key] = value
  end,
}

_G.form = {
  clear = function()
    resetFormRows()
  end,
  addLine = function(label)
    formRows[#formRows + 1] = { label = label }
    return #formRows
  end,
  addBitmapField = function(row, _, _, getter, setter)
    formRows[row].type = "bitmap"
    formRows[row].getter = getter
    formRows[row].setter = setter
  end,
  addSensorField = function(row, _, getter, setter, filter)
    formRows[row].type = "sensor"
    formRows[row].getter = getter
    formRows[row].setter = setter
    formRows[row].filter = filter
  end,
  -- Ethos 26.1 examples surface source fields, but not boolean fields.
  addSourceField = function(row, _, getter, setter)
    formRows[row].type = "source"
    formRows[row].getter = getter
    formRows[row].setter = setter
  end,
  addChoiceField = function(row, _, choices, getter, setter)
    formRows[row].type = "choice"
    formRows[row].choices = choices
    formRows[row].getter = getter
    formRows[row].setter = setter
  end,
  addNumberField = function(row, _, low, high, getter, setter)
    formRows[row].type = "number"
    formRows[row].low = low
    formRows[row].high = high
    formRows[row].getter = getter
    formRows[row].setter = setter
  end,
  addTextButton = function(row, _, text, callback)
    formRows[row].type = "button"
    formRows[row].text = text
    formRows[row].callback = callback
  end,
  addStaticText = function(row, _, text)
    formRows[row].type = "static"
    formRows[row].value = text
  end,
}

_G.lcd = {
  RGB = function(r, g, b)
    return (r * 65536) + (g * 256) + b
  end,
  getWindowSize = function()
    return 480, 272
  end,
  color = function(value)
    currentColor = value
  end,
  font = function(_) end,
  drawText = function(x, y, text, flags)
    drawTexts[#drawTexts + 1] = { x = x, y = y, text = text, flags = flags, color = currentColor }
  end,
  drawFilledRectangle = function(_, _, _, _) end,
  drawRectangle = function(_, _, _, _) end,
  drawLine = function(_, _, _, _) end,
  drawFilledCircle = function(_, _, _) end,
  drawBitmap = function(x, y, bitmap)
    drawnBitmaps[#drawnBitmaps + 1] = { x = x, y = y, name = bitmap and bitmap.name or "" }
  end,
  loadMask = function(path)
    return {
      name = path,
      width = function()
        return 16
      end,
      height = function()
        return 20
      end,
    }
  end,
  loadBitmap = function(path)
    if path and path:find("icons/", 1, true) then
      local icon = {
        name = path,
        width = function()
          return 16
        end,
        height = function()
          return 20
        end,
      }
      function icon:rotate(heading)
        return {
          name = path .. ":rotated:" .. tostring(heading),
          width = function()
            return 16
          end,
          height = function()
            return 20
          end,
        }
      end
      return icon
    end
    return {
      name = path,
      width = function()
        return 480
      end,
      height = function()
        return 272
      end,
    }
  end,
  invalidate = function() end,
  isVisible = function()
    return true
  end,
}

_G.io = {
  stderr = {
    write = function(_, text)
      _G.__stderr__ = (_G.__stderr__ or "") .. tostring(text)
    end,
  },
  open = function(path, mode)
    if mode == "r" then
      local content = ioReads[path]
      if content == nil then
        return nil
      end
      return {
        read = function(_, _)
          return content
        end,
        close = function() end,
      }
    end
    if mode == "w" then
      local handle = {}
      function handle:write(content)
        ioWrites[path] = content
        ioWriteCounts[path] = (ioWriteCounts[path] or 0) + 1
      end
      function handle:close() end
      return handle
    end
    return nil
  end,
}

local function fail(message)
  io.stderr:write(message .. "\n")
  print(message)
  os.exit(1)
end

local function assert_equal(actual, expected, label)
  if actual ~= expected then
    fail((label or "assert_equal failed") .. ": expected=" .. tostring(expected) .. " actual=" .. tostring(actual))
  end
end

local function assert_true(value, label)
  if not value then
    fail(label or "assert_true failed")
  end
end

local function findDrawText(text, x, y)
  for _, call in ipairs(drawTexts) do
    if call.text == text and call.x == x and call.y == y then
      return call
    end
  end
  return nil
end

local function hasDrawText(text)
  for _, call in ipairs(drawTexts) do
    if call.text == text then
      return true
    end
  end
  return false
end

local function assert_shadowed_text(text, x, y, label)
  local prefix = label or text
  local upperLeft = findDrawText(text, x - 1, y - 1)
  local upperRight = findDrawText(text, x + 1, y - 1)
  local lowerLeft = findDrawText(text, x - 1, y + 1)
  local lowerRight = findDrawText(text, x + 1, y + 1)
  assert_true(upperLeft ~= nil, prefix .. " shadow upper-left")
  assert_true(upperRight ~= nil, prefix .. " shadow upper-right")
  assert_true(lowerLeft ~= nil, prefix .. " shadow lower-left")
  assert_true(lowerRight ~= nil, prefix .. " shadow lower-right")
  assert_equal(upperLeft.color, 0, prefix .. " shadow upper-left color")
  assert_equal(upperRight.color, 0, prefix .. " shadow upper-right color")
  assert_equal(lowerLeft.color, 0, prefix .. " shadow lower-left color")
  assert_equal(lowerRight.color, 0, prefix .. " shadow lower-right color")
  local mainCall = findDrawText(text, x, y)
  assert_true(mainCall ~= nil, prefix .. " main text")
  assert_true(mainCall.color ~= 0, prefix .. " main text color")
  return mainCall
end

local function estimatedTextIntersectsRect(call, rect)
  local width = #tostring(call.text or "") * 6
  local bounds = {
    left = call.x,
    top = call.y,
    right = call.x + width,
    bottom = call.y + 10,
  }
  return bounds.left < rect.right and bounds.right > rect.left and bounds.top < rect.bottom and bounds.bottom > rect.top
end

local function assert_text_avoids_controls(call, rects, label)
  assert_true(not estimatedTextIntersectsRect(call, rects.draw), label .. " avoids draw")
  assert_true(not estimatedTextIntersectsRect(call, rects.delete), label .. " avoids delete")
  assert_true(not estimatedTextIntersectsRect(call, rects.save), label .. " avoids save")
end

local module = dofile("scripts/BoundryMap/main.lua")
local test = module._test
assert_true(type(test) == "table", "expected _test export table")
local registeredWidget = nil
_G.system.registerWidget = function(spec)
  registeredWidget = spec
end
module.init()
assert_true(type(registeredWidget) == "table", "registered widget spec")

assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16640), "start", "raw touch start maps")
assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16642), "move", "raw touch move maps")
assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16641), "end", "raw touch end maps")

ioReads["/scripts/BoundryMap/assets/maps/TestMap.json"] = '{"topLat":39.78045886,"bottomLat":39.77254308,"leftLon":-75.21268129,"rightLon":-75.19585848}'
local meta = test.loadMapMetadata("TestMap.bmp")
assert_true(type(meta) == "table", "metadata loads")

local widget = test.create()
widget.mapMeta = meta
widget.bmpW = 480
widget.bmpH = 272
widget.offX = 0
widget.offY = 0
widget.mapRect = { left = 0, top = 0, right = 480, bottom = 272 }

local x, y = test.latLonToBitmapLocal(widget, 39.77650000, -75.20426988)
assert_true(type(x) == "number" and type(y) == "number", "lat lon converts to bitmap local")
local lat, lon = test.bitmapLocalToLatLon(widget, x, y)
assert_true(math.abs(lat - 39.7765) < 0.001, "bitmap local converts back to latitude")
assert_true(math.abs(lon + 75.20426988) < 0.001, "bitmap local converts back to longitude")

local sx, sy = test.screenToBitmapLocal(widget, -10, 500)
assert_equal(sx, 0, "screen clamp x")
assert_equal(sy, 272, "screen clamp y")

assert_true(test.segmentsIntersect(0, 0, 10, 10, 0, 10, 10, 0), "segments intersect")
assert_true(not test.segmentsIntersect(0, 0, 4, 0, 5, 1, 10, 1), "segments do not intersect")

assert_true(test.addBoundary(widget, 10, 10, 100, 100), "add first boundary")
assert_equal(#widget.boundaries, 1, "boundary count after add")
assert_true(widget.boundaryDirty, "add marks boundaries dirty")
assert_true(not test.addBoundary(widget, 10, 10, 11, 11), "short boundary rejected")

for idx = 2, 6 do
  assert_true(test.addBoundary(widget, idx * 10, 20, idx * 10, 100), "boundary add until cap")
end
assert_equal(#widget.boundaries, 6, "max boundaries enforced")
assert_true(not test.addBoundary(widget, 200, 20, 200, 120), "boundary add rejected at cap")

widget.offX = 0
widget.offY = 0
assert_true(test.removeBoundaryAtPoint(widget, 10, 10), "delete near line removes boundary")
assert_equal(#widget.boundaries, 5, "boundary count after delete")
assert_true(not test.removeBoundaryAtPoint(widget, 400, 10), "delete far from line does nothing")

widget.homeX = 0
widget.homeY = 0
widget.boundaries = {
  { x1 = 10, y1 = 0, x2 = 10, y2 = 40, lat1 = 0, lon1 = 0, lat2 = 0, lon2 = 0 }
}
assert_true(test.isExceeded(widget, 20, 20), "boundary exceeded when home-aircraft line intersects")
assert_true(not test.isExceeded(widget, 5, 20), "not exceeded when aircraft stays same side")

widget.bitmapFile = "TestMap.bmp"
widget.boundaries = {
  {
    x1 = 1,
    y1 = 2,
    x2 = 3,
    y2 = 4,
    lat1 = 39.1,
    lon1 = -75.1,
    lat2 = 39.2,
    lon2 = -75.2,
  },
}
assert_true(test.saveBoundaries(widget), "save boundaries succeeds")
local savedPayload = ioWrites["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"]
assert_true(type(savedPayload) == "string" and savedPayload:find('"schemaVersion":1', 1, true) ~= nil, "saved payload has schema version")

local parsed = test.parseBoundaryObjects(savedPayload)
assert_equal(#parsed, 1, "parse saved boundary payload")

ioReads["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"] = savedPayload
widget.boundaries = {}
widget.savedBoundarySignature = ""
test.loadBoundaries(widget)
assert_equal(#widget.boundaries, 1, "load boundaries restores payload")

ioReads["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"] = '{"schemaVersion":1,"mapFile":"TestMap.bmp","boundaries":[{"oops":1}]}'
widget.boundaries = {
  { x1 = 5, y1 = 5, x2 = 6, y2 = 6, lat1 = 0, lon1 = 0, lat2 = 0, lon2 = 0 },
}
test.loadBoundaries(widget)
assert_equal(#widget.boundaries, 0, "malformed boundary file fails soft to empty")

widget = test.create()
widget.bitmapFile = "TestMap.bmp"
widget.gpsSensorName = "GPS"
widget.distEnabled = true
widget.altSensorName = "Alt"
widget.indicatorType = 1
widget.signalTimeout = 9
widget.boundryWarningMode = 3
widget.warningType = 1
widget.coordsEnabled = true
widget.speedSensorName = "GPS speed"
widget.predictionSeconds = 7
test.write(widget)
assert_true(type(storageValues.cfg) == "string", "write persists config")

local restored = test.create()
test.read(restored)
assert_equal(restored.bitmapFile, "TestMap.bmp", "read restores bitmap")
assert_equal(restored.gpsSensorName, "GPS", "read restores gps sensor")
assert_true(restored.distEnabled, "read restores distance toggle")
assert_equal(restored.altSensorName, "Alt", "read restores altitude sensor")
assert_equal(restored.indicatorType, 1, "read restores indicator")
assert_equal(restored.signalTimeout, 9, "read restores signal timeout")
assert_equal(restored.boundryWarningMode, 3, "read restores warning mode")
assert_equal(restored.warningType, 1, "read restores warning type")
assert_true(restored.coordsEnabled, "read restores coordinate toggle")
assert_equal(restored.speedSensorName, "GPS speed", "read restores speed sensor")
assert_equal(restored.predictionSeconds, 7, "read restores prediction time")

storageValues.cfg = "LegacyMap.bmp|GPS|1|Alt"
local legacy = test.create()
legacy.speedSrc = { value = 99 }
test.read(legacy)
assert_equal(legacy.bitmapFile, "LegacyMap.bmp", "legacy read restores bitmap")
assert_equal(legacy.gpsSensorName, "GPS", "legacy read restores gps")
assert_true(legacy.distEnabled, "legacy read restores distance toggle")
assert_equal(legacy.altSensorName, "Alt", "legacy read restores altitude")
assert_equal(legacy.indicatorType, 0, "legacy read defaults indicator")
assert_equal(legacy.signalTimeout, 2, "legacy read defaults signal timeout")
assert_equal(legacy.boundryWarningMode, 0, "legacy read defaults warning mode")
assert_equal(legacy.warningType, 0, "legacy read defaults warning type")
assert_true(not legacy.coordsEnabled, "legacy read defaults coordinates hidden")
assert_equal(legacy.speedSensorName, "", "legacy read defaults speed source")
assert_equal(legacy.predictionSeconds, 0, "legacy read defaults prediction off")
assert_true(legacy.speedSrc == nil, "legacy read clears any previous speed source handle")

local originalTelemetryGetSource = _G.system.getSource
local telemetryQueryCalls = {}
local telemetryValues = {
  GPS1 = { lat = 39.101, lon = -75.101, age = 25 },
  GPS2 = { lat = 39.202, lon = -75.202, age = 3500 },
  RestoredGPS = { lat = 39.303, lon = -75.303, age = 50 },
}
_G.system.getSource = function(query)
  local name = query and query.name or ""
  local options = query and query.options or nil
  telemetryQueryCalls[#telemetryQueryCalls + 1] = { name = name, options = options }
  local values = telemetryValues[name]
  if not values then
    return nil
  end
  return {
    value = function()
      if options == _G.OPTION_LATITUDE then
        return values.lat
      end
      if options == _G.OPTION_LONGITUDE then
        return values.lon
      end
      return nil
    end,
    age = function()
      return values.age
    end,
  }
end

local function assert_near(actual, expected, tolerance, label)
  assert_true(type(actual) == "number" and math.abs(actual - expected) <= tolerance,
    (label or "assert_near failed") .. ": expected=" .. tostring(expected) .. " actual=" .. tostring(actual))
end

local gpsWidget1 = test.create()
gpsWidget1.gpsSensorName = "GPS1"
local telemetry1, gps1Lat, gps1Lon = test.refreshTelemetry(gpsWidget1)
assert_equal(gps1Lat, telemetryValues.GPS1.lat, "gps query uses first widget latitude name")
assert_equal(gps1Lon, telemetryValues.GPS1.lon, "gps query uses first widget longitude name")
assert_equal(telemetry1.age, telemetryValues.GPS1.age, "gps query preserves first widget source age")

local gpsWidget2 = test.create()
gpsWidget2.gpsSensorName = "GPS2"
local telemetry2, gps2Lat, gps2Lon = test.refreshTelemetry(gpsWidget2)
assert_equal(gps2Lat, telemetryValues.GPS2.lat, "gps query refreshes second widget latitude name")
assert_equal(gps2Lon, telemetryValues.GPS2.lon, "gps query refreshes second widget longitude name")
assert_equal(telemetry2.age, telemetryValues.GPS2.age, "gps query preserves second widget source age")
gpsWidget2.signalTimeout = 2
registeredWidget.wakeup(gpsWidget2)
assert_true(gpsWidget2.gpsStale, "gps age above configured timeout is stale")
gpsWidget2.signalTimeout = 4
registeredWidget.wakeup(gpsWidget2)
assert_true(not gpsWidget2.gpsStale, "gps age below configured timeout is fresh")

local _, gps1AgainLat, gps1AgainLon = test.refreshTelemetry(gpsWidget1)
assert_equal(gps1AgainLat, telemetryValues.GPS1.lat, "gps query switches back to first widget latitude name")
assert_equal(gps1AgainLon, telemetryValues.GPS1.lon, "gps query switches back to first widget longitude name")

local blankGpsWidget = test.create()
local callsBeforeBlank = #telemetryQueryCalls
local blankTelemetry, blankLat, blankLon = test.refreshTelemetry(blankGpsWidget)
assert_true(blankTelemetry == nil and blankLat == nil and blankLon == nil, "blank gps source returns no telemetry")
assert_equal(#telemetryQueryCalls, callsBeforeBlank, "blank gps source does not query the previous sensor")

storageValues.cfg = "RestoredMap.bmp|RestoredGPS|0||0|2|0|0|0||0"
local restoredGpsWidget = test.create()
test.read(restoredGpsWidget)
local restoredTelemetry, restoredLat, restoredLon = test.refreshTelemetry(restoredGpsWidget)
assert_equal(restoredLat, telemetryValues.RestoredGPS.lat, "read config refreshes gps latitude query name")
assert_equal(restoredLon, telemetryValues.RestoredGPS.lon, "read config refreshes gps longitude query name")
assert_equal(restoredTelemetry.age, telemetryValues.RestoredGPS.age, "read config keeps restored gps age")
_G.system.getSource = originalTelemetryGetSource

widget = test.create()
widget.bitmapFile = "UnsavedMap.bmp"
registeredWidget.configure(widget)
assert_equal(rowLabel(1), "Map", "main form keeps map first")
local diagnosticsRow = findFormRow("Diagnostics")
assert_true(diagnosticsRow ~= nil, "main form adds diagnostics row")
assert_equal(formRows[diagnosticsRow].text, "Run", "diagnostics button text")
local speedSourceRow = findFormRow("Speed Source")
local predictionRow = findFormRow("Pre-warning Time")
assert_true(speedSourceRow ~= nil, "main form adds speed source row")
assert_true(predictionRow ~= nil, "main form adds prediction time row")
assert_equal(formRows[predictionRow].choices[1][1], "Off", "prediction defaults include off choice")
assert_equal(formRows[predictionRow].choices[11][2], 10, "prediction choices include ten seconds")
formRows[1].setter("ChangedBeforeDiagnostics.bmp")
formRows[diagnosticsRow].callback()
assert_equal(widget.bitmapFile, "ChangedBeforeDiagnostics.bmp", "diagnostics keeps unsaved map edit")
assert_equal(rowLabel(1), "GPS Source", "diagnostics form first row")
assert_equal(rowValue(1), "Not configured", "diagnostics reports unconfigured gps")
assert_equal(rowLabel(2), "GPS Lat/Lon", "diagnostics includes coords")
assert_equal(rowValue(2), "-", "diagnostics coords absent without gps")
assert_equal(rowLabel(3), "Map Bitmap", "diagnostics includes bitmap")
assert_equal(rowValue(3), "Available, not loaded (ChangedBeforeDiagnostics.bmp)", "diagnostics checks bitmap path")
assert_equal(rowLabel(4), "JSON Metadata", "diagnostics includes metadata")
assert_equal(rowValue(4), "Missing (ChangedBeforeDiagnostics.json)", "diagnostics reports missing metadata")
assert_equal(rowLabel(5), "Boundary Sidecar", "diagnostics includes sidecar")
assert_equal(rowValue(5), "Missing (ChangedBeforeDiagnostics.boundries.json)", "diagnostics reports missing sidecar")
assert_equal(rowLabel(6), "Last Error", "diagnostics includes last error")
assert_equal(rowValue(6), "None", "diagnostics reports no error")
assert_equal(formRows[8].text, "Back", "diagnostics adds back button")
formRows[8].callback()
assert_equal(rowLabel(1), "Map", "back returns to main form")
assert_equal(widget.bitmapFile, "ChangedBeforeDiagnostics.bmp", "back keeps unsaved map edit")

widget = test.create()
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(3), "No map selected", "diagnostics reports no selected bitmap")
assert_equal(rowValue(4), "No map selected", "diagnostics reports no selected metadata")
assert_equal(rowValue(5), "No map selected", "diagnostics reports no selected sidecar")

local originalLoadBitmap = _G.lcd.loadBitmap
_G.lcd.loadBitmap = function(_)
  return nil
end
widget = test.create()
widget.bitmapFile = "MissingMap.bmp"
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(3), "Missing (MissingMap.bmp)", "diagnostics reports missing bitmap")
_G.lcd.loadBitmap = originalLoadBitmap

local originalGetSource = _G.system.getSource
_G.system.getSource = function(query)
  if query.name == "GPS1" then
    return {
      value = function()
        if query.options == _G.OPTION_LATITUDE then
          return 39.123456
        end
        if query.options == _G.OPTION_LONGITUDE then
          return -75.654321
        end
        return nil
      end,
    }
  end
  if query.name == "GPS2" then
    return {
      value = function()
        return nil
      end,
    }
  end
  return nil
end
widget = test.create()
widget.gpsSensorName = "GPS1"
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(1), "Found (GPS1)", "diagnostics reports found gps")
assert_equal(rowValue(2), "39.12346, -75.65432", "diagnostics reports gps coords")
widget = test.create()
widget.gpsSensorName = "GPS2"
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(1), "Found, no fix (GPS2)", "diagnostics reports gps without fix")
assert_equal(rowValue(2), "-", "diagnostics hides coords without fix")
widget = test.create()
widget.gpsSensorName = "MissingGPS"
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(1), "Not found (MissingGPS)", "diagnostics reports missing gps")
_G.system.getSource = originalGetSource

ioReads["/scripts/BoundryMap/assets/maps/DiagMap.json"] = '{"topLat":39.78045886,"bottomLat":39.77254308,"leftLon":-75.21268129,"rightLon":-75.19585848}'
ioReads["/scripts/BoundryMap/assets/maps/DiagMap.boundries.json"] = savedPayload
widget = test.create()
widget.bitmapFile = "DiagMap.bmp"
widget.loadedBitmap = true
widget.loadedFile = "DiagMap.bmp"
widget.lastError = "paint: bad draw"
widget.boundaries = {
  { x1 = 99, y1 = 99, x2 = 100, y2 = 100, lat1 = 1, lon1 = 1, lat2 = 2, lon2 = 2 },
}
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(3), "Loaded (DiagMap.bmp)", "diagnostics reports loaded bitmap")
assert_equal(rowValue(4), "OK (DiagMap.json)", "diagnostics reports valid metadata")
assert_equal(rowValue(5), "Loaded 1 lines (DiagMap)", "diagnostics reports loaded sidecar count")
assert_equal(rowValue(6), "paint: bad draw", "diagnostics reports last error")
assert_equal(#widget.boundaries, 1, "diagnostics does not reload boundaries")
assert_equal(widget.boundaries[1].x1, 99, "diagnostics leaves current boundaries untouched")

ioReads["/scripts/BoundryMap/assets/maps/DiagMap.boundries.json"] = '{"schemaVersion":1,"mapFile":"DiagMap.bmp","boundaries":[{"oops":1}]}'
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(5), "Malformed (DiagMap.boundries.json: sidecar malformed)", "diagnostics reports malformed sidecar")

ioReads["/scripts/BoundryMap/assets/maps/DiagMap.json"] = '{"topLat":39.78045886,"bottomLat":39.77254308,"leftLon":-75.21268129}'
registeredWidget.configure(widget)
diagnosticsRow = findFormRow("Diagnostics")
formRows[diagnosticsRow].callback()
assert_equal(rowValue(4), "Malformed (DiagMap.json: metadata invalid)", "diagnostics reports malformed metadata")

widget = test.create()
widget.windowW = 480
widget.windowH = 272
widget.mapRect = { left = 0, top = 0, right = 480, bottom = 272 }
widget.loadedBitmap = true
widget.bmpW = 480
widget.bmpH = 272
widget.mapMeta = meta
local rects = test.controlRects(widget)
local function rawTouchY(contentY)
  return contentY + TOUCH_CONTENT_Y_OFFSET
end

assert_equal(rects.draw.right - rects.draw.left, 72, "draw button width expanded")
assert_equal(rects.draw.bottom - rects.draw.top, 24, "draw button height expanded")
local drawX = rects.draw.left + 2
local drawY = rects.draw.top + 2
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, drawX, rawTouchY(drawY)), "draw button end-only tap consumed")
assert_true(widget.drawMode, "draw mode toggled on by end-only tap")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, drawX, rawTouchY(drawY)), "draw button duplicate end-only tap consumed")
assert_true(widget.drawMode, "draw mode stays on after duplicate end-only tap")
widget.lastEndOnlyAt = os.clock() - 1
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, drawX, rawTouchY(drawY)), "draw button later end-only tap consumed")
assert_true(not widget.drawMode, "draw mode toggled off by later end-only tap")
local deleteX = rects.delete.left + 2
local deleteY = rects.delete.top + 2
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, deleteX, rawTouchY(deleteY)), "delete button end-only tap consumed")
assert_true(widget.deleteMode, "delete mode toggled on by end-only tap")
widget.deleteMode = false
widget.boundaryDirty = true
local unarmedSaveX = rects.save.left + 2
local unarmedSaveY = rects.save.top + 2
ioWriteCounts["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"] = 0
assert_true(not test.event(widget, _G.EVT_TOUCH, 16641, unarmedSaveX, rawTouchY(unarmedSaveY)), "save button end-only tap remains unconsumed")
assert_true(widget.boundaryDirty, "save button end-only tap does not clear dirty state")
assert_equal(ioWriteCounts["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"], 0, "save button end-only tap does not write")
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, drawX, rawTouchY(drawY)), "draw button start consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, drawX, rawTouchY(drawY)), "draw button end consumed")
assert_true(widget.drawMode, "draw mode toggled on")

local startX = 20
local startY = 20
local endX = 120
local endY = 40
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, startX, rawTouchY(startY)), "map draw start consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16642, 80, rawTouchY(30)), "map draw move consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, endX, rawTouchY(endY)), "map draw end consumed")
assert_equal(#widget.boundaries, 1, "event draw flow adds boundary")

widget.bitmapFile = "TestMap.bmp"
widget.boundaryDirty = true
local saveStartX = rects.save.left + 2
local saveStartY = rects.save.top + 2
local saveDriftX = rects.save.right + 5
local saveDriftY = rects.save.bottom + 5
ioWriteCounts["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"] = 0
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, saveStartX, rawTouchY(saveStartY)), "save button start consumed")
assert_true(not widget.boundaryDirty, "save button clears dirty state on start")
assert_equal(ioWriteCounts["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"], 1, "save button writes on start")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, saveDriftX, rawTouchY(saveDriftY)), "save button tolerates small release drift")
assert_true(not widget.boundaryDirty, "save button remains clean after drift")
assert_equal(ioWriteCounts["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"], 1, "save button does not double-write on release")

widget.drawMode = true
widget.deleteMode = false
widget.draftBoundary = nil
widget.pendingDraftPoint = nil
local nearSaveX = rects.save.left - 5
local nearSaveY = rects.save.top + 2
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, nearSaveX, rawTouchY(nearSaveY)), "map touch near save start consumed")
assert_true(widget.draftBoundary ~= nil, "map touch just outside save starts drawing")
assert_equal(widget.draftBoundary.x1, nearSaveX, "near-save map touch preserves x")
assert_equal(widget.draftBoundary.y1, nearSaveY, "near-save map touch preserves y")

widget.deleteMode = true
widget.drawMode = false
widget.draftBoundary = nil
widget.pendingDraftPoint = nil
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, 22, rawTouchY(22)), "delete touch consumed")
assert_equal(#widget.boundaries, 0, "event delete flow removes boundary")

widget = test.create()
widget.bitmapFile = "TestMap.bmp"
widget.windowW = 480
widget.windowH = 272
widget.mapRect = { left = 0, top = 0, right = 480, bottom = 272 }
widget.loadedBitmap = true
widget.bmpW = 480
widget.bmpH = 272
widget.mapMeta = meta
widget.drawMode = true
widget.boundaries = {}
widget.draftBoundary = {
  x1 = 40,
  y1 = 30,
  x2 = 120,
  y2 = 48,
  lat1 = 0,
  lon1 = 0,
  lat2 = 0,
  lon2 = 0,
}
widget.pendingDraftPoint = { x = 120, y = 48 }
local saveRects = test.controlRects(widget)
assert_equal(saveRects.save.right - saveRects.save.left, 72, "save button width expanded")
assert_equal(saveRects.save.bottom - saveRects.save.top, 24, "save button height expanded")
local saveX = saveRects.save.left + 4
local saveY = saveRects.save.top + 4
ioWrites["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"] = nil
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, saveX, rawTouchY(saveY)), "save start with active draft stays on controls")
assert_equal(#widget.boundaries, 0, "save start does not add active draft boundary")
assert_equal(widget.draftBoundary.x2, 120, "save start does not move draft boundary endpoint")
assert_true(type(ioWrites["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"]) == "string", "save start writes sidecar with active draft")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, saveX, rawTouchY(saveY)), "save release after active draft save consumed")

widget = test.create()
widget.bitmapFile = "TestMap.bmp"
widget.windowW = 480
widget.windowH = 272
widget.mapRect = { left = 0, top = 0, right = 480, bottom = 272 }
widget.loadedBitmap = true
widget.bmpW = 480
widget.bmpH = 272
widget.mapMeta = meta
widget.drawMode = true
widget.boundaries = {}
ioWrites["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"] = nil
local overlapRects = test.controlRects(widget)
local overlapSaveX = overlapRects.save.left + 3
local overlapSaveY = overlapRects.save.top + 3
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, 40, rawTouchY(30)), "map draw over save start consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16642, overlapSaveX - 10, rawTouchY(overlapSaveY)), "map draw over save move consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, overlapSaveX, rawTouchY(overlapSaveY)), "map draw over save end consumed")
assert_equal(#widget.boundaries, 1, "map release over save adds boundary")
assert_equal(widget.boundaries[1].x1, 40, "map release over save keeps start x")
assert_equal(widget.boundaries[1].y1, 30, "map release over save keeps start y")
assert_equal(widget.boundaries[1].x2, overlapSaveX, "map release over save uses release x")
assert_equal(widget.boundaries[1].y2, overlapSaveY, "map release over save uses release y")
assert_true(ioWrites["/scripts/BoundryMap/assets/maps/TestMap.boundries.json"] == nil, "map release over save does not save sidecar")

widget = test.create()
widget.boundryWarningMode = 1
widget.warningType = 0
widget.homeLat = 39.0
widget.homeLon = -75.0
widget.homeX = 0
widget.homeY = 0
widget.aircraftX = 20
widget.aircraftY = 20
widget.boundaries = {
  { x1 = 10, y1 = 0, x2 = 10, y2 = 30, lat1 = 0, lon1 = 0, lat2 = 0, lon2 = 0 },
}
widget.prevHomeDistance = 10
local plays = 0
_G.system.playTone = function(...)
  plays = plays + 1
  return true
end
test.updateWarnings(widget, 1.0, 39.001, -74.999)
assert_equal(plays, 1, "momentary warning fires on entering exceeded while moving away")
test.updateWarnings(widget, 1.5, 39.001, -74.999)
assert_equal(plays, 1, "momentary warning does not repeat while still exceeded")

local function mockSpeedSource(value, unit, age, stringUnit)
  return {
    value = function()
      return value
    end,
    unit = function()
      return unit
    end,
    age = function()
      return age
    end,
    stringUnit = function()
      return stringUnit
    end,
  }
end

assert_near(test.speedMetersPerSecond(mockSpeedSource(12, _G.UNIT_METER_PER_SECOND, 0)), 12, 0.0001,
  "meters per second conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(36, _G.UNIT_KILOMETER_PER_HOUR, 0)), 10, 0.0001,
  "kilometers per hour conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(10, _G.UNIT_MILE_PER_HOUR, 0)), 4.4704, 0.0001,
  "miles per hour conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(100, _G.UNIT_CENTIMETER_PER_SECOND, 0)), 1, 0.0001,
  "centimeters per second conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(10, _G.UNIT_FOOT_PER_SECOND, 0)), 3.048, 0.0001,
  "feet per second conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(600, _G.UNIT_METER_PER_MINUTE, 0)), 10, 0.0001,
  "meters per minute conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(600, _G.UNIT_FOOT_PER_MINUTE, 0)), 3.048, 0.0001,
  "feet per minute conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(10, _G.UNIT_KNOT, 0)), 5.14444, 0.0001,
  "knot conversion")
assert_near(test.speedMetersPerSecond(mockSpeedSource(10, nil, 0, "knots")), 5.14444, 0.0001,
  "string speed unit fallback conversion")
assert_true(test.speedMetersPerSecond(mockSpeedSource(10, nil, 0, "widgets")) == nil,
  "unsupported speed unit disables prediction")
assert_true(test.speedMetersPerSecond(mockSpeedSource(0, _G.UNIT_METER_PER_SECOND, 0)) == nil,
  "nonpositive speed disables prediction")

local predictionWidget = test.create()
predictionWidget.mapMeta = meta
predictionWidget.bmpW = 480
predictionWidget.bmpH = 272
predictionWidget.homeLat, predictionWidget.homeLon = test.bitmapLocalToLatLon(predictionWidget, 100, 136)
local predictionLat, predictionLon = test.bitmapLocalToLatLon(predictionWidget, 200, 136)
predictionWidget.homeX = 100
predictionWidget.homeY = 136
predictionWidget.aircraftX = 200
predictionWidget.aircraftY = 136
predictionWidget.boundaries = {
  { x1 = 240, y1 = 40, x2 = 240, y2 = 230, lat1 = 0, lon1 = 0, lat2 = 0, lon2 = 0 },
}
predictionWidget.headingValid = true
predictionWidget.lastHeading = 90
predictionWidget.speedSrc = mockSpeedSource(30, _G.UNIT_METER_PER_SECOND, 0)
predictionWidget.predictionSeconds = 5
predictionWidget.staleMs = 2000
predictionWidget.gpsStale = false
local projectedLat, projectedLon = test.projectLatLon(predictionLat, predictionLon, 90, 150)
assert_true(type(projectedLat) == "number" and type(projectedLon) == "number", "prediction projects coordinates")
assert_true(test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "outbound projected path crosses boundary")

predictionWidget.lastHeading = 270
predictionWidget.boundaries[1].x1 = 180
predictionWidget.boundaries[1].x2 = 180
assert_true(not test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "inbound projected crossing is rejected by outbound gate")
predictionWidget.lastHeading = 90
predictionWidget.boundaries[1].x1 = 240
predictionWidget.boundaries[1].x2 = 240
predictionWidget.speedSrc = mockSpeedSource(30, _G.UNIT_METER_PER_SECOND, 2501)
assert_true(not test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "stale speed source disables prediction")
predictionWidget.speedSrc = mockSpeedSource(30, _G.UNIT_METER_PER_SECOND, nil)
assert_true(not test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "speed source without age disables prediction")
predictionWidget.speedSrc = mockSpeedSource(30, _G.UNIT_METER_PER_SECOND, 0)
predictionWidget.gpsStale = true
assert_true(not test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "stale gps disables prediction")
predictionWidget.gpsStale = false
predictionWidget.predictionSeconds = 0
assert_true(not test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "prediction off preserves legacy warning behavior")
predictionWidget.predictionSeconds = 11
assert_true(not test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "prediction beyond ten seconds is rejected")
predictionWidget.predictionSeconds = 5
predictionWidget.headingValid = false
assert_true(not test.predictionCrossesBoundary(predictionWidget, predictionLat, predictionLon),
  "prediction requires a derived heading")
predictionWidget.headingValid = true

predictionWidget.predictionSeconds = 5
predictionWidget.boundryWarningMode = 1
predictionWidget.warningType = 0
predictionWidget.prevHomeDistance = 1
local predictionPlays = 0
_G.system.playTone = function(...)
  predictionPlays = predictionPlays + 1
  return true
end
test.updateWarnings(predictionWidget, 10.0, predictionLat, predictionLon)
assert_true(predictionWidget.warningImminent, "prediction activates ahead state")
assert_true(not predictionWidget.warningActive, "prediction does not mark boundary exceeded")
assert_equal(predictionPlays, 1, "prediction alerts on entering ahead state")
test.updateWarnings(predictionWidget, 10.5, predictionLat, predictionLon)
assert_equal(predictionPlays, 1, "momentary prediction does not repeat while ahead")

local crossedLat, crossedLon = test.bitmapLocalToLatLon(predictionWidget, 260, 136)
predictionWidget.aircraftX = 260
predictionWidget.aircraftY = 136
test.updateWarnings(predictionWidget, 11.0, crossedLat, crossedLon)
assert_true(predictionWidget.warningActive, "actual crossing replaces ahead state")
assert_true(not predictionWidget.warningImminent, "actual crossing clears ahead state")
assert_equal(predictionPlays, 1, "ahead to exceeded transition does not double alert")

predictionWidget.aircraftX = 200
predictionWidget.aircraftY = 136
predictionWidget.lastHeading = 270
test.updateWarnings(predictionWidget, 11.5, predictionLat, predictionLon)
assert_true(not predictionWidget.wasWarningCondition, "leaving predicted path rearms warning")
predictionWidget.lastHeading = 90
test.updateWarnings(predictionWidget, 12.0, predictionLat, predictionLon)
assert_equal(predictionPlays, 2, "re-entering predicted path alerts again")

predictionWidget.warningType = 1
predictionWidget.warningActive = false
predictionWidget.warningImminent = false
predictionWidget.wasExceeded = false
predictionWidget.wasWarningCondition = false
predictionWidget.lastWarningAt = nil
predictionWidget.prevHomeDistance = 1
predictionWidget.aircraftX = 200
predictionWidget.aircraftY = 136
predictionWidget.lastHeading = 90
predictionPlays = 0
test.updateWarnings(predictionWidget, 20.0, predictionLat, predictionLon)
assert_equal(predictionPlays, 1, "constant prediction alerts on entry")
predictionWidget.aircraftX = 260
predictionWidget.aircraftY = 136
test.updateWarnings(predictionWidget, 20.5, crossedLat, crossedLon)
assert_equal(predictionPlays, 1, "constant ahead to exceeded transition keeps cadence")
test.updateWarnings(predictionWidget, 21.9, crossedLat, crossedLon)
assert_equal(predictionPlays, 1, "constant prediction waits for repeat interval")
test.updateWarnings(predictionWidget, 22.1, crossedLat, crossedLon)
assert_equal(predictionPlays, 2, "constant prediction repeats after interval")

widget = test.create()
widget.homeLat = 39.0
widget.homeLon = -75.0
widget.distEnabled = true
test.updateDistanceTexts(widget, 39.001, -75.0)
assert_true(type(widget.distText) == "string" and widget.distText:find("Distance:", 1, true) ~= nil, "distance uses ground distance without altitude")
assert_true(widget.distFromHome ~= nil and widget.distFromHome > 100, "ground distance is stored without altitude")
local groundDistanceText = widget.lastGroundDistText
widget.altSrc = {
  value = function()
    return 120
  end,
}
test.updateDistanceTexts(widget, 39.001, -75.0)
assert_true(widget.distText ~= groundDistanceText, "active distance uses 3d text when altitude is configured")
assert_equal(widget.lastGroundDistText, groundDistanceText, "cached stale distance remains 2d ground text")
widget.distEnabled = false
test.updateDistanceTexts(widget, 39.001, -75.0)
assert_true(widget.distText == nil, "distance disabled hides distance text")
assert_equal(widget.lastGroundDistText, groundDistanceText, "distance toggle does not erase stale ground text")
test.resetHome(widget)
assert_true(widget.distText == nil and widget.lastGroundDistText == nil and widget.prevGroundDistance == nil,
  "reset home clears active and stale distance state")

widget = test.create()
widget.homeLat = 39.0
widget.homeLon = -75.0
widget.distEnabled = true
test.updateDistanceTexts(widget, 39.00001, -75.0)
assert_equal(widget.lastGroundDistText, "Distance: 0 m", "ground jitter under five meters displays zero")

widget = test.create()
widget.boundryWarningMode = 2
widget.warningType = 1
widget.homeLat = 39.0
widget.homeLon = -75.0
widget.homeX = 0
widget.homeY = 0
widget.aircraftX = 20
widget.aircraftY = 20
widget.boundaries = {
  { x1 = 10, y1 = 0, x2 = 10, y2 = 30, lat1 = 0, lon1 = 0, lat2 = 0, lon2 = 0 },
}
widget.prevHomeDistance = 10
local haptics = 0
_G.system.playHaptic = function(_)
  haptics = haptics + 1
  return true
end
test.updateWarnings(widget, 1.0, 39.001, -74.999)
test.updateWarnings(widget, 2.0, 39.001, -74.999)
assert_equal(haptics, 1, "constant warning waits for repeat interval")
test.updateWarnings(widget, 3.1, 39.001, -74.999)
assert_equal(haptics, 2, "constant warning repeats after interval")

widget = test.create()
widget.bitmapFile = "TestMap.bmp"
widget.coordsEnabled = true
widget.distEnabled = true
widget.distText = "123 m"
widget.lastCoordsText = "39.12345, -75.54321"
widget.homeX = 50
widget.homeY = 60
widget.aircraftScreenX = 200
widget.aircraftScreenY = 100
widget.indicatorType = 1
widget.lastHeading = 45
resetDrawCalls()
registeredWidget.paint(widget)
local drewHomeIcon = false
local drewArrowIcon = false
for _, call in ipairs(drawnBitmaps) do
  if call.name == "assets/icons/home.png" then
    drewHomeIcon = true
  end
  if call.name == "assets/icons/arrow.png:rotated:45" then
    drewArrowIcon = true
  end
end
assert_true(drewHomeIcon, "paint draws home icon asset")
assert_true(drewArrowIcon, "paint draws rotated aircraft icon asset")
local coordText = nil
for _, call in ipairs(drawTexts) do
  if call.text == "39.12345, -75.54321" then
    coordText = call
  end
end
assert_true(coordText ~= nil, "coordinate toggle draws coordinates")
coordText = assert_shadowed_text("39.12345, -75.54321", 4, 232, "coordinates")
assert_text_avoids_controls(coordText, test.controlRects(widget), "normal coordinates")
assert_shadowed_text("0/6 lines", 4, 4, "status")
assert_shadowed_text("123 m", 4, 248, "distance")

widget.coordsEnabled = false
resetDrawCalls()
registeredWidget.paint(widget)
for _, call in ipairs(drawTexts) do
  assert_true(call.text ~= "39.12345, -75.54321", "coordinate toggle hides coordinates")
end

widget = test.create()
widget.bitmapFile = "TestMap.bmp"
widget.loadedBitmap = true
widget.loadedFile = "TestMap.bmp"
widget.mapMeta = meta
widget.boundaryLoadPending = false
widget.coordsEnabled = true
widget.gpsStale = true
widget.lastCoordsText = "39.12345, -75.54321"
widget.lastGroundDistText = "Distance: 12345.6 km"
widget.distEnabled = true
widget.distText = "Distance: 3D active"
widget.warningActive = true
widget.warningImminent = true
widget.boundaryDirty = true
local originalGetWindowSize = _G.lcd.getWindowSize
_G.lcd.getWindowSize = function()
  return 190, 160
end
resetDrawCalls()
registeredWidget.paint(widget)
local compactRects = test.controlRects(widget)
local compactStatus = assert_shadowed_text("Unsaved *", 4, 4, "compact status")
local compactWarning = assert_shadowed_text("Boundary exceeded", 4, 18, "compact warning")
local compactCoords = assert_shadowed_text("39.12345, -75.54321", 4, 62, "compact coordinates")
local compactDistance = assert_shadowed_text("Distance: 12345.6 km", 4, 50, "compact stale distance")
assert_text_avoids_controls(compactStatus, compactRects, "compact status")
assert_text_avoids_controls(compactWarning, compactRects, "compact warning")
assert_text_avoids_controls(compactCoords, compactRects, "compact coordinates")
assert_text_avoids_controls(compactDistance, compactRects, "compact stale distance")
assert_true(not hasDrawText("Distance: 3D active"), "stale paint suppresses active 3d distance text")
assert_true(not hasDrawText("Boundary ahead"), "exceeded overlay takes priority over ahead overlay")
local originalRight = _G.RIGHT
_G.RIGHT = nil
resetDrawCalls()
registeredWidget.paint(widget)
compactRects = test.controlRects(widget)
compactCoords = assert_shadowed_text("39.12345, -75.54321", 4, 62, "compact coordinates without RIGHT")
compactDistance = assert_shadowed_text("Distance: 12345.6 km", 4, 50, "compact stale distance without RIGHT")
assert_text_avoids_controls(compactCoords, compactRects, "compact coordinates without RIGHT")
assert_text_avoids_controls(compactDistance, compactRects, "compact stale distance without RIGHT")
_G.RIGHT = originalRight

widget.warningActive = false
widget.warningImminent = true
widget.distEnabled = false
widget.distText = nil
resetDrawCalls()
registeredWidget.paint(widget)
assert_shadowed_text("Boundary ahead", 4, 18, "prediction warning overlay")
assert_shadowed_text("Distance: 12345.6 km", 4, 50, "stale distance remains visible when disabled")
_G.lcd.getWindowSize = originalGetWindowSize

ioReads["/scripts/BoundryMap/assets/maps/UATMap.json"] = '{"topLat":39.78045886,"bottomLat":39.77254308,"leftLon":-75.21268129,"rightLon":-75.19585848}'
ioReads["/scripts/BoundryMap/assets/maps/UATMap.boundries.json"] = nil
ioWrites["/scripts/BoundryMap/assets/maps/UATMap.boundries.json"] = nil
widget = test.create()
registeredWidget.configure(widget)
formRows[1].setter("UATMap.bmp")
widget.gpsSensorName = "GPS"
widget.coordsEnabled = true
widget.distEnabled = true
widget.altSensorName = "Alt"
widget.indicatorType = 1
widget.signalTimeout = 2
widget.boundryWarningMode = 3
widget.warningType = 1
resetDrawCalls()
registeredWidget.paint(widget)
assert_true(widget.loadedBitmap ~= nil, "uat loads selected bitmap")
assert_true(type(widget.mapMeta) == "table", "uat loads selected metadata")
assert_true(type(widget.mapRect) == "table", "uat computes map rect")
local uatRects = test.controlRects(widget)
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, uatRects.draw.left + 4, rawTouchY(uatRects.draw.top + 4)), "uat draw control start")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, uatRects.draw.left + 4, rawTouchY(uatRects.draw.top + 4)), "uat draw control end")
assert_true(widget.drawMode, "uat draw mode enabled")
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, 240, rawTouchY(70)), "uat boundary start")
assert_true(test.event(widget, _G.EVT_TOUCH, 16642, 240, rawTouchY(136)), "uat boundary move")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, 240, rawTouchY(210)), "uat boundary end")
assert_equal(#widget.boundaries, 1, "uat boundary drawn")
assert_true(widget.boundaryDirty, "uat boundary dirty after draw")

local homeLat, homeLon = test.bitmapLocalToLatLon(widget, 120, 136)
local aircraftLat, aircraftLon = test.bitmapLocalToLatLon(widget, 360, 136)
widget.homeLat = homeLat
widget.homeLon = homeLon
widget.homeX = 120
widget.homeY = 136
widget.prevHomeDistance = 1
widget.altSrc = {
  value = function()
    return 20
  end,
}
local currentLat = aircraftLat
local currentLon = aircraftLon
local warningTones = 0
local warningHaptics = 0
_G.system.playTone = function(...)
  warningTones = warningTones + 1
  return true
end
_G.system.playHaptic = function(_)
  warningHaptics = warningHaptics + 1
  return true
end
_G.system.getSource = function(query)
  if query and query.name == "GPS" and query.options == _G.OPTION_LATITUDE then
    return {
      value = function()
        return currentLat
      end,
      age = function()
        return 0
      end,
    }
  end
  if query and query.name == "GPS" and query.options == _G.OPTION_LONGITUDE then
    return {
      value = function()
        return currentLon
      end,
      age = function()
        return 0
      end,
    }
  end
  if query and query.name == "Alt" then
    return widget.altSrc
  end
  return nil
end
registeredWidget.wakeup(widget)
assert_true(type(widget.aircraftX) == "number" and widget.aircraftX > 300, "uat telemetry updates aircraft x")
assert_true(widget.warningActive, "uat crossing activates boundary warning")
assert_equal(warningTones, 1, "uat audio warning emitted")
assert_equal(warningHaptics, 1, "uat haptic warning emitted")
assert_true(type(widget.lastCoordsText) == "string" and widget.lastCoordsText:find("39.", 1, true) ~= nil, "uat coordinates updated")
assert_true(type(widget.distText) == "string" and widget.distText:find("Distance:", 1, true) ~= nil, "uat distance updated")
resetDrawCalls()
registeredWidget.paint(widget)
assert_shadowed_text("Boundary exceeded", 4, 18, "uat warning overlay")

local uatSaveRects = test.controlRects(widget)
ioWriteCounts["/scripts/BoundryMap/assets/maps/UATMap.boundries.json"] = 0
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, uatSaveRects.save.left + 3, rawTouchY(uatSaveRects.save.top + 3)), "uat save control start")
assert_equal(ioWriteCounts["/scripts/BoundryMap/assets/maps/UATMap.boundries.json"], 1, "uat save writes sidecar")
assert_true(not widget.boundaryDirty, "uat save clears dirty state")
local uatPayload = ioWrites["/scripts/BoundryMap/assets/maps/UATMap.boundries.json"]
assert_true(type(uatPayload) == "string" and uatPayload:find('"mapFile":"UATMap.bmp"', 1, true) ~= nil, "uat sidecar records map file")
ioReads["/scripts/BoundryMap/assets/maps/UATMap.boundries.json"] = uatPayload
local uatRestored = test.create()
uatRestored.bitmapFile = "UATMap.bmp"
registeredWidget.paint(uatRestored)
assert_equal(#uatRestored.boundaries, 1, "uat restored sidecar boundary")
registeredWidget.write(widget)
local uatStoredConfig = storageValues.cfg
assert_true(type(uatStoredConfig) == "string" and uatStoredConfig:find("UATMap.bmp", 1, true) ~= nil, "uat writes widget config")
storageValues.cfg = uatStoredConfig
local uatConfigRestored = test.create()
registeredWidget.read(uatConfigRestored)
assert_equal(uatConfigRestored.bitmapFile, "UATMap.bmp", "uat reads widget config map")
assert_equal(uatConfigRestored.gpsSensorName, "GPS", "uat reads widget config gps")
assert_true(uatConfigRestored.coordsEnabled, "uat reads widget config coordinates")
assert_true(uatConfigRestored.distEnabled, "uat reads widget config distance")

print("boundrymap lua tests passed")
