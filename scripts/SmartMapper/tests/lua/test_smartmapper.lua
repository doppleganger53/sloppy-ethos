_G.__SMARTMAPPER_TEST__ = true
_G.FONT_XS = 0
_G.FONT_STD = 0
_G.FONT_STD_BOLD = 0
_G.EVT_TOUCH = 1
_G.TOUCH_START = 16640
_G.TOUCH_END = 16641
_G.TOUCH_MOVE = 16642
_G.EVT_VIRTUAL_NEXT = 200
_G.EVT_VIRTUAL_PREV = 201
_G.EVT_ENTER_BREAK = 202
_G.CATEGORY_ANALOG = 8
_G.CATEGORY_SWITCH = 9
_G.CATEGORY_SWITCH_POSITION = 10
_G.CATEGORY_FUNCTION_SWITCH = 12
_G.CATEGORY_LOGIC_SWITCH = 13
_G.CATEGORY_TRIM = 14
_G.CATEGORY_TRIM_POSITION = 15
_G.CATEGORY_CHANNEL = 16
_G.CATEGORY_TIMER = 21
_G.CATEGORY_TELEMETRY_SENSOR = 22

local registeredWidget = nil
local drawCalls = {}
local invalidated = false
local loadedIconPath = nil
local realPrint = print

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

local function contains_row(rows, needle)
  for _, row in ipairs(rows) do
    for _, value in pairs(row) do
      if tostring(value):find(needle, 1, true) then
        return true
      end
    end
  end
  return false
end

local function contains_assignment(mapping, control, target)
  for _, assignment in ipairs(mapping.assignments) do
    if assignment.controlLabel == control and assignment.target == target then
      return true
    end
  end
  return false
end

local function assert_source_used(mapping, label, expected, context)
  local source = mapping.sourceByKey[label:lower()]
  assert_true(source ~= nil, "source exists: " .. label)
  assert_equal(source.used, expected, context or ("source used: " .. label))
end

local function sourceHandle(label)
  return {
    name = function()
      return label
    end,
  }
end

_G.system = {
  registerWidget = function(spec)
    registeredWidget = spec
  end,
  getSource = function(query)
    if query.category == _G.CATEGORY_SWITCH_POSITION and query.member == 5 then
      return sourceHandle("SD up")
    end
    return nil
  end,
  getSources = function(category)
    assert_true(type(category) == "number", "getSources receives category number")
    if category == _G.CATEGORY_SWITCH_POSITION then
      return { "SA up", "SA mid", "SA down", "SB up", "SC up", "SD up" }
    elseif category == _G.CATEGORY_FUNCTION_SWITCH then
      return { "FS1", "FS2" }
    elseif category == _G.CATEGORY_ANALOG then
      return { "Throttle" }
    elseif category == _G.CATEGORY_CHANNEL then
      return { "CH1" }
    end
    return {}
  end,
}

_G.model = {
  name = function()
    return "Mapping Model"
  end,
  getMixes = function()
    return {
      { name = "Gear", input = "SA up" },
      { name = "Throttle mix", source = "Throttle", weight = 80 },
      { name = "Mode", input = { category = _G.CATEGORY_SWITCH_POSITION, member = 5 } },
    }
  end,
  getLogicSwitch = function(index)
    if index == 0 then
      return {
        name = "L01",
        operation = "a>x",
        values = function()
          return { sourceHandle("SB up"), sourceHandle("SC up") }
        end,
      }
    end
    return nil
  end,
  getChannel = function(index)
    if index == 0 then
      return { name = "Aileron" }
    end
    return nil
  end,
  getSpecialFunctions = function()
    return {
      { switch = "SA down", action = { type = "playFile", file = "gear_down.wav" } },
      { activeCondition = "SA mid", action = { type = "playText", text = "mid mode" } },
    }
  end,
}

