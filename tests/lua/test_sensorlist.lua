_G.__SENSORLIST_TEST__ = true
_G.COLOR_WHITE = 0
_G.COLOR_BLACK = 1

_G.system = {
  registerWidget = function(_) end,
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
  drawText = function(_, _, _) end,
  drawFilledRectangle = function(_, _, _, _) end,
  isVisible = function()
    return true
  end,
  invalidate = function() end,
}

_G.model = nil
_G.EVT_TOUCH_FIRST = 100
_G.EVT_TOUCH_MOVE = 101
_G.EVT_TOUCH_BREAK = 102
_G.EVT_TOUCH_LONG = 103
_G.EVT_TOUCH = 1

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

local module = dofile("scripts/SensorList/main.lua")
local test = module._test

assert_true(type(test) == "table", "expected _test export table")

assert_equal(test.toInt("0x0A"), 10, "hex string parse")
assert_equal(test.toInt("15.9"), 15, "decimal floor")
assert_equal(test.toInt(""), nil, "empty string parse")
assert_equal(test.formatHex(10, 2), "0A", "hex formatting")
assert_equal(test.formatHex(nil, 2), "--", "nil formatting")

local sensors = test.normalizeSensors({
  { name = "Gamma", physicalId = "0A", applicationId = "6801" },
  { name = "Alpha", physicalId = "00", applicationId = "0001" },
  { name = "Beta", physicalId = "00", applicationId = "0000" },
})

assert_equal(#sensors, 3, "normalized count")
assert_equal(sensors[1].name, "Beta", "sort by application id")
assert_equal(sensors[2].name, "Alpha", "sort by application id second")
assert_equal(sensors[3].name, "Gamma", "sort by physical id")

local groups = test.buildPhysicalGroups(sensors)
assert_true(groups[0] ~= nil, "duplicate physical id grouped")
assert_true(groups[10] == nil, "unique physical id not grouped")

local sig1 = test.buildSignature(sensors)
local sig2 = test.buildSignature(sensors)
assert_equal(sig1, sig2, "stable signatures")

local large = {}
for i = 1, 40 do
  large[i] = {
    name = "Sensor " .. tostring(i),
    physical = i,
    application = i,
    physicalText = string.format("%02X", i),
    applicationText = string.format("%04X", i),
  }
end

local widget = { sensors = large, scrollOffset = 0 }
local moved = test.applyScroll(widget, 2)
assert_true(moved, "scroll should move")
test.clampOffset(widget)
assert_true(widget.scrollOffset >= 0, "clamped lower bound")

_G.system = {}
local empty, debug = test.getSensorList({}, false)
assert_equal(#empty, 0, "no-api empty list")
assert_equal(debug.strategy, "no-source-api", "no-api strategy")

local eventWidget = {
  sensors = large,
  scrollOffset = 0,
  touchActive = false,
  touchLastY = nil,
  touchAccumY = 0,
  needsInvalidate = false,
}

local consumedUnknown = test.event(eventWidget, 999, 0, 12, 12)
assert_true(consumedUnknown == false, "unknown category should not be treated as touch")
assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16640), "start", "raw touch start value maps")
assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16641), "end", "raw touch end value maps")
assert_equal(test.resolveTouchPhase(_G.EVT_TOUCH, 16642), "move", "raw touch move value maps")

local moveOnlyWidget = {
  sensors = large,
  scrollOffset = 0,
  touchActive = false,
  touchLastY = nil,
  touchAccumY = 0,
  needsInvalidate = false,
}
local consumedMoveWithoutStart = test.event(moveOnlyWidget, _G.EVT_TOUCH, 16642, 20, 220)
assert_true(consumedMoveWithoutStart == false, "move without start should be ignored")
assert_true(moveOnlyWidget.touchActive == false, "move without start should not activate touch session")

local consumedStart = test.event(eventWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 12, 120)
assert_true(consumedStart, "touch start should be consumed")
assert_true(eventWidget.touchActive, "touch session should activate")

local consumedMove = test.event(eventWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_MOVE, 12, -900)
assert_true(consumedMove, "touch move should be consumed")
assert_true(eventWidget.scrollOffset >= 0, "move should keep offset in valid range")
assert_true(math.abs(eventWidget.touchAccumY) < 16, "accumulator remains bounded")

local consumedEnd = test.event(eventWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 12, -900)
assert_true(consumedEnd, "touch end should be consumed")
assert_true(eventWidget.touchActive == false, "touch session should deactivate")

local sortableWidget = {
  sensors = test.normalizeSensors({
    { name = "Zulu", physicalId = "02", applicationId = "1000" },
    { name = "Alpha", physicalId = "03", applicationId = "0001" },
    { name = "Bravo", physicalId = "01", applicationId = "2000" },
  }),
  groups = {},
  colorCache = {},
  scrollOffset = 0,
  touchActive = false,
  touchLastY = nil,
  touchAccumY = 0,
  needsInvalidate = false,
}

