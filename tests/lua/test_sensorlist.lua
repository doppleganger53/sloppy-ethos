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

local function makeRefreshWidget()
  return {
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
    needsInvalidate = false,
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
    showValue = false,
    debugRefreshCount = 0,
    debugDeepScanCount = 0,
    debugCachedScanCount = 0,
    debugDeferredScanCount = 0,
  }
end

local module = dofile("scripts/SensorList/main.lua")
local test = module._test

assert_true(type(test) == "table", "expected _test export table")

assert_equal(test.toInt("0x0A"), 10, "hex string parse")
assert_equal(test.toInt("15.9"), 15, "decimal floor")
assert_equal(test.toInt(""), nil, "empty string parse")
assert_equal(test.formatHex(10, 2), "0A", "hex formatting")
assert_equal(test.formatHex(nil, 2), "--", "nil formatting")

local methodBackedCandidate = {
  subId = function(_self)
    return "0010"
  end,
}
assert_equal(test.readCandidate(methodBackedCandidate, { "subId" }), "0010", "function-valued candidate is invoked")

local guardedCandidate = setmetatable({}, {
  __index = function(_table, key)
    if key == "subId" then
      error("direct accessor unsupported")
    end
    return nil
  end,
})
assert_equal(test.readCandidate(guardedCandidate, { "subId" }), nil, "candidate accessor errors fail soft")

local createLogs = {}
local savedSystem = _G.system
local savedPrint = _G.print
_G.print = function(message)
  createLogs[#createLogs + 1] = tostring(message)
end
_G.system = {
  registerWidget = function(_) end,
  getSensors = function()
    return setmetatable({}, {
      __len = function()
        error("len fault")
      end,
    })
  end,
}
local recoveredWidget = test.create()
_G.system = savedSystem
_G.print = savedPrint
assert_true(type(recoveredWidget) == "table", "create returns widget when refresh errors")
assert_true(type(recoveredWidget.lastError) == "string", "create stores refresh error")
assert_true(recoveredWidget.lastError:find("create", 1, true) ~= nil, "create error context is retained")
assert_true(#createLogs >= 1 and createLogs[1]:find("SLERR", 1, true) ~= nil, "create error is logged to serial")

local sensors = test.normalizeSensors({
  { name = "Gamma", physicalId = "0A", applicationId = "6801", subId = "0000", value = 7.25 },
  { name = "Alpha", physicalId = "00", applicationId = "0001", subId = "0001", stringValue = "12.5V" },
  { name = "Beta", physicalId = "00", applicationId = "0001", subId = "0000" },
  { name = "Delta", physicalId = "00", applicationId = "0001", subId = "0000" },
})

assert_equal(#sensors, 4, "normalized count")
assert_equal(sensors[1].name, "Beta", "sort by subId before name")
assert_equal(sensors[2].name, "Delta", "same triplet sorts by name")
assert_equal(sensors[3].name, "Alpha", "higher subId sorts later")
assert_equal(sensors[4].name, "Gamma", "higher physical id sorts later")
assert_equal(sensors[1].subIdText, "0000", "subId formatting")
assert_equal(sensors[3].subIdText, "0001", "subId tie breaker retained")
assert_equal(sensors[3].valueText, "12.5V", "string values are preserved")
assert_equal(sensors[4].valueText, "7.25", "numeric values are compact-formatted")

local groups = test.buildConflictGroups(sensors)
assert_true(groups["00|0001|0000"] ~= nil, "duplicate conflict triplet grouped")
assert_true(groups["00|0001|0001"] == nil, "different subId should not group")

local sig1 = test.buildSignature(sensors)
local sigStable = test.buildSignature(sensors)
assert_equal(sig1, sigStable, "stable signatures")
local sensorsWithChangedValue = test.normalizeSensors({
  { name = "Gamma", physicalId = "0A", applicationId = "6801", subId = "0000", value = 8.25 },
  { name = "Alpha", physicalId = "00", applicationId = "0001", subId = "0001", stringValue = "12.5V" },
  { name = "Beta", physicalId = "00", applicationId = "0001", subId = "0000" },
  { name = "Delta", physicalId = "00", applicationId = "0001", subId = "0000" },
})
local sigChanged = test.buildSignature(sensorsWithChangedValue)
assert_true(sig1 ~= sigChanged, "signature includes value changes")

local large = {}
for i = 1, 40 do
  large[i] = {
    name = "Sensor " .. tostring(i),
    physical = i,
    application = i,
    subId = i,
    physicalText = string.format("%02X", i),
    applicationText = string.format("%04X", i),
    subIdText = string.format("%04X", i),
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
    { name = "Zulu", physicalId = "02", applicationId = "1000", subId = "0002" },
    { name = "Alpha", physicalId = "03", applicationId = "0001", subId = "0001" },
    { name = "Bravo", physicalId = "01", applicationId = "2000", subId = "0003" },
    { name = "Delta", physicalId = "01", applicationId = "2000", subId = "0001" },
  }),
  groups = {},
  colorCache = {},
  scrollOffset = 0,
  showValue = false,
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
assert_equal(sortableWidget.sensors[4].name, "Zulu", "name sort ascending last row")

local nameHeaderStart2 = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 10, 22)
local nameHeaderEnd2 = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 10, 22)
assert_true(nameHeaderStart2 and nameHeaderEnd2, "second name header tap should be consumed")
assert_true(sortableWidget.sortDescending == true, "second tap toggles to descending")
assert_equal(sortableWidget.sensors[1].name, "Zulu", "name sort descending first row")
assert_equal(sortableWidget.sensors[4].name, "Alpha", "name sort descending last row")

