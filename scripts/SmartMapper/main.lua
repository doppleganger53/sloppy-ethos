local TOOL_NAME = "SmartMapper Probe"
local ICON_PATHS = {
  "/scripts/SmartMapper/smartmapper.png",
  "/SCRIPTS/SmartMapper/smartmapper.png",
  "smartmapper.png",
}
local REPORT_PATHS = {
  "/documents/SmartMapper-api-probe.txt",
  "/scripts/SmartMapper/smartmapper-api-probe.txt",
  "smartmapper-api-probe.txt",
}

local LINE_HEIGHT = 14
local HEADER_HEIGHT = 34
local FONT_HEADER = rawget(_G, "FONT_STD_BOLD") or rawget(_G, "FONT_STD") or rawget(_G, "FONT_XS")
local FONT_BODY = rawget(_G, "FONT_XS") or rawget(_G, "FONT_STD")
local unpackArgs = table.unpack or unpack

local MODEL_API_CANDIDATES = {
  "name",
  "createMix",
  "getMix",
  "getMixes",
  "getMixCount",
  "mixes",
  "getLogicalSwitch",
  "getLogicalSwitches",
  "getLogicSwitch",
  "getLogicSwitches",
  "getTrim",
  "getTrims",
  "getSpecialFunction",
  "getSpecialFunctions",
  "getInput",
  "getInputs",
  "getSwitch",
  "getSwitches",
  "getChannel",
  "getChannels",
  "getOutput",
  "getOutputs",
}

local SYSTEM_API_CANDIDATES = {
  "getSource",
  "getSources",
  "getSensors",
  "registerSystemTool",
  "registerWidget",
}

local REQUIRED_SURFACES = {
  {
    key = "mixes",
    label = "mixes",
    candidates = { "getMixes", "getMix", "getMixCount", "mixes" },
  },
  {
    key = "logicalSwitches",
    label = "logical switches",
    candidates = { "getLogicalSwitches", "getLogicSwitches", "getLogicalSwitch", "getLogicSwitch" },
  },
  {
    key = "trims",
    label = "trims",
    candidates = { "getTrims", "getTrim" },
  },
  {
    key = "specialFunctions",
    label = "special functions",
    candidates = { "getSpecialFunctions", "getSpecialFunction" },
  },
  {
    key = "switchAssignments",
    label = "switch/input assignments",
    candidates = { "getSwitches", "getSwitch", "getInputs", "getInput", "getSources" },
  },
}

-- Category list comes from the Ethos 26.1.0-RC1 Lua docs release asset.
local SOURCE_CATEGORY_NAMES = {
  "CATEGORY_NONE",
  "CATEGORY_ALWAYS_ON",
  "CATEGORY_ANALOG",
  "CATEGORY_SWITCH",
  "CATEGORY_SWITCH_POSITION",
  "CATEGORY_FUNCTION_SWITCH",
  "CATEGORY_LOGIC_SWITCH",
  "CATEGORY_TRIM",
  "CATEGORY_TRIM_POSITION",
  "CATEGORY_CHANNEL",
  "CATEGORY_GYRO",
  "CATEGORY_GYRO_SWITCH",
  "CATEGORY_TRAINER",
  "CATEGORY_FLIGHT",
  "CATEGORY_FLIGHT_VALUE",
  "CATEGORY_TIMER",
  "CATEGORY_TELEMETRY_SENSOR",
  "CATEGORY_SYSTEM",
  "CATEGORY_SYSTEM_EVENT",
  "CATEGORY_SPECIAL",
}

local function safePrint(line)
  if type(print) == "function" then
    pcall(print, line)
  end
end

local function loadIcon()
  if type(lcd) ~= "table" or type(lcd.loadMask) ~= "function" then
    return nil
  end
  for _, path in ipairs(ICON_PATHS) do
    local ok, icon = pcall(lcd.loadMask, path)
    if ok and icon then
      return icon
    end
  end
  return nil
end

local icon = loadIcon()

