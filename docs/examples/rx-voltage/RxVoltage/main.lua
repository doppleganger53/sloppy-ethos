-- ETHOS 1.6.6 example: the model's receiver-voltage sensor is named RxBatt.
local function create()
  return { text = "-", nextPoll = 0 }
end

local function wakeup(widget)
  local now = os.clock()
  if now < widget.nextPoll then return end
  widget.nextPoll = now + 0.5
  local source = system.getSource({ name = "RxBatt" })
  local voltage = source and source:state() and source:value()
  local text = type(voltage) == "number" and string.format("%.1f V", voltage) or "-"
  if widget.text ~= text then
    widget.text = text
    lcd.invalidate()
  end
end

local function paint(widget)
  lcd.font(FONT_XL)
  lcd.drawText(8, 8, widget.text)
end

local function init()
  system.registerWidget({
    key = "rxvolt",
    name = function() return "RX Voltage" end,
    create = create,
    wakeup = wakeup,
    paint = paint,
  })
end

return { init = init }