local physicalHeaderStart = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 300, 22)
local physicalHeaderEnd = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 300, 22)
assert_true(physicalHeaderStart and physicalHeaderEnd, "physical header tap should be consumed")
assert_equal(sortableWidget.sortKey, "physical", "physical sort key selected")
assert_true(sortableWidget.sortDescending == false, "new key resets to ascending")
assert_equal(sortableWidget.sensors[1].physicalText, "01", "physical sort ascending first row")
assert_equal(sortableWidget.sensors[4].physicalText, "03", "physical sort ascending last row")
assert_equal(sortableWidget.sensors[1].subIdText, "0001", "physical sort uses subId as final tie breaker")
assert_equal(sortableWidget.sensors[2].subIdText, "0003", "physical sort keeps higher subId later")

local applicationHeaderStart = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 390, 22)
local applicationHeaderEnd = test.event(sortableWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 390, 22)
assert_true(applicationHeaderStart and applicationHeaderEnd, "application header tap should be consumed")
assert_equal(sortableWidget.sortKey, "application", "application sort key selected")
assert_true(sortableWidget.sortDescending == false, "application sort defaults to ascending")
assert_equal(sortableWidget.sensors[1].applicationText, "0001", "application sort ascending first row")
assert_equal(sortableWidget.sensors[4].applicationText, "07D0", "application sort ascending last row")
assert_equal(sortableWidget.sensors[3].subIdText, "0001", "application tie keeps lower subId first")
assert_equal(sortableWidget.sensors[4].subIdText, "0003", "application tie keeps higher subId later")

local expandedHitboxWidget = {
  sensors = test.normalizeSensors({
    { name = "Zulu", physicalId = "02", applicationId = "1000", subId = "0002" },
    { name = "Alpha", physicalId = "03", applicationId = "0001", subId = "0001" },
    { name = "Bravo", physicalId = "01", applicationId = "2000", subId = "0003" },
  }),
  groups = {},
  colorCache = {},
  scrollOffset = 0,
  showValue = true,
  touchActive = false,
  touchLastY = nil,
  touchAccumY = 0,
  needsInvalidate = false,
}

local valueLayoutHitboxes = test.getColumnLayout(480, true)
local physicalExpandedX = valueLayoutHitboxes[2].x + 20
local applicationExpandedX = valueLayoutHitboxes[3].x + 20
local subIdExpandedX = valueLayoutHitboxes[4].x + 20