_G.lcd = {
  getWindowSize = function()
    return 480, 90
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
assert_true(type(registeredWidget) == "table", "widget registers")
assert_equal(registeredWidget.key, "smrtmpr", "widget key")
assert_equal(registeredWidget.name(), "SmartMapper", "widget name")
assert_true(type(registeredWidget.create) == "function", "create registered")
assert_true(type(registeredWidget.paint) == "function", "paint registered")
assert_true(type(registeredWidget.wakeup) == "function", "wakeup registered")
assert_true(type(registeredWidget.event) == "function", "event registered")
assert_equal(loadedIconPath, "/scripts/SmartMapper/smartmapper.png", "icon loads from SmartMapper path first")

local mapping = test.buildMapping()
assert_equal(mapping.modelName, "Mapping Model", "model name captured")
assert_true(#mapping.sources >= 8, "source inventory captured")
assert_true(#mapping.assignments >= 6, "assignments captured")
assert_true(contains_assignment(mapping, "SA up", "Gear"), "mix input mapped")
assert_true(contains_assignment(mapping, "Throttle", "Throttle mix"), "mix source mapped")
assert_true(contains_assignment(mapping, "SD up", "Mode"), "source query mix input mapped")
assert_true(contains_assignment(mapping, "SB up", "L01"), "logic switch values source mapped")
assert_true(contains_assignment(mapping, "SC up", "L01"), "second logic switch values source mapped")
assert_true(contains_assignment(mapping, "SA down", "gear_down.wav"), "special function audio target mapped")
assert_true(contains_assignment(mapping, "SA mid", "mid mode"), "special function text target mapped")
assert_source_used(mapping, "SB up", true, "logic switch values mark SB up used")
assert_source_used(mapping, "SC up", true, "logic switch values mark SC up used")
assert_true(not contains_assignment(mapping, "Aileron", "Aileron"), "channel name-only record is not mapped")

local rows = test.buildRows(mapping)
assert_true(contains_row(rows, "Assigned controls"), "assigned section rendered")
assert_true(contains_row(rows, "Gear"), "mix name used as target")
assert_true(contains_row(rows, "L01"), "logic switch name used as target")
assert_true(not contains_row(rows, "Aileron"), "channel name-only record does not render as assignment")
assert_true(contains_row(rows, "gear_down.wav"), "special function audio label used")
assert_true(contains_row(rows, "mid mode"), "special function text label used")
assert_true(contains_row(rows, "Unused switches"), "unused section rendered")
assert_true(contains_row(rows, "SD up"), "unused switch position rendered")
assert_true(contains_row(rows, "FS1"), "unused function switch rendered")
assert_true(contains_row(rows, "Status"), "API status rows rendered for unavailable surfaces")

local widget = test.create()
registeredWidget.paint(widget)
assert_true(#drawCalls > 4, "paint renders rows")
assert_equal(drawCalls[1].text, "SmartMapper", "title should render")

local before = widget.scroll
assert_true(test.event(widget, nil, _G.EVT_VIRTUAL_NEXT), "next event scrolls")
assert_true(widget.scroll > before, "scroll increased")
assert_true(invalidated, "scroll invalidates display")

invalidated = false
assert_true(test.event(widget, _G.EVT_TOUCH, _G.TOUCH_START, 20, 60), "touch start handled")
test.event(widget, _G.EVT_TOUCH, _G.TOUCH_MOVE, 20, 20)
assert_true(widget.scroll > before, "touch drag scrolls")
assert_true(invalidated, "touch drag invalidates display")
assert_true(test.event(widget, _G.EVT_TOUCH, _G.TOUCH_END, 20, 20), "touch end handled")

_G.model = {}
_G.system.getSources = function()
  return {}
end

local emptyMapping = test.buildMapping()
local emptyRows = test.buildRows(emptyMapping)
assert_true(contains_row(emptyRows, "No accessible model mappings found."), "empty state rendered")

realPrint("smartmapper lua tests passed")
