local WIDGET_NAME = "SmartMapper"
local WIDGET_KEY = "smrtmpr"
local ICON_PATHS = {
  "/scripts/SmartMapper/smartmapper.png",
  "/SCRIPTS/SmartMapper/smartmapper.png",
  "smartmapper.png",
}

local LINE_HEIGHT = 14
local HEADER_HEIGHT = 32
local MAX_SCAN_INDEX = 12
local MAX_EMPTY_GAP = 3
local POLL_INTERVAL = 1.0
local DEEP_SCAN_INTERVAL = 5.0
local TOUCH_START = 16640
local TOUCH_END = 16641
local TOUCH_MOVE = 16642

local FONT_HEADER = rawget(_G, "FONT_STD_BOLD") or rawget(_G, "FONT_STD") or rawget(_G, "FONT_XS")
local FONT_BODY = rawget(_G, "FONT_XS") or rawget(_G, "FONT_STD")
local unpackArgs = table.unpack or unpack

local SOURCE_CATEGORIES = {
  { name = "CATEGORY_ANALOG", label = "Analog", inventory = true },
  { name = "CATEGORY_SWITCH", label = "Switch", inventory = true },
  { name = "CATEGORY_SWITCH_POSITION", label = "Switch position", inventory = true, unused = true },
  { name = "CATEGORY_FUNCTION_SWITCH", label = "Function switch", inventory = true, unused = true },
  { name = "CATEGORY_LOGIC_SWITCH", label = "Logic switch", inventory = true },
  { name = "CATEGORY_TRIM", label = "Trim", inventory = true },
  { name = "CATEGORY_TRIM_POSITION", label = "Trim position", inventory = true },
  { name = "CATEGORY_CHANNEL", label = "Channel", inventory = true },
  { name = "CATEGORY_TIMER", label = "Timer", inventory = true },
  { name = "CATEGORY_TELEMETRY_SENSOR", label = "Telemetry", inventory = true },
}

local MODEL_SURFACES = {
  {
    key = "mixes",
    label = "Mix",
    plural = { "getMixes" },
    count = { "getMixCount", "getMixesCount" },
    getters = { "getMix" },
  },
  {
    key = "logicalSwitches",
    label = "Logic",
    plural = { "getLogicalSwitches", "getLogicSwitches" },
    count = { "getLogicalSwitchCount", "getLogicSwitchCount", "getLogicalSwitchesCount", "getLogicSwitchesCount" },
    getters = { "getLogicalSwitch", "getLogicSwitch" },
  },
  {
    key = "trims",
    label = "Trim",
    plural = { "getTrims" },
    count = { "getTrimCount", "getTrimsCount" },
    getters = { "getTrim" },
  },
  {
    key = "specialFunctions",
    label = "Special",
    plural = { "getSpecialFunctions" },
    count = { "getSpecialFunctionCount", "getSpecialFunctionsCount" },
    getters = { "getSpecialFunction" },
  },
  {
    key = "inputs",
    label = "Input",
    plural = { "getInputs", "getSwitches" },
    count = { "getInputCount", "getInputsCount", "getSwitchCount", "getSwitchesCount" },
    getters = { "getInput", "getSwitch" },
  },
  {
    key = "channels",
    label = "Channel",
    plural = { "getChannels", "getOutputs" },
    count = { "getChannelCount", "getChannelsCount", "getOutputCount", "getOutputsCount" },
    getters = { "getChannel", "getOutput" },
  },
}

local CONTROL_FIELDS = {
  "input",
  "source",
  "src",
  "switch",
  "sw",
  "activeCondition",
  "condition",
  "control",
  "trim",
}

local NAME_FIELDS = {
  "name",
  "label",
  "displayName",
  "text",
  "id",
}

local SOURCE_LABEL_FIELDS = {
  "name",
  "label",
  "displayName",
  "id",
}

local AUDIO_FIELDS = {
  "file",
  "filename",
  "path",
  "name",
}