local nameHeaderStart = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 10, 22)
local nameHeaderEnd = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 10, 22)
assert_true(nameHeaderStart and nameHeaderEnd, "name header tap should be consumed")
assert_equal(sortableWidget.sortKey, "name", "name sort key selected")
assert_true(sortableWidget.sortDescending == false, "name sort defaults to ascending")
assert_equal(sortableWidget.sensors[1].name, "Alpha", "name sort ascending first row")
assert_equal(sortableWidget.sensors[3].name, "Zulu", "name sort ascending last row")

local nameHeaderStart2 = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 10, 22)
local nameHeaderEnd2 = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 10, 22)
assert_true(nameHeaderStart2 and nameHeaderEnd2, "second name header tap should be consumed")
assert_true(sortableWidget.sortDescending == true, "second tap toggles to descending")
assert_equal(sortableWidget.sensors[1].name, "Zulu", "name sort descending first row")
assert_equal(sortableWidget.sensors[3].name, "Alpha", "name sort descending last row")

local physicalHeaderStart = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 300, 22)
local physicalHeaderEnd = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 300, 22)
assert_true(physicalHeaderStart and physicalHeaderEnd, "physical header tap should be consumed")
assert_equal(sortableWidget.sortKey, "physical", "physical sort key selected")
assert_true(sortableWidget.sortDescending == false, "new key resets to ascending")
assert_equal(sortableWidget.sensors[1].physicalText, "01", "physical sort ascending first row")
assert_equal(sortableWidget.sensors[3].physicalText, "03", "physical sort ascending last row")

local applicationHeaderStart = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 390, 22)
local applicationHeaderEnd = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 390, 22)
assert_true(applicationHeaderStart and applicationHeaderEnd, "application header tap should be consumed")
assert_equal(sortableWidget.sortKey, "application", "application sort key selected")
assert_true(sortableWidget.sortDescending == false, "application sort defaults to ascending")
assert_equal(sortableWidget.sensors[1].applicationText, "0001", "application sort ascending first row")
assert_equal(sortableWidget.sensors[3].applicationText, "07D0", "application sort ascending last row")

local expandedHitboxWidget = {
  sensors = test.normalizeSensors({
    { name = "Zulu", physicalId = "02", applicationId = "1000" },
    { name = "Alpha", physicalId = "03", applicationId = "0001" },
    { name = "Bravo", physicalId = "01", applicationId = "2000" },
  }),
  groups = {},
  colorCache = {},
  scrollOffset = 0,
  touchActive = false,
  touchLastY = nil,
  touchAccumY = 0,
  needsInvalidate = false,
}

local physicalExpandedStart = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 260, 12)
local physicalExpandedEnd = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 260, 12)
assert_true(physicalExpandedStart and physicalExpandedEnd, "expanded top-left physical zone should be consumed")
assert_equal(expandedHitboxWidget.sortKey, "physical", "expanded top-left physical zone should sort physical column")

local applicationExpandedStart = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 390, 45)
local applicationExpandedEnd = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 390, 45)
assert_true(applicationExpandedStart and applicationExpandedEnd, "expanded bottom application zone should be consumed")
assert_equal(expandedHitboxWidget.sortKey, "application", "expanded bottom application zone should sort application column")

local canceledHeaderWidget = {
  sensors = test.normalizeSensors({
    { name = "Zulu", physicalId = "02", applicationId = "1000" },
    { name = "Alpha", physicalId = "03", applicationId = "0001" },
  }),
  groups = {},
  colorCache = {},
  scrollOffset = 0,
  touchActive = false,
  touchLastY = nil,
  touchAccumY = 0,
  needsInvalidate = false,
}
local canceledHeaderStart = test.event(canceledHeaderWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 10, 22)
local canceledHeaderMove = test.event(canceledHeaderWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_MOVE, 10, 48)
local canceledHeaderEnd = test.event(canceledHeaderWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 10, 48)
assert_true(canceledHeaderStart and canceledHeaderMove and canceledHeaderEnd, "header drag path should be consumed")
assert_equal(canceledHeaderWidget.sortKey, nil, "header drag should not trigger sort toggle")

_G.system = {
  getSensors = function()
    return {
      { name = "From long press", physicalId = "01", applicationId = "1001" },
    }
  end,
}
local consumedLong = test.event(eventWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_LONG, 12, 12)
assert_true(consumedLong, "long press should be consumed")
assert_equal(#eventWidget.sensors, 1, "long press should refresh sensors")

print("sensorlist lua tests passed")