local physicalExpandedStart = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, physicalExpandedX, 12)
local physicalExpandedEnd = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, physicalExpandedX, 12)
assert_true(physicalExpandedStart and physicalExpandedEnd, "expanded top-left physical zone should be consumed")
assert_equal(expandedHitboxWidget.sortKey, "physical", "expanded top-left physical zone should sort physical column")

local applicationExpandedStart = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, applicationExpandedX, 45)
local applicationExpandedEnd = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, applicationExpandedX, 45)
assert_true(applicationExpandedStart and applicationExpandedEnd, "expanded bottom application zone should be consumed")
assert_equal(expandedHitboxWidget.sortKey, "application", "expanded bottom application zone should sort application column")

local subIdHeaderStart = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, subIdExpandedX, 22)
local subIdHeaderEnd = test.event(expandedHitboxWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, subIdExpandedX, 22)
assert_true(subIdHeaderStart and subIdHeaderEnd, "subId header touch should still be consumed as touch input")
assert_equal(expandedHitboxWidget.sortKey, "application", "subId header should not change sort key")

local canceledHeaderWidget = {
  sensors = test.normalizeSensors({
    { name = "Zulu", physicalId = "02", applicationId = "1000", subId = "0002" },
    { name = "Alpha", physicalId = "03", applicationId = "0001", subId = "0001" },
  }),
  groups = {},
  colorCache = {},
  scrollOffset = 0,
  showValue = false,
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

local storageReads = { displayValue = 1 }
local storageWrites = {}
_G.storage = {
  read = function(key)
    return storageReads[key]
  end,
  write = function(key, value)
    storageWrites[key] = value
  end,
}

local persistedWidget = makeRefreshWidget()
persistedWidget.showValue = false
persistedWidget.needsInvalidate = false
test.read(persistedWidget)
assert_true(persistedWidget.showValue == true, "read loads persisted show-value option")
assert_true(persistedWidget.needsInvalidate == true, "read invalidates after loading persisted option")
test.write(persistedWidget)
assert_equal(storageWrites.displayValue, 1, "write persists show-value option")

local capturedChoices = {}
_G.form = {
  addLine = function(label)
    return { label = label }
  end,
  addChoiceField = function(line, _unused, choices, getter, setter)
    capturedChoices.line = line
    capturedChoices.choices = choices
    capturedChoices.getter = getter
    capturedChoices.setter = setter
  end,
}
local configuredWidget = makeRefreshWidget()
test.configure(configuredWidget)
assert_equal(capturedChoices.line.label, "Display Value", "configure adds display-value line")
assert_equal(capturedChoices.getter(), 0, "configure getter reflects current option state")
capturedChoices.setter(1)
assert_true(configuredWidget.showValue == true, "configure setter enables show-value option")
assert_true(configuredWidget.needsInvalidate == true, "configure setter invalidates widget")

local layoutOff = test.getColumnLayout(480, false)
local layoutOn = test.getColumnLayout(480, true)
assert_equal(#layoutOff, 4, "baseline layout has four columns")
assert_equal(#layoutOn, 5, "show-value layout adds a fifth column")
assert_true(layoutOn[2].x < layoutOff[2].x, "show-value layout compresses id columns")

local drawnText = {}
local savedDrawText = _G.lcd.drawText
_G.lcd.drawText = function(x, y, text)
  drawnText[#drawnText + 1] = { x = x, y = y, text = text }
end
local renderWidget = makeRefreshWidget()
renderWidget.showValue = true
renderWidget.sensors = test.normalizeSensors({
  {
    name = "Telemetry Name That Is Long",
    physicalId = "01",
    applicationId = "1000",
    subId = "0001",
    stringValue = "123.456789V",
  },
})
test.drawSensorRows(renderWidget)
_G.lcd.drawText = savedDrawText

local sawValueHeader = false
local sawTruncatedValue = false
for _, call in ipairs(drawnText) do
  if call.text == "Value" then
    sawValueHeader = true
  end
  if call.text == "123.45..." then
    sawTruncatedValue = true
  end
end
assert_true(sawValueHeader, "show-value paint renders value header")
assert_true(sawTruncatedValue, "show-value paint truncates long value text")

local hapticCalls = {}
local hapticRefreshCount = 0
_G.system = {
  getSensors = function()
    hapticRefreshCount = hapticRefreshCount + 1
    return {
      { name = "From long press", physicalId = "01", applicationId = "1001" },
    }
  end,
  playHaptic = function(pattern)
    hapticCalls[#hapticCalls + 1] = pattern
  end,
}
local hapticWidget = makeRefreshWidget()
local hapticStart = test.event(hapticWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_FIRST, 12, 120)
hapticWidget.touchStartClock = os.clock() - 1
local hapticLong = test.event(hapticWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_LONG, 12, 120)
local hapticEnd = test.event(hapticWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_BREAK, 12, 120)
assert_true(hapticStart and hapticLong and hapticEnd, "long-press gesture path should be consumed")
assert_equal(hapticRefreshCount, 1, "explicit long press should refresh once")
assert_equal(#hapticCalls, 1, "explicit long press should trigger one haptic call")
assert_equal(hapticCalls[1], 200, "numeric haptic call should be first choice")
assert_equal(#hapticWidget.sensors, 1, "long press should refresh sensors")

local toneCalls = {}
local toneRefreshCount = 0
_G.system = {
  getSensors = function()
    toneRefreshCount = toneRefreshCount + 1
    return {
      { name = "From tone fallback", physicalId = "02", applicationId = "1002" },
    }
  end,
  playTone = function(freq, duration, pause)
    toneCalls[#toneCalls + 1] = { freq = freq, duration = duration, pause = pause }
  end,
}
local toneWidget = makeRefreshWidget()
local toneLong = test.event(toneWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_LONG, 18, 140)
assert_true(toneLong, "long press should consume when using tone fallback")
assert_equal(toneRefreshCount, 1, "tone fallback long press should refresh once")
assert_equal(#toneCalls, 1, "tone fallback should play one tone")
assert_equal(toneCalls[1].freq, 1800, "tone fallback frequency")
assert_equal(toneCalls[1].duration, 80, "tone fallback duration")
assert_equal(toneCalls[1].pause, 0, "tone fallback pause")

local noFeedbackRefreshCount = 0
_G.system = {
  getSensors = function()
    noFeedbackRefreshCount = noFeedbackRefreshCount + 1
    return {
      { name = "From silent refresh", physicalId = "03", applicationId = "1003" },
    }
  end,
}
local noFeedbackWidget = makeRefreshWidget()
local noFeedbackLong = test.event(noFeedbackWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_LONG, 24, 150)
assert_true(noFeedbackLong, "long press should be consumed without feedback APIs")
assert_equal(noFeedbackRefreshCount, 1, "silent long press should refresh once")
assert_equal(#noFeedbackWidget.sensors, 1, "silent long press should still refresh sensors")

local incrementalRefreshCalls = {}
_G.system = {
  getSource = function(request)
    incrementalRefreshCalls[#incrementalRefreshCalls + 1] = request.member
    if request.member == 15 then
      return {
        name = function()
          return "Queued refresh"
        end,
        physicalId = function()
          return 1
        end,
        applicationId = function()
          return 0x1001
        end,
        subId = function()
          return 0
        end,
      }
    end
    return nil
  end,
}
local incrementalWidget = makeRefreshWidget()
incrementalWidget.sourceCategory = 42
incrementalWidget.sourceMaxMember = 255
local incrementalLong = test.event(incrementalWidget, _G.EVT_TOUCH, _G.EVT_TOUCH_LONG, 30, 160)
assert_true(incrementalLong, "long press should queue incremental refresh")
assert_equal(incrementalWidget.sourceMaxMember, 32, "long press resets scan window to first expansion step")
assert_true(incrementalWidget.deepScanPending, "long press leaves deeper refresh queued")
assert_true(#incrementalWidget.sensors >= 1, "long press still refreshes first chunk")
assert_true(#incrementalRefreshCalls <= 20, "long press should not rescan the full category in one callback")

print("sensorlist lua tests passed")