local TEXT_FIELDS = {
  "text",
  "message",
  "value",
}

local function safePrint(line)
  if type(print) == "function" then
    pcall(print, line)
  end
end

local function now()
  if type(os) == "table" and type(os.clock) == "function" then
    local ok, value = pcall(os.clock)
    if ok and type(value) == "number" then
      return value
    end
  end
  return 0
end

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
    return false, nil
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

  return false, nil
end

local function trimText(text)
  text = tostring(text or "")
  return (text:gsub("^%s+", ""):gsub("%s+$", ""))
end

local function compactText(text)
  text = trimText(text)
  text = text:gsub("[\r\n\t]+", " ")
  text = text:gsub("  +", " ")
  return text
end

local function normalizeKey(value)
  return compactText(value):lower()
end

local function truncate(text, maxChars)
  text = tostring(text or "")
  if maxChars == nil or maxChars <= 0 or #text <= maxChars then
    return text
  end
  if maxChars <= 3 then
    return text:sub(1, maxChars)
  end
  return text:sub(1, maxChars - 3) .. "..."
end

local function isRecordList(value)
  return type(value) == "table" and #value > 0
end

local function valueToText(value)
  local valueType = type(value)
  if value == nil then
    return nil
  end
  if valueType == "string" or valueType == "number" or valueType == "boolean" then
    return compactText(value)
  end
  if valueType == "table" or valueType == "userdata" then
    for _, field in ipairs(NAME_FIELDS) do
      local raw = safeGet(value, field)
      if type(raw) == "function" then
        local ok, result = invoke(value, field, {})
        if ok and result ~= nil then
          local label = compactText(result)
          if label ~= "" then
            return label
          end
        end
      elseif raw ~= nil and type(raw) ~= "table" and type(raw) ~= "userdata" then
        local label = compactText(raw)
        if label ~= "" then
          return label
        end
      end
    end
    if type(safeGet(value, "stringValue")) == "function" then
      local ok, result = invoke(value, "stringValue", {})
      if ok and result ~= nil then
        local label = compactText(result)
        if label ~= "" then
          return label
        end
      end
    end
    return compactText(value)
  end
  return compactText(value)
end

local function sourceValueToText(value, depth)
  depth = depth or 0
  if value == nil then
    return nil
  end
  if type(value) == "string" or type(value) == "number" or type(value) == "boolean" then
    return compactText(value)
  end
  if type(value) == "table" or type(value) == "userdata" then
    local category = safeGet(value, "category")
    if type(category) == "function" then
      local ok, result = invoke(value, "category", {})
      category = ok and result or nil
    end
    local member = safeGet(value, "member")
    if type(member) == "function" then
      local ok, result = invoke(value, "member", {})
      member = ok and result or nil
    end
    if depth < 2 and type(category) == "number" and type(member) == "number" and type(system) == "table" then
      local ok, resolved = invoke(system, "getSource", { { category = category, member = member } })
      if ok and resolved ~= nil and resolved ~= value then
        local label = sourceValueToText(resolved, depth + 1)
        if label and label ~= "" then
          return label
        end
      end
    end

    for _, field in ipairs(SOURCE_LABEL_FIELDS) do
      local raw = safeGet(value, field)
      if type(raw) == "function" then
        local ok, result = invoke(value, field, {})
        if ok and result ~= nil then
          local label = compactText(result)
          if label ~= "" then
            return label
          end
        end
      elseif raw ~= nil and type(raw) ~= "table" and type(raw) ~= "userdata" then
        local label = compactText(raw)
        if label ~= "" then
          return label
        end
      end
    end
    if type(safeGet(value, "stringValue")) == "function" then
      local ok, result = invoke(value, "stringValue", {})
      if ok and result ~= nil then
        local label = compactText(result)
        if label ~= "" then
          return label
        end
      end
    end
  end
  return nil
end

