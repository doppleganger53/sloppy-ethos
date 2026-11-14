-- Run from the repo root against the widget that will be packaged.
local definition, sensor, text, font
local now = 0
os.clock = function() return now end
FONT_XL = 10
FONT_XXL = 20
local expectedFont = arg[1] == "larger" and FONT_XXL or FONT_XL
system = {
  registerWidget = function(widget) definition = widget end,
  getSource = function(query)
    assert(query.name == "RxBatt")
    return sensor
  end,
}
lcd = {
  invalidate = function() end,
  font = function(value) font = value end,
  drawText = function(_, _, value) text = value end,
}

dofile("scripts/RxVoltage/main.lua").init()
assert(definition.name() == "RX Voltage")
local widget = definition.create()
local function observe(source, expected)
  sensor = source
  now = now + 1
  text, font = nil, nil
  definition.wakeup(widget)
  definition.paint(widget)
  assert(text == expected, string.format("Expected %s; got %s", expected, tostring(text)))
  assert(font == expectedFont, "Unexpected display font")
  print(definition.name() .. ": " .. text)
end

observe(nil, "-")
local function reading(active, value)
  return { state = function() return active end, value = function() return value end }
end
observe(reading(true, 5.2), "5.2 V")
observe(reading(true, 0), "0.0 V")
observe(reading(false, 5.2), "-")
observe(nil, "-")
