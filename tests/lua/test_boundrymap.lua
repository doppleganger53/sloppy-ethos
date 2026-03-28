_G.__BOUNDRYMAP_TEST__ = true
_G.RIGHT = 0
_G.FONT_XS = 0
_G.FONT_STD = 0
_G.FONT_STD_BOLD = 0
_G.FONT_BOLD = 0
_G.OPTION_LATITUDE = 1
_G.OPTION_LONGITUDE = 2
_G.EVT_TOUCH = 1
_G.EVT_TOUCH_FIRST = 100
_G.EVT_TOUCH_MOVE = 101
_G.EVT_TOUCH_BREAK = 102
_G.TOUCH_START = 16640
_G.TOUCH_MOVE = 16642
_G.TOUCH_END = 16641

local storageValues = {}
local ioReads = {}
local ioWrites = {}

_G.system = {
  registerWidget = function(_) end,
  getSource = function(_) return nil end,
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

_G.lcd = {
  RGB = function(r, g, b)
    return (r * 65536) + (g * 256) + b
  end,
  getWindowSize = function()
    return 480, 272
  end,
  color = function(_) end,
  font = function(_) end,
  drawText = function(_, _, _, _) end,
  drawFilledRectangle = function(_, _, _, _) end,
  drawRectangle = function(_, _, _, _) end,
  drawLine = function(_, _, _, _) end,
  drawFilledCircle = function(_, _, _) end,
  drawBitmap = function(_, _, _) end,
  loadBitmap = function(_)
    return {
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
      end
      function handle:close() end
      return handle
    end
    return nil
  end,
}

local function fail(message)
  io.stderr:write(message .. "\n")
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

local module = dofile("scripts/BoundryMap/main.lua")
local test = module._test
assert_true(type(test) == "table", "expected _test export table")

assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16640), "start", "raw touch start maps")
assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16642), "move", "raw touch move maps")
assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16641), "end", "raw touch end maps")

ioReads["/documents/user/TestMap.json"] = '{"topLat":39.78045886,"bottomLat":39.77254308,"leftLon":-75.21268129,"rightLon":-75.19585848}'
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
local savedPayload = ioWrites["/documents/user/TestMap.boundries.json"]
assert_true(type(savedPayload) == "string" and savedPayload:find('"schemaVersion":1', 1, true) ~= nil, "saved payload has schema version")

local parsed = test.parseBoundaryObjects(savedPayload)
assert_equal(#parsed, 1, "parse saved boundary payload")

ioReads["/documents/user/TestMap.boundries.json"] = savedPayload
widget.boundaries = {}
widget.savedBoundarySignature = ""
test.loadBoundaries(widget)
assert_equal(#widget.boundaries, 1, "load boundaries restores payload")

ioReads["/documents/user/TestMap.boundries.json"] = '{"schemaVersion":1,"mapFile":"TestMap.bmp","boundaries":[{"oops":1}]}'
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

widget = test.create()
widget.windowW = 480
widget.windowH = 272
widget.mapRect = { left = 0, top = 0, right = 480, bottom = 272 }
widget.loadedBitmap = true
widget.bmpW = 480
widget.bmpH = 272
widget.mapMeta = meta
local rects = test.controlRects(widget)
local drawX = rects.draw.left + 2
local drawY = rects.draw.top + 2
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, drawX, drawY), "draw button start consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, drawX, drawY), "draw button end consumed")
assert_true(widget.drawMode, "draw mode toggled on")

local startX = 20
local startY = 20
local endX = 120
local endY = 40
assert_true(test.event(widget, _G.EVT_TOUCH, 16640, startX, startY), "map draw start consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16642, 80, 30), "map draw move consumed")
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, endX, endY), "map draw end consumed")
assert_equal(#widget.boundaries, 1, "event draw flow adds boundary")

widget.deleteMode = true
widget.drawMode = false
assert_true(test.event(widget, _G.EVT_TOUCH, 16641, 22, 22), "delete touch consumed")
assert_equal(#widget.boundaries, 0, "event delete flow removes boundary")

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

print("boundrymap lua tests passed")
