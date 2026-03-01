_G.__SMARTMAPPER_TEST__ = true

local registered = nil

_G.system = {
  registerWidget = function(definition)
    registered = definition
  end,
}

local drawCalls = {}
local fontCalls = {}

_G.lcd = {
  font = function(font)
    fontCalls[#fontCalls + 1] = font
  end,
  drawText = function(x, y, text)
    drawCalls[#drawCalls + 1] = { x = x, y = y, text = text }
  end,
}

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

local module = dofile("scripts/SmartMapper/main.lua")
local test = module._test

assert_true(type(test) == "table", "expected _test export table")

module.init()
assert_true(type(registered) == "table", "widget should register")
assert_equal(registered.key, "smrtmpr", "widget key should use short form")
assert_true(type(registered.create) == "function", "create callback should be registered")
assert_true(type(registered.paint) == "function", "paint callback should be registered")

local widget = test.create()
assert_equal(widget.status, "Deferred pending Ethos 1.7.x final API validation.", "status should explain defer")
assert_equal(widget.detail, "Ethos 1.6.4 widget runtime does not expose mapping APIs.", "detail should explain runtime limit")

test.paint(widget)
assert_true(#drawCalls >= 3, "paint should draw title and two message lines")
assert_equal(drawCalls[1].text, "SmartMapper", "title should render")
assert_equal(drawCalls[2].text, widget.status, "status line should render")
assert_equal(drawCalls[3].text, widget.detail, "detail line should render")

print("smartmapper lua tests passed")