local function firstPresent(container, fields)
  if container == nil then
    return nil
  end
  for _, field in ipairs(fields) do
    local value = safeGet(container, field)
    if type(value) == "function" then
      local ok, result = invoke(container, field, {})
      if ok and result ~= nil then
        return result
      end
    elseif value ~= nil then
      return value
    end
  end
  return nil
end

local function firstText(container, fields)
  local text = valueToText(firstPresent(container, fields))
  if text and text ~= "" then
    return text
  end
  return nil
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

local function sourceCategories()
  local categories = {}
  for _, category in ipairs(SOURCE_CATEGORIES) do
    categories[#categories + 1] = {
      name = category.name,
      label = category.label,
      inventory = category.inventory,
      unused = category.unused,
      value = rawget(_G, category.name),
    }
  end
  return categories
end

local function newMapping()
  return {
    modelName = "Unknown model",
    sources = {},
    sourceByKey = {},
    assignments = {},
    statuses = {},
  }
end

local function addStatus(mapping, subsystem, message)
  mapping.statuses[#mapping.statuses + 1] = {
    subsystem = subsystem,
    message = message,
  }
end

local function addSource(mapping, category, value, index)
  local label = valueToText(value)
  if label == nil or label == "" then
    label = string.format("%s %d", category.label, index)
  end

  local key = normalizeKey(label)
  if mapping.sourceByKey[key] then
    return mapping.sourceByKey[key]
  end

  local source = {
    key = key,
    label = label,
    controlType = category.label,
    category = category.name,
    categoryValue = category.value,
    raw = value,
    used = false,
    unusedCandidate = category.unused,
  }
  mapping.sources[#mapping.sources + 1] = source
  mapping.sourceByKey[key] = source
  return source
end

local function addAssignment(mapping, assignment)
  assignment.controlLabel = compactText(assignment.controlLabel)
  if assignment.controlLabel == "" then
    return
  end

  assignment.target = compactText(assignment.target or assignment.subsystem or "Mapped")
  assignment.detail = compactText(assignment.detail or "")
  assignment.controlType = assignment.controlType or "Mapped"
  mapping.assignments[#mapping.assignments + 1] = assignment

  local source = mapping.sourceByKey[normalizeKey(assignment.controlLabel)]
  if source then
    source.used = true
    assignment.controlType = source.controlType or assignment.controlType
  end
end

local function collectSources(mapping)
  if type(system) ~= "table" then
    addStatus(mapping, "Source inventory", "system namespace unavailable")
    return
  end
  if type(safeGet(system, "getSources")) ~= "function" then
    addStatus(mapping, "Source inventory", "system.getSources unavailable")
    return
  end

  for _, category in ipairs(sourceCategories()) do
    if category.inventory then
      if type(category.value) ~= "number" then
        addStatus(mapping, category.label, category.name .. " unavailable")
      else
        local ok, result = invoke(system, "getSources", { category.value })
        if ok and isRecordList(result) then
          for index, source in ipairs(result) do
            addSource(mapping, category, source, index)
          end
        elseif ok then
          addStatus(mapping, category.label, "no sources returned")
        else
          addStatus(mapping, category.label, "getSources failed")
        end
      end
    end
  end
end

local function tableRecords(value)
  local records = {}
  if type(value) ~= "table" then
    return records
  end
  if #value > 0 then
    for _, item in ipairs(value) do
      records[#records + 1] = item
    end
    return records
  end

  local keys = {}
  for key, item in pairs(value) do
    if type(item) == "table" or type(item) == "userdata" then
      keys[#keys + 1] = key
    end
  end
  table.sort(keys, function(a, b)
    return tostring(a) < tostring(b)
  end)
  for _, key in ipairs(keys) do
    records[#records + 1] = value[key]
  end
  return records
end

local function enumerateFromPlural(apiName)
  local ok, result = invoke(model, apiName, {})
  if ok and type(result) == "table" then
    return tableRecords(result), nil
  end
  if ok and result ~= nil then
    return { result }, nil
  end
  return nil, "call failed"
end

local function enumerateWithCount(countName, getterName)
  local ok, count = invoke(model, countName, {})
  if not ok or type(count) ~= "number" then
    return nil, "count unavailable"
  end

  local records = {}
  for index = 0, math.max(0, count - 1) do
    local readOk, value = invoke(model, getterName, { index })
    if readOk and value ~= nil then
      records[#records + 1] = value
    end
  end
  return records, nil
end

local function enumerateBounded(getterName)
  local records = {}
  local emptyGap = 0
  for index = 0, MAX_SCAN_INDEX do
    local ok, value = invoke(model, getterName, { index })
    if ok and value ~= nil then
      records[#records + 1] = value
      emptyGap = 0
    else
      emptyGap = emptyGap + 1
      if emptyGap >= MAX_EMPTY_GAP then
        break
      end
    end
  end
  return records
end

local function enumerateSurface(surface)
  if type(model) ~= "table" and type(model) ~= "userdata" then
    return {}, "model namespace unavailable"
  end

  for _, apiName in ipairs(surface.plural) do
    if type(safeGet(model, apiName)) == "function" then
      local records, err = enumerateFromPlural(apiName)
      if records then
        return records, nil, apiName
      end
      return {}, err or "call failed"
    end
  end

  for _, countName in ipairs(surface.count) do
    if type(safeGet(model, countName)) == "function" then
      for _, getterName in ipairs(surface.getters) do
        if type(safeGet(model, getterName)) == "function" then
          local records, err = enumerateWithCount(countName, getterName)
          if records then
            return records, nil, countName .. "/" .. getterName
          end
          return {}, err or "count read failed"
        end
      end
    end
  end

  for _, getterName in ipairs(surface.getters) do
    if type(safeGet(model, getterName)) == "function" then
      return enumerateBounded(getterName), nil, getterName .. " bounded"
    end
  end

  return {}, "no readable API found"
end

local function addUniqueControl(controls, seen, value)
  local label = sourceValueToText(value) or valueToText(value)
  if label and label ~= "" then
    local key = normalizeKey(label)
    if not seen[key] then
      controls[#controls + 1] = label
      seen[key] = true
    end
  end
end

local function collectNestedSources(controls, seen, value, depth)
  if value == nil or depth > 2 then
    return
  end
  if type(value) ~= "table" and type(value) ~= "userdata" then
    addUniqueControl(controls, seen, value)
    return
  end

  addUniqueControl(controls, seen, value)
  if type(value) ~= "table" then
    return
  end

  if #value > 0 then
    for _, item in ipairs(value) do
      collectNestedSources(controls, seen, item, depth + 1)
    end
  end
end

local function explicitControls(record, surface)
  local controls = {}
  local seen = {}
  for _, field in ipairs(CONTROL_FIELDS) do
    local value = safeGet(record, field)
    if type(value) == "function" then
      local ok, result = invoke(record, field, {})
      if ok then
        collectNestedSources(controls, seen, result, 0)
      end
    elseif value ~= nil then
      collectNestedSources(controls, seen, value, 0)
    end
  end

  if surface.key == "logicalSwitches" and type(safeGet(record, "values")) == "function" then
    local ok, values = invoke(record, "values", {})
    if ok then
      collectNestedSources(controls, seen, values, 0)
    end
  end

  return controls
end

local function findActionValue(value, fields, typeHints, depth)
  if value == nil or depth > 3 then
    return nil
  end
  local valueType = type(value)
  if valueType == "string" then
    if #typeHints == 0 then
      return compactText(value)
    end
    return nil
  end
  if valueType ~= "table" and valueType ~= "userdata" then
    return nil
  end

  local actionType = firstText(value, { "type", "action", "name" }) or ""
  local lowerAction = actionType:lower()
  local typeMatches = #typeHints == 0
  for _, hint in ipairs(typeHints) do
    if lowerAction:find(hint, 1, true) then
      typeMatches = true
      break
    end
  end
  if typeMatches then
    local text = firstText(value, fields)
    if text then
      return text
    end
  end

  if valueType == "table" then
    if #value > 0 then
      for _, item in ipairs(value) do
        local found = findActionValue(item, fields, typeHints, depth + 1)
        if found then
          return found
        end
      end
    else
      for _, item in pairs(value) do
        if type(item) ~= "function" then
          local found = findActionValue(item, fields, typeHints, depth + 1)
          if found then
            return found
          end
        end
      end
    end
  end
  return nil
end

local function assignmentTarget(record, surface)
  if surface.key == "specialFunctions" then
    local audio = findActionValue(record, AUDIO_FIELDS, { "play", "audio", "file", "sound" }, 0)
    if audio then
      return audio
    end
    local text = findActionValue(record, TEXT_FIELDS, { "play", "text", "say" }, 0)
    if text then
      return text
    end
  end

  local name = firstText(record, NAME_FIELDS)
  if name then
    return name
  end
  return surface.label
end

local function recordDetail(record, apiName)
  local detailParts = {}
  if apiName and apiName ~= "" then
    detailParts[#detailParts + 1] = apiName
  end

  local op = firstText(record, { "operation", "func", "mode" })
  if op then
    detailParts[#detailParts + 1] = op
  end

  local out = ""
  for index, value in ipairs(detailParts) do
    if index > 1 then
      out = out .. " / "
    end
    out = out .. value
  end
  return out
end

local function collectAssignments(mapping)
  for _, surface in ipairs(MODEL_SURFACES) do
    local records, err, apiName = enumerateSurface(surface)
    if err then
      addStatus(mapping, surface.label, err)
    elseif #records == 0 then
      addStatus(mapping, surface.label, "no records returned")
    end

    for _, record in ipairs(records) do
      local controls = explicitControls(record, surface)
      local target = assignmentTarget(record, surface)
      for _, control in ipairs(controls) do
        addAssignment(mapping, {
          subsystem = surface.label,
          controlLabel = control,
          controlType = surface.label,
          target = target,
          detail = recordDetail(record, apiName),
        })
      end
    end
  end
end

local function modelName()
  if type(model) ~= "table" and type(model) ~= "userdata" then
    return "Unknown model"
  end
  if type(safeGet(model, "name")) == "function" then
    local ok, result = invoke(model, "name", {})
    if ok and result ~= nil then
      local text = valueToText(result)
      if text and text ~= "" then
        return text
      end
    end
  end
  return "Unknown model"
end

local function buildMapping()
  local mapping = newMapping()
  mapping.modelName = modelName()
  collectSources(mapping)
  collectAssignments(mapping)
  return mapping
end

local function assignmentSort(a, b)
  local left = (a.controlType or ""):lower() .. "\0" .. (a.controlLabel or ""):lower() .. "\0" .. (a.target or ""):lower()
  local right = (b.controlType or ""):lower() .. "\0" .. (b.controlLabel or ""):lower() .. "\0" .. (b.target or ""):lower()
  return left < right
end

local function sourceSort(a, b)
  local left = (a.controlType or ""):lower() .. "\0" .. (a.label or ""):lower()
  local right = (b.controlType or ""):lower() .. "\0" .. (b.label or ""):lower()
  return left < right
end

local function addSection(rows, title)
  rows[#rows + 1] = { kind = "section", title = title }
end

local function buildRows(mapping)
  local rows = {}
  local assignments = {}
  for _, assignment in ipairs(mapping.assignments or {}) do
    assignments[#assignments + 1] = assignment
  end
  table.sort(assignments, assignmentSort)

  if #assignments > 0 then
    addSection(rows, "Assigned controls")
    for _, assignment in ipairs(assignments) do
      rows[#rows + 1] = {
        kind = "assignment",
        type = assignment.controlType,
        control = assignment.controlLabel,
        target = assignment.target,
        detail = assignment.detail,
      }
    end
  end

  local unused = {}
  for _, source in ipairs(mapping.sources or {}) do
    if source.unusedCandidate and not source.used then
      unused[#unused + 1] = source
    end
  end
  table.sort(unused, sourceSort)
  if #unused > 0 then
    addSection(rows, "Unused switches")
    for _, source in ipairs(unused) do
      rows[#rows + 1] = {
        kind = "unused",
        type = source.controlType,
        control = source.label,
        target = "unused",
        detail = source.category,
      }
    end
  end

  if #rows == 0 then
    rows[#rows + 1] = {
      kind = "empty",
      message = "No accessible model mappings found.",
    }
  end

  if #(mapping.statuses or {}) > 0 then
    addSection(rows, "Status")
    for _, status in ipairs(mapping.statuses) do
      rows[#rows + 1] = {
        kind = "status",
        type = status.subsystem,
        control = "",
        target = status.message,
        detail = "",
      }
    end
  end

  return rows
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

local function setFont(font)
  if type(lcd) == "table" and type(lcd.font) == "function" and font then
    pcall(lcd.font, font)
  end
end

local function drawText(x, y, text)
  if type(lcd) == "table" and type(lcd.drawText) == "function" then
    pcall(lcd.drawText, x, y, tostring(text or ""))
  end
end

local function invalidate()
  if type(lcd) == "table" and type(lcd.invalidate) == "function" then
    pcall(lcd.invalidate)
  end
end

local function maxScroll(widget)
  local _, height = getWindowSize()
  local visible = math.max(1, math.floor((height - HEADER_HEIGHT) / LINE_HEIGHT))
  local count = #(widget.rows or {})
  return math.max(0, count - visible)
end

local function clampScroll(widget)
  local limit = maxScroll(widget)
  if (widget.scroll or 0) < 0 then
    widget.scroll = 0
  elseif widget.scroll > limit then
    widget.scroll = limit
  end
end

local function refresh(widget, reason)
  widget.mapping = buildMapping()
  widget.rows = buildRows(widget.mapping)
  widget.modelName = widget.mapping.modelName
  widget.lastDeepScan = now()
  widget.lastPoll = widget.lastDeepScan
  widget.refreshReason = reason or "scan"
  clampScroll(widget)
  invalidate()
end

local function create()
  local widget = {
    scroll = 0,
    rows = {},
    mapping = nil,
    modelName = "Unknown model",
    lastDeepScan = 0,
    lastPoll = 0,
    touchY = nil,
  }
  refresh(widget, "create")
  return widget
end

local function visibleRows(height)
  return math.max(1, math.floor((height - HEADER_HEIGHT) / LINE_HEIGHT))
end

local function paintRow(row, y, width)
  if row.kind == "section" then
    if width < 360 then
      setFont(FONT_BODY)
    else
      setFont(FONT_HEADER)
    end
    drawText(4, y, truncate(row.title, math.floor(width / 8)))
    setFont(FONT_BODY)
    return
  end

  if row.kind == "empty" then
    drawText(4, y, row.message)
    return
  end

  local typeWidth = 12
  local controlWidth = 17
  local targetWidth = 18
  if width >= 700 then
    typeWidth = 16
    controlWidth = 24
    targetWidth = 28
  elseif width < 400 then
    typeWidth = 9
    controlWidth = 14
    targetWidth = 14
  end

  drawText(4, y, truncate(row.type or "", typeWidth))
  drawText(82, y, truncate(row.control or "", controlWidth))
  drawText(208, y, truncate(row.target or "", targetWidth))
  if width >= 430 then
    drawText(340, y, truncate(row.detail or "", math.max(8, math.floor((width - 340) / 8))))
  end
end

local function paint(widget)
  if type(widget) ~= "table" then
    return
  end

  local width, height = getWindowSize()
  setFont(FONT_HEADER)
  drawText(4, 4, WIDGET_NAME)
  setFont(FONT_BODY)
  drawText(4, 18, truncate(widget.modelName or "Unknown model", math.max(10, math.floor((width - 8) / 8))))

  local rows = widget.rows or {}
  local start = (widget.scroll or 0) + 1
  local maxRows = visibleRows(height)
  for index = 1, maxRows do
    local row = rows[start + index - 1]
    if row then
      paintRow(row, HEADER_HEIGHT + ((index - 1) * LINE_HEIGHT), width)
    end
  end
end

local function scrollBy(widget, delta)
  widget.scroll = (widget.scroll or 0) + delta
  local before = widget.scroll
  clampScroll(widget)
  if widget.scroll ~= before or delta ~= 0 then
    invalidate()
  end
end

local function isTouch(category)
  local touchCategory = rawget(_G, "EVT_TOUCH")
  return touchCategory == nil or category == touchCategory
end

local function eventValue(category, value)
  return value ~= nil and value or category
end

local function event(widget, category, value, x, y)
  if type(widget) ~= "table" then
    return false
  end

  local key = eventValue(category, value)
  if key == rawget(_G, "EVT_VIRTUAL_NEXT") or key == rawget(_G, "EVT_ROT_RIGHT") or key == rawget(_G, "EVT_PLUS_BREAK") then
    scrollBy(widget, 1)
    return true
  end
  if key == rawget(_G, "EVT_VIRTUAL_PREV") or key == rawget(_G, "EVT_ROT_LEFT") or key == rawget(_G, "EVT_MINUS_BREAK") then
    scrollBy(widget, -1)
    return true
  end
  if key == rawget(_G, "EVT_ENTER_BREAK") then
    refresh(widget, "manual")
    return true
  end

  if isTouch(category) then
    if value == TOUCH_START or value == rawget(_G, "TOUCH_START") then
      widget.touchY = y
      return true
    end
    if value == TOUCH_MOVE or value == rawget(_G, "TOUCH_MOVE") then
      if type(widget.touchY) == "number" and type(y) == "number" then
        local delta = y - widget.touchY
        if math.abs(delta) >= LINE_HEIGHT then
          scrollBy(widget, delta < 0 and 1 or -1)
          widget.touchY = y
        end
      end
      return true
    end
    if value == TOUCH_END or value == rawget(_G, "TOUCH_END") or value == rawget(_G, "EVT_TOUCH_BREAK") then
      widget.touchY = nil
      return true
    end
  end

  return false
end

local function wakeup(widget)
  if type(widget) ~= "table" then
    return
  end

  local current = now()
  if current - (widget.lastPoll or 0) < POLL_INTERVAL then
    return
  end
  widget.lastPoll = current

  local currentModelName = modelName()
  if currentModelName ~= widget.modelName or current - (widget.lastDeepScan or 0) >= DEEP_SCAN_INTERVAL then
    refresh(widget, "poll")
  end
end

local function init()
  if type(system) ~= "table" or type(system.registerWidget) ~= "function" then
    safePrint("[SmartMapper] system.registerWidget unavailable")
    return
  end

  local ok, err = pcall(system.registerWidget, {
    key = WIDGET_KEY,
    name = function()
      return WIDGET_NAME
    end,
    icon = icon,
    create = create,
    paint = paint,
    wakeup = wakeup,
    event = event,
  })
  if not ok then
    safePrint("[SmartMapper] registerWidget failed: " .. tostring(err))
  end
end

local module = { init = init }
if rawget(_G, "__SMARTMAPPER_TEST__") then
  module._test = {
    buildMapping = buildMapping,
    buildRows = buildRows,
    create = create,
    event = event,
    paint = paint,
    refresh = refresh,
    wakeup = wakeup,
    valueToText = valueToText,
  }
end

return module
