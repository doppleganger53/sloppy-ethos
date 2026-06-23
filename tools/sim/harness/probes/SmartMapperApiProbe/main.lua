local PROBE_NAME = "SmartMapperApiProbe"

local CATEGORY_NAMES = {
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
  "CATEGORY_TIMER",
  "CATEGORY_TELEMETRY_SENSOR",
  "CATEGORY_FLIGHT",
  "CATEGORY_FLIGHT_VALUE",
  "CATEGORY_SYSTEM",
  "CATEGORY_SYSTEM_EVENT",
  "CATEGORY_SPECIAL",
}

local MODEL_API_NAMES = {
  "name",
  "createMix",
  "getMix",
  "getMixes",
  "getMixCount",
  "getLogicSwitch",
  "getLogicSwitches",
  "getLogicalSwitch",
  "getLogicalSwitches",
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

local unpackArgs = table.unpack or unpack

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

local function invoke(container, key, args)
  local fn = safeGet(container, key)
  if type(fn) ~= "function" then
    return false, "not callable"
  end
  args = args or {}
  local ok, result = pcall(fn, unpackArgs(args))
  if ok then
    return true, result
  end
  local selfArgs = { container }
  for _, arg in ipairs(args) do
    selfArgs[#selfArgs + 1] = arg
  end
  ok, result = pcall(fn, unpackArgs(selfArgs))
  if ok then
    return true, result
  end
  return false, tostring(result)
end

local function jsonString(value)
  value = tostring(value or "")
  value = value:gsub("\\", "\\\\")
  value = value:gsub("\"", "\\\"")
  value = value:gsub("\n", "\\n")
  value = value:gsub("\r", "\\r")
  value = value:gsub("\t", "\\t")
  return "\"" .. value .. "\""
end

local function isArray(value)
  if type(value) ~= "table" then
    return false
  end
  local maxIndex = 0
  local count = 0
  for key, _ in pairs(value) do
    if type(key) ~= "number" or key < 1 or key % 1 ~= 0 then
      return false
    end
    if key > maxIndex then
      maxIndex = key
    end
    count = count + 1
  end
  return maxIndex == count
end

local function encodeJson(value)
  local valueType = type(value)
  if value == nil then
    return "null"
  end
  if valueType == "string" then
    return jsonString(value)
  end
  if valueType == "number" or valueType == "boolean" then
    return tostring(value)
  end
  if valueType ~= "table" then
    return jsonString(valueType)
  end

  local parts = {}
  if isArray(value) then
    for index, item in ipairs(value) do
      parts[index] = encodeJson(item)
    end
    return "[" .. table.concat(parts, ",") .. "]"
  end

  local keys = {}
  for key, _ in pairs(value) do
    keys[#keys + 1] = tostring(key)
  end
  table.sort(keys)
  for _, key in ipairs(keys) do
    parts[#parts + 1] = jsonString(key) .. ":" .. encodeJson(value[key])
  end
  return "{" .. table.concat(parts, ",") .. "}"
end

local function countTable(value)
  if type(value) ~= "table" then
    return nil
  end
  local count = 0
  for _, _ in pairs(value) do
    count = count + 1
  end
  return count
end

local function sampleValue(value)
  if value == nil then
    return "nil"
  end
  if type(value) == "string" or type(value) == "number" or type(value) == "boolean" then
    return tostring(value)
  end
  return type(value)
end

local function sourceReports()
  local reports = {}
  if type(system) ~= "table" or type(safeGet(system, "getSources")) ~= "function" then
    return reports
  end
  for _, name in ipairs(CATEGORY_NAMES) do
    local category = rawget(_G, name)
    local report = {
      value = category,
      available = type(category) == "number",
    }
    if type(category) == "number" then
      local ok, result = invoke(system, "getSources", { category })
      report.ok = ok
      report.count = countTable(result) or 0
      if type(result) == "table" and result[1] ~= nil then
        report.first = sampleValue(result[1])
      end
    end
    reports[name] = report
  end
  return reports
end

local function modelReport()
  local report = {
    availableApis = {},
    samples = {},
  }
  if type(model) ~= "table" and type(model) ~= "userdata" then
    report.available = false
    return report
  end
  report.available = true

  if type(safeGet(model, "name")) == "function" then
    local ok, result = invoke(model, "name", {})
    report.name = ok and sampleValue(result) or "error"
  end

  for _, name in ipairs(MODEL_API_NAMES) do
    local value = safeGet(model, name)
    if value ~= nil then
      report.availableApis[#report.availableApis + 1] = name .. "=" .. type(value)
    end
  end

  for _, name in ipairs({ "getMix", "getLogicSwitch", "getLogicalSwitch", "getTrim", "getSpecialFunction", "getChannel" }) do
    if type(safeGet(model, name)) == "function" then
      local ok, result = invoke(model, name, { 0 })
      report.samples[name] = {
        ok = ok,
        type = type(result),
        sample = sampleValue(result),
      }
    end
  end
  return report
end

local function buildReport()
  return {
    probe = PROBE_NAME,
    categories = sourceReports(),
    model = modelReport(),
  }
end

local function init()
  local payload = encodeJson(buildReport())
  print("[SimProbe:" .. PROBE_NAME .. "] " .. payload)
end

return { init = init }