local function safeGet(container, key)
  if container == nil then
    return nil
  end
  local ok, value = pcall(function()
    return container[key]
  end)
  if ok then
    return value
  end
  return nil
end

local function sortedKeys(container)
  local keys = {}
  if type(container) ~= "table" then
    return keys
  end

  local ok = pcall(function()
    for key, _ in pairs(container) do
      keys[#keys + 1] = tostring(key)
    end
  end)
  if not ok then
    return {}
  end
  table.sort(keys, function(a, b)
    return a:lower() < b:lower()
  end)
  return keys
end

local function join(list, separator)
  separator = separator or ", "
  local out = ""
  for index, value in ipairs(list) do
    if index > 1 then
      out = out .. separator
    end
    out = out .. tostring(value)
  end
  return out
end

local function summarizeValue(value)
  local valueType = type(value)
  if value == nil then
    return "nil"
  end
  if valueType == "table" then
    local count = 0
    local keys = {}
    local ok = pcall(function()
      for key, _ in pairs(value) do
        count = count + 1
        if #keys < 5 then
          keys[#keys + 1] = tostring(key)
        end
      end
    end)
    if ok then
      table.sort(keys)
      if #keys > 0 then
        return string.format("table(%d; keys=%s)", count, join(keys))
      end
      return string.format("table(%d)", count)
    end
    return "table(?)"
  end
  if valueType == "string" then
    if #value > 48 then
      return value:sub(1, 45) .. "..."
    end
    return value
  end
  return valueType .. "(" .. tostring(value) .. ")"
end

local function invoke(container, key, args)
  local fn = safeGet(container, key)
  if type(fn) ~= "function" then
    return false, "not callable: " .. type(fn)
  end

  args = args or {}
  local ok, result = pcall(fn, unpackArgs(args))
  if ok then
    return true, result
  end
  local firstError = result

  local selfArgs = { container }
  for _, arg in ipairs(args) do
    selfArgs[#selfArgs + 1] = arg
  end
  ok, result = pcall(fn, unpackArgs(selfArgs))
  if ok then
    return true, result
  end

  return false, firstError
end

local function describeCandidate(containerName, container, key)
  local value = safeGet(container, key)
  return string.format("%s.%s=%s", containerName, key, type(value))
end

local function listAvailable(containerName, container, candidates)
  local available = {}
  for _, key in ipairs(candidates) do
    local value = safeGet(container, key)
    if value ~= nil then
      available[#available + 1] = describeCandidate(containerName, container, key)
    end
  end
  return available
end

local function tryVariants(container, key, variants)
  for _, args in ipairs(variants) do
    local ok, result = invoke(container, key, args)
    if ok then
      return true, result, args
    end
  end
  return false, nil, nil
end

local function categoryQuery(name)
  local value = rawget(_G, name)
  if value == nil then
    return nil
  end
  return { category = value }
end

local function sourceCategories()
  local categories = {}
  for _, name in ipairs(SOURCE_CATEGORY_NAMES) do
    categories[#categories + 1] = {
      name = name,
      value = rawget(_G, name),
    }
  end
  return categories
end

local function appendCategoryConstants(lines)
  lines[#lines + 1] = "Source category constants:"
  for _, category in ipairs(sourceCategories()) do
    if type(category.value) == "number" then
      lines[#lines + 1] = string.format("- %s=%s", category.name, tostring(category.value))
    else
      lines[#lines + 1] = string.format("- %s unavailable (%s)", category.name, type(category.value))
    end
  end
end

local function appendSourceProbe(lines)
  if type(system) ~= "table" then
    lines[#lines + 1] = "system namespace: absent"
    return
  end

  appendCategoryConstants(lines)

  local getSources = safeGet(system, "getSources")
  lines[#lines + 1] = "system.getSources: " .. type(getSources)
  if type(getSources) == "function" then
    for _, category in ipairs(sourceCategories()) do
      if type(category.value) == "number" then
        local ok, result = invoke(system, "getSources", { category.value })
        lines[#lines + 1] = string.format(
          "system.getSources %s (%s): %s %s",
          category.name,
          tostring(category.value),
          ok and "ok" or "error",
          summarizeValue(result)
        )
      else
        lines[#lines + 1] = string.format(
          "system.getSources %s: skipped (%s)",
          category.name,
          type(category.value)
        )
      end
    end
  end

  local getSource = safeGet(system, "getSource")
  lines[#lines + 1] = "system.getSource: " .. type(getSource)
  if type(getSource) == "function" then
    for _, name in ipairs(SOURCE_CATEGORY_NAMES) do
      local query = categoryQuery(name)
      if query then
        query.member = 0
        local ok, result = invoke(system, "getSource", { query })
        lines[#lines + 1] = string.format("system.getSource %s member 0: %s %s", name, ok and "ok" or "error", summarizeValue(result))
      end
    end
  end
end

local function appendModelProbe(lines)
  if type(model) ~= "table" and type(model) ~= "userdata" then
    lines[#lines + 1] = "model namespace: absent"
    return
  end

  local modelName = "unavailable"
  if type(safeGet(model, "name")) == "function" then
    local ok, result = invoke(model, "name", {})
    if ok then
      modelName = summarizeValue(result)
    else
      modelName = "error: " .. tostring(result)
    end
  end
  lines[#lines + 1] = "model.name(): " .. modelName

  local keys = sortedKeys(model)
  if #keys > 0 then
    lines[#lines + 1] = "model enumerable keys: " .. join(keys)
  else
    lines[#lines + 1] = "model enumerable keys: none or not enumerable"
  end

  local available = listAvailable("model", model, MODEL_API_CANDIDATES)
  if #available > 0 then
    lines[#lines + 1] = "known model APIs: " .. join(available)
  else
    lines[#lines + 1] = "known model APIs: none from candidate list"
  end

  for _, key in ipairs({ "getMixes", "getLogicalSwitches", "getLogicSwitches", "getTrims", "getSpecialFunctions", "getInputs", "getSwitches", "getChannels" }) do
    if type(safeGet(model, key)) == "function" then
      local ok, result = invoke(model, key, {})
      lines[#lines + 1] = string.format("model.%s(): %s %s", key, ok and "ok" or "error", summarizeValue(result))
    end
  end

  for _, key in ipairs({ "getMix", "getLogicalSwitch", "getLogicSwitch", "getTrim", "getSpecialFunction", "getInput", "getSwitch", "getChannel" }) do
    if type(safeGet(model, key)) == "function" then
      local ok, result = tryVariants(model, key, { { 0 }, { 1 } })
      lines[#lines + 1] = string.format("model.%s(0/1): %s %s", key, ok and "ok" or "error", summarizeValue(result))
    end
  end
end

local function appendRequiredSurfaceProbe(lines)
  lines[#lines + 1] = "Required SmartMapper read/enumeration support:"
  for _, surface in ipairs(REQUIRED_SURFACES) do
    local available = {}
    for _, key in ipairs(surface.candidates) do
      local namespace = model
      local namespaceName = "model"
      if key == "getSources" then
        namespace = system
        namespaceName = "system"
      end
      local value = safeGet(namespace, key)
      if value ~= nil then
        available[#available + 1] = namespaceName .. "." .. key .. "=" .. type(value)
      end
    end
    if #available > 0 then
      lines[#lines + 1] = "- " .. surface.label .. ": candidate support: " .. join(available)
    else
      lines[#lines + 1] = "- " .. surface.label .. ": no candidate read/enumeration API found"
    end
  end
end

local function buildReportLines()
  local lines = {}
  lines[#lines + 1] = "SmartMapper Ethos API Probe"
  if type(os) == "table" and type(os.date) == "function" then
    local ok, stamp = pcall(os.date, "%Y-%m-%d %H:%M:%S")
    if ok and stamp then
      lines[#lines + 1] = "Captured: " .. tostring(stamp)
    end
  end
  lines[#lines + 1] = "Purpose: validate APIs needed by sloppy-ethos issues #83 and #45."

  appendModelProbe(lines)
  local systemAvailable = listAvailable("system", system, SYSTEM_API_CANDIDATES)
  if #systemAvailable > 0 then
    lines[#lines + 1] = "known system APIs: " .. join(systemAvailable)
  end
  appendSourceProbe(lines)
  appendRequiredSurfaceProbe(lines)
  return lines
end

local function writeReport(lines)
  local payload = join(lines, "\n") .. "\n"
  for _, path in ipairs(REPORT_PATHS) do
    if type(io) == "table" and type(io.open) == "function" then
      local handle = io.open(path, "w")
      if handle then
        handle:write(payload)
        handle:close()
        return path
      end
    end
  end
  return nil
end

local function runProbe()
  local lines = buildReportLines()
  for _, line in ipairs(lines) do
    safePrint("[SmartMapperProbe] " .. line)
  end
  local reportPath = writeReport(lines)
  if reportPath then
    safePrint("[SmartMapperProbe] wrote " .. reportPath)
  else
    safePrint("[SmartMapperProbe] report file write unavailable")
  end
  return lines, reportPath
end

local function create()
  local lines, reportPath = runProbe()
  return {
    lines = lines,
    reportPath = reportPath,
    scroll = 0,
  }
end

local function getWindowSize()
  if type(lcd) == "table" and type(lcd.getWindowSize) == "function" then
    local ok, w, h = pcall(lcd.getWindowSize)
    if ok and type(w) == "number" and type(h) == "number" then
      return w, h
    end
  end
  return 480, 272
end

local function drawText(x, y, text)
  if type(lcd) == "table" and type(lcd.drawText) == "function" then
    pcall(lcd.drawText, x, y, tostring(text or ""))
  end
end

local function setFont(font)
  if type(lcd) == "table" and type(lcd.font) == "function" and font then
    pcall(lcd.font, font)
  end
end

local function paint(tool)
  if type(tool) ~= "table" then
    return
  end

  local _, height = getWindowSize()
  setFont(FONT_HEADER)
  drawText(4, 4, TOOL_NAME)
  setFont(FONT_BODY)
  local reportText = tool.reportPath and ("Report: " .. tool.reportPath) or "Report: print log only"
  drawText(4, 20, reportText)

  local lines = tool.lines or {}
  local maxLines = math.floor((height - HEADER_HEIGHT) / LINE_HEIGHT)
  if maxLines < 1 then
    maxLines = 1
  end
  local start = (tool.scroll or 0) + 1
  for index = 1, maxLines do
    local line = lines[start + index - 1]
    if line then
      drawText(4, HEADER_HEIGHT + ((index - 1) * LINE_HEIGHT), line)
    end
  end
end

local function rerun(tool)
  local lines, reportPath = runProbe()
  tool.lines = lines
  tool.reportPath = reportPath
  tool.scroll = 0
  if type(lcd) == "table" and type(lcd.invalidate) == "function" then
    pcall(lcd.invalidate)
  end
end

local function event(tool, category, value)
  if type(tool) ~= "table" then
    return false
  end
  local touchCategory = rawget(_G, "EVT_TOUCH")
  if touchCategory == nil or category == touchCategory then
    if value == nil or value == 16641 or value == rawget(_G, "TOUCH_END") or value == rawget(_G, "EVT_TOUCH_BREAK") then
      rerun(tool)
      return true
    end
  end
  return false
end

local function init()
  if type(system) ~= "table" or type(system.registerSystemTool) ~= "function" then
    safePrint("[SmartMapperProbe] system.registerSystemTool unavailable")
    return
  end

  local ok, err = pcall(system.registerSystemTool, {
    name = function()
      return TOOL_NAME
    end,
    icon = icon,
    create = create,
    paint = paint,
    event = event,
  })
  if not ok then
    safePrint("[SmartMapperProbe] registerSystemTool failed: " .. tostring(err))
  end
end

local module = { init = init }
if rawget(_G, "__SMARTMAPPER_TEST__") then
  module._test = {
    buildReportLines = buildReportLines,
    create = create,
    event = event,
    paint = paint,
    runProbe = runProbe,
    writeReport = writeReport,
  }
end

return module
