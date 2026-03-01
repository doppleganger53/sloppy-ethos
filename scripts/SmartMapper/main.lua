local WIDGET_NAME = "SmartMapper"
local WIDGET_KEY = "smrtmpr"
local FONT_HEADER = FONT_STD_BOLD or FONT_STD or FONT_XS
local FONT_BODY = FONT_XS or FONT_STD

local HEADER_Y = 0
local BODY_Y = 24

local function safeCall(fn, ...)
  if type(fn) ~= "function" then
    return nil
  end
  local ok, result = pcall(fn, ...)
  if ok then
    return result
  end
  return nil
end

local function drawTextSafe(x, y, text)
  if type(lcd) == "table" and type(lcd.drawText) == "function" then
    safeCall(lcd.drawText, x, y, text)
  end
end

local function setFontSafe(font)
  if type(lcd) == "table" and type(lcd.font) == "function" and font then
    safeCall(lcd.font, font)
  end
end

local function create()
  return {
    status = "Deferred pending Ethos 1.7.x final API validation.",
    detail = "Ethos 1.6.4 widget runtime does not expose mapping APIs.",
  }
end

local function wakeup(widget)
end

local function paint(widget)
  if type(widget) ~= "table" then
    return
  end

  setFontSafe(FONT_HEADER)
  drawTextSafe(4, HEADER_Y, WIDGET_NAME)

  setFontSafe(FONT_BODY)
  drawTextSafe(4, BODY_Y, widget.status or "Deferred pending Ethos 1.7.x final API validation.")
  drawTextSafe(4, BODY_Y + 16, widget.detail or "Ethos 1.6.4 widget runtime does not expose mapping APIs.")
end

local function init()
  if type(system) ~= "table" or type(system.registerWidget) ~= "function" then
    return
  end

  safeCall(system.registerWidget, {
    key = WIDGET_KEY,
    name = function()
      return WIDGET_NAME
    end,
    create = create,
    wakeup = wakeup,
    paint = paint,
    persistent = false,
  })
end

local function testExports()
  return {
    create = create,
    paint = paint,
  }
end

local module = { init = init }
if rawget(_G, "__SMARTMAPPER_TEST__") then
  module._test = testExports()
end

return module
