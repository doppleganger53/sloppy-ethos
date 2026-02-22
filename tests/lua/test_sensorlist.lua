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

local module = dofile("src/scripts/SensorList/main.lua")
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

print("sensorlist lua tests passed")
