_G.__SMARTMAPPER_TEST__ = true
_G.FONT_XS = 0
_G.FONT_STD = 0
_G.FONT_STD_BOLD = 0
_G.EVT_TOUCH = 1
_G.TOUCH_END = 16641
_G.CATEGORY_NONE = 0
_G.CATEGORY_ALWAYS_ON = 1
_G.CATEGORY_ANALOG = 43
_G.CATEGORY_SWITCH = 44
_G.CATEGORY_SWITCH_POSITION = 42
_G.CATEGORY_FUNCTION_SWITCH = 45
_G.CATEGORY_LOGIC_SWITCH = 46
_G.CATEGORY_TRIM = 47
_G.CATEGORY_TRIM_POSITION = 48
_G.CATEGORY_CHANNEL = 49
_G.CATEGORY_GYRO = 50
_G.CATEGORY_GYRO_SWITCH = 51
_G.CATEGORY_TRAINER = 52
_G.CATEGORY_FLIGHT = 53
_G.CATEGORY_FLIGHT_VALUE = 54
_G.CATEGORY_TIMER = 55
_G.CATEGORY_TELEMETRY_SENSOR = 56
_G.CATEGORY_SYSTEM = 57
_G.CATEGORY_SYSTEM_EVENT = 58
_G.CATEGORY_SPECIAL = 59

local registeredTool = nil
local printed = {}
local written = {}
local drawCalls = {}
local invalidated = false
local realPrint = print
local loadedIconPath = nil

local function contains(lines, needle)
  for _, line in ipairs(lines) do
    if tostring(line):find(needle, 1, true) then
      return true
    end
  end
  return false
end

local function fail(message)
  io.stderr:write(message .. "\n")
  os.exit(1)
end

local function assert_true(value, label)
  if not value then
    fail(label or "assert_true failed")
  end
end

local function assert_equal(actual, expected, label)
  if actual ~= expected then
    fail((label or "assert_equal failed") .. ": expected=" .. tostring(expected) .. " actual=" .. tostring(actual))
  end
end

_G.print = function(line)
  printed[#printed + 1] = tostring(line)
end

_G.system = {
  registerSystemTool = function(spec)
    registeredTool = spec
  end,
  getSources = function(category)
    assert_true(type(category) == "number", "getSources receives category number")
    if category == _G.CATEGORY_SWITCH_POSITION then
      return { "SAup", "SAmid", "SAdown" }
    end
    return {}
  end,
  getSource = function(query)
    return { category = query.category, member = query.member }
  end,
}

_G.model = {
  name = function()
    return "Probe Model"
  end,
  createMix = function()
    return true
  end,
  getMix = function(index)
    if index == 0 then
      return { name = "Throttle", input = "SA" }
    end
    return nil
  end,
  getLogicalSwitches = function()
    return { { name = "L01" } }
  end,
}

local realIo = io
_G.io = {
  stderr = realIo.stderr,
  open = function(path, mode)
    if mode ~= "w" then
      return nil
    end
    return {
      write = function(_, content)
        written[path] = content
      end,
      close = function() end,
    }
  end,
}

_G.lcd = {
  getWindowSize = function()
    return 480, 272
  end,
  font = function(_) end,
  drawText = function(x, y, text)
    drawCalls[#drawCalls + 1] = { x = x, y = y, text = text }
  end,
  loadMask = function(path)
    loadedIconPath = path
    return { name = path }
  end,
  invalidate = function()
    invalidated = true
  end,
}

local module = dofile("scripts/SmartMapper/main.lua")
local test = module._test
assert_true(type(test) == "table", "expected _test export table")

module.init()
assert_true(type(registeredTool) == "table", "tool registers")
assert_true(type(registeredTool.create) == "function", "create registered")
assert_true(type(registeredTool.paint) == "function", "paint registered")
assert_true(type(registeredTool.event) == "function", "event registered")
assert_true(type(registeredTool.icon) == "table", "icon registered")
assert_equal(loadedIconPath, "/scripts/SmartMapper/smartmapper.png", "icon loads from SmartMapper path first")
assert_equal(registeredTool.name(), "SmartMapper Probe", "tool name")

local lines, path = test.runProbe()
assert_true(type(lines) == "table" and #lines > 8, "probe returns report lines")
assert_equal(path, "/documents/SmartMapper-api-probe.txt", "first writable report path")
assert_true(written[path]:find("SmartMapper Ethos API Probe", 1, true) ~= nil, "report content written")
assert_true(contains(lines, "model.name(): Probe Model"), "model name captured")
assert_true(contains(lines, "model.createMix=function"), "createMix availability captured")
assert_true(contains(lines, "model.getMix(0/1): ok"), "mix read probe captured")
assert_true(contains(lines, "Source category constants:"), "category section captured")
assert_true(contains(lines, "- CATEGORY_SWITCH_POSITION=42"), "category value captured")
assert_true(contains(lines, "system.getSources CATEGORY_SWITCH_POSITION (42): ok"), "getSources category captured")
assert_true(contains(lines, "- mixes: candidate support"), "mix surface summarized")
assert_true(contains(lines, "- special functions: no candidate read/enumeration API found"), "missing special functions summarized")

local state = test.create()
assert_true(type(state.lines) == "table", "create stores report lines")
registeredTool.paint(state)
assert_true(#drawCalls > 2, "paint renders report")
assert_true(test.event(state, _G.EVT_TOUCH, _G.TOUCH_END), "touch end reruns probe")
assert_true(invalidated, "rerun invalidates display")

realPrint("smartmapper lua tests passed")
