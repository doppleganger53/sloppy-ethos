'use strict';

// All actions operate on a newly generated model and disposable radio storage.
const openTool = [['tap', 706, 391], ['tap', 520, 444], ['nextPage'], ['tap', 306, 168]];
const addCalculated = [['tap', 706, 391], ['tap', 226, 442], ['nextPage'], ['tap', 150, 173], ['tap', 588, 266]];
const openCustom = [...addCalculated, ['tap', 752, 154], ['scroll'], ['tap', 381, 359]];

function tool(body) {
  return {
    actions: openTool,
    lua: `local function run()
${body}
  model.dirty()
  print("FIXTURE OK")
  return {}
end
return { init = function()
  system.registerSystemTool({
    name = function() return "Fixture" end,
    icon = lcd.loadMask("/scripts/FxLab/icon.png"),
    create = run, paint = function() end
  })
end }`,
  };
}

function throttle(application) {
  return tool(`model.name("Bench")
for _, entry in ipairs({{0, "Throttle amps"}, {1, "Throttle volts"}, {2, "Throttle temp"}, {4, "Throttle pos"}}) do
  local row = assert(model.createSensor({type=SENSOR_TYPE_SPORT, module=0, band=1,
    subId=entry[1], physId=3, appId=${application}}))
  row:name(entry[2])
  print("FX", row:name(), row:module(), row:band(), row:subId(), row:physId(), row:appId())
end`);
}

const variantKeys = [
  [0xf108, [0, 1, 2]], [0xf011, [0]], [0xf010, [0]], [0xf107, [0]], [0xf101, [0, 1]],
  [0x0e20, [0]], [0x0d90, [1]], [0x0dc0, [0]], [0x0dd0, [1]], [0x0df0, [0]],
  [0x0d80, [0, 1]], [0x0da0, [0, 1]], [0x0de0, [0, 1]], [0x0db0, [0, 1]], [0xf103, [0]],
  [0x6800, [0, 1, 2, 4, 5, 6]], [0x6801, [0, 1, 2, 4, 5, 6]],
  [0x6802, [0, 1, 2, 4, 5, 6]], [0x6803, [0, 1, 2, 4, 5, 6]], [0x0100, [0]],
];
const keysLua = variantKeys.map(([app, slots]) => `{${app},{${slots.join(',')}}}`).join(',');
const sensorVariants = tool(`for _, group in ipairs({${keysLua}}) do
  for _, sub in ipairs(group[2]) do
    local row=assert(model.createSensor({type=SENSOR_TYPE_SPORT, appId=group[1], subId=sub, physId=3, band=1}))
    row:name(string.format("Q%04X%02X",group[1],sub))
    print("FX",row:name(),row:module(),row:band(),row:subId(),row:physId(),row:appId())
  end
end`);

const layout = tool(`model.name("Fixture layout")
local first=assert(model.createSensor({type=SENSOR_TYPE_SPORT, module=1, band=1, subId=2, physId=3, appId=0x6803}))
first:name("Repeated")
local second=assert(model.createSensor({type=SENSOR_TYPE_SPORT, module=0, band=0, subId=1, physId=4, appId=0x6802}))
second:name("Repeated")
for _, row in ipairs({first,second}) do
  print("FX",row:name(),row:module(),row:band(),row:subId(),row:physId(),row:appId())
end
local timer=model.getTimer(0)
timer:name("Ref timer")
timer:startCondition(second)
local source=timer:startCondition()
print("FX",timer:name(),source:name(),source:category(),source:member())`);

const reordered = tool(`model.name("Bench")
for _, entry in ipairs({{1, "Throttle volts"}, {0, "Throttle amps"}, {4, "Throttle pos"}, {5, "Throttle speed"}}) do
  local row=assert(model.createSensor({type=SENSOR_TYPE_SPORT,module=0,band=1,subId=entry[1],physId=3,appId=0x6803}))
  row:name(entry[2])
  print("FX",row:name(),row:module(),row:band(),row:subId(),row:physId(),row:appId())
end`);
const collisions = tool(`for _, app in ipairs({0x6802,0x6803}) do
  local row=assert(model.createSensor({type=SENSOR_TYPE_SPORT,module=0,band=1,subId=1,physId=3,appId=app}))
  row:name("Twin")
  print("FX",row:name(),row:module(),row:band(),row:subId(),row:physId(),row:appId())
end`);

const modelMenu = [['tap', 706, 391], ['tap', 226, 442]];
const modelPage2 = [...modelMenu, ['nextPage']];
const screenMenu = [['tap', 706, 391], ['tap', 375, 444]];
const newScreen = [...screenMenu, ['tap', 191, 90], ['tap', 487, 156]];
const add = ['tap', 400, 270];
const luaMenu = [...modelPage2, ['tap', 650, 340]];
const registeredLua = `return { init = function()
  system.registerTask({key="FxTask", name=function() return "Fixture task" end, wakeup=function() end})
  system.registerSource({key="FxSrc", name="Fixture source", wakeup=function() end})
  system.registerWidget({key="FxView", name=function() return "Fixture view" end,
    create=function() return {} end, paint=function() end})
  print("FIXTURE OK")
end }`;

function menuCase(id, description, actions, readback, options = {}) {
  return { id, description, compare: options.compare || 'default', readback,
    fixture: { actions, lua: options.lua } };
}

const structureCases = [
  menuCase('flight-mode', 'Add a flight mode after the default mode.',
    [...modelMenu, ['tap', 484, 169], ['tap', 571, 90]],
    { 'flightModes.length': 2, 'flightModes.1.activeCondition': null, 'flightModes.1.fadeIn': 0 }),
  menuCase('free-mix', 'Add Free mix with its default weight action.',
    [...modelMenu, ['tap', 652, 169], add, ['tap', 96, 98]],
    { 'mixes.0.key': 'free', 'mixes.0.name': 'Free mix', 'mixes.0.actions.0.weight.defaultValue': 100 }),
  menuCase('logic-switch', 'Create the default A approximately equal to X logic switch.',
    [...modelPage2, ['tap', 486, 173], add],
    { 'logicSwitches.0.key': 'around', 'logicSwitches.0.values.0.category': 'analog', 'logicSwitches.0.values.1': 0 }),
  menuCase('special-function', 'Create the initially disabled Reset special function.',
    [...modelPage2, ['tap', 655, 173], add],
    { 'functions.0.key': 'reset', 'functions.0.enable': false, 'functions.0.activeCondition.category': 'always_on' }),
  menuCase('variable', 'Create one default variable.',
    [...modelPage2, ['tap', 315, 338], add],
    { 'vars.0.min': -1000, 'vars.0.max': 1000, 'vars.0.values.defaultValue': 0 }),
  menuCase('trim', 'Set the rudder trim step to Medium.',
    [...modelMenu, ['tap', 486, 345], ['tap', 760, 212], ['tap', 344, 313]],
    { 'trims.0.step': 3, 'trims.0.range': 25 }),
  menuCase('rf', 'Enable the internal module in the isolated radio.',
    [...modelMenu, ['tap', 652, 342], ['tap', 772, 151], ['tap', 715, 209]],
    { 'modulesType.0': 11, 'rf.modules.0.enable': true, 'rf.modules.0.channelsCount': 16 }),
  menuCase('checklist', 'Change switch SA preflight position from Up to Middle.',
    [...modelPage2, ['tap', 318, 172], ['tap', 772, 208], ['tap', 760, 326], ['tap', 348, 285]],
    { 'preflightChecks.switches': 1431655766 }),
  menuCase('trainer', 'Select trainer Master mode.',
    [...modelPage2, ['tap', 482, 348], ['tap', 766, 92], ['tap', 317, 265]],
    { 'trainerCable.settingsType': 'trainer_in', 'trainerCable.trainer_in.channels.0.destination.category': 'analog' }),
  menuCase('screen', 'Add a second screen with a 2x2 layout and four empty widget slots.',
    newScreen, { 'screens.length': 2, 'screens.1.key': '2x2', 'screens.1.widgets.length': 4,
      'screens.1.widgets.0.key': 'none' }),
  menuCase('lua-references', 'Enable the synthetic FxTask and FxSrc Lua registrations.',
    [...luaMenu, ['tap', 710, 91], ['tap', 710, 151]],
    { 'luaSources.0.key': 427119835206, 'luaTasks.0.key': 118143298336838 }, { lua: registeredLua }),
  menuCase('widget-reference', 'Assign the synthetic FxView widget to screen 2 slot 0.',
    [...newScreen, ['tap', 310, 240], ['scroll'], ['scroll'], ['tap', 359, 361]],
    { 'screens.1.key': '2x2', 'screens.1.widgets.0.key': 'FxView' },
    { compare: 'screen', lua: registeredLua }),
];

const cases = [
  { id: 'default', description: 'Generate an empty model and resave after an ordinary startup and shutdown.' },
  { id: 'name-short', compare: 'default', description: 'Set the model label to A.', fixture: tool('model.name("A")\nprint("FX",model.name())') },
  { id: 'name-long', compare: 'name-short', description: 'Set the 15-byte model label Model Test 1234.', fixture: tool('model.name("Model Test 1234")\nprint("FX",model.name())') },
  { id: 'throttle-before', compare: 'default', description: 'Create four named S.Port throttle slots at 03/6803.', fixture: throttle(0x6803) },
  { id: 'throttle-after', compare: 'throttle-before', description: 'Independently create the same slots at 03/6802.', fixture: throttle(0x6802) },
  { id: 'throttle-reordered', compare: 'throttle-before', description: 'Swap the first two slots, remove temperature and add speed.', fixture: reordered },
  { id: 'id-conflict', compare: 'default', description: 'Create duplicate labels on the same route, sub-ID and Physical ID with distinct Application IDs.', fixture: collisions },
  { id: 'layout-references', compare: 'default', description: 'Create two duplicate labels on different modules/bands and point timer 0 start at telemetry slot 1.', fixture: layout },
  { id: 'sensor-variants', compare: 'default', description: 'Create S.Port keys with varying field widths and identify the firmware-accepted keys.', fixture: sensorVariants },
  { id: 'timer', compare: 'default', description: 'Name timer 1, set down direction, start 173 and alarm 37 seconds.',
    fixture: tool(`local t=model.getTimer(1)
t:name("Flight timer")
t:direction(-1)
t:start(173)
t:alarm(37)
print("FX",t:name(),t:direction(),t:start(),t:alarm())`) },
  { id: 'channel', compare: 'default', description: 'Set output 1 to Flap left, center 3, limits -72 and +83, and reverse its direction.',
    fixture: tool(`local ch=model.getChannel(1)
ch:name("Flap left")
ch:min(-72)
ch:max(83)
ch:center(3)
ch:direction(-1)
print("FX",ch:name(),ch:center(),ch:min(),ch:max(),ch:direction())`) },
  { id: 'curve', compare: 'default', description: 'Insert a curve and name it Bench rate.',
    fixture: tool('local cv=model.createCurve()\ncv:name("Bench rate")\nprint("FX",cv:name(),cv:type())') },
  { id: 'diy', compare: 'default', description: 'Create a DIY sensor with physical 03, application 0100.',
    fixture: tool(`local row=model.createSensor({type=SENSOR_TYPE_DIY,appId=0x0100,physId=3})
row:name("Bench DIY")
print("FX",row:name(),row:physId(),row:appId(),row:protocolUnit(),row:protocolDecimals())`) },
  { id: 'calculated-consumption', compare: 'default', description: 'Use Model > Telemetry > Create calculated sensor, default Consumption.', fixture: { actions: addCalculated } },
  { id: 'calculated-custom', compare: 'calculated-consumption', description: 'Switch the calculated sensor formula from Consumption to Custom.', fixture: { actions: openCustom } },
  { id: 'calculated-operation', compare: 'calculated-custom', description: 'Add one default Add operation to a Custom sensor.', fixture: { actions: [...openCustom, ['scroll'], ['tap', 567, 451]] } },
];

function timerAudio(rows) {
  return tool(`local t=model.getTimer(0)
t:audioActions({${rows.join(',')}})
for _,r in ipairs(t:audioActions()) do
  print("FX",r.type,r.start,r.step,r.filename or "",tostring(r.haptic))
end`);
}

const firstAudio = '{type=COUNTDOWN_VALUE,start=10,step=30,haptic=1}';
const widgetMenu = [...newScreen, ['tap', 310, 240]];
const extraCases = [
  { id: 'timer-limits', compare: 'default', description: 'Set timer 1 start and alarm to the two signed 32-bit extrema.',
    fixture: tool(`local t=model.getTimer(1)
t:start(-2147483648)
t:alarm(2147483647)
print("FX",t:start(),t:alarm())`),
    readback: { 'timers.1.startValue': -2147483648, 'timers.1.alarmValue': 2147483647 } },
  { id: 'timer-audio-one', compare: 'default', description: 'Add one haptic countdown value at 10 seconds, interval 30.',
    fixture: timerAudio([firstAudio]), readback: { 'timers.0.audioActions.length': 1, 'timers.0.audioActions.0.haptic': true } },
  { id: 'timer-audio-three', compare: 'timer-audio-one', description: 'Append a countdown beep and a haptic sound-file action.',
    fixture: timerAudio([firstAudio, '{type=COUNTDOWN_BEEP,start=11,step=31,haptic=0}',
      '{type=PLAY_FILE,start=12,step=32,filename="bench",haptic=1}']),
    readback: { 'timers.0.audioActions.length': 3, 'timers.0.audioActions.1.type': 'countdown_beep',
      'timers.0.audioActions.2.filename': 'bench', 'timers.0.audioActions.2.step': 32 } },
  menuCase('widget-title', 'Turn off the title of the first Timer Value widget.',
    [...screenMenu, ['tap', 653, 181], ['tap', 713, 212]], { 'screens.0.widgets.1.title': false }),
  menuCase('widget-timer-source', 'Change the first Value widget from Timer1 to Timer2.',
    [...screenMenu, ['tap', 653, 181], ['tap', 758, 151], ['tap', 500, 302], ['tap', 500, 263]],
    { 'screens.0.widgets.1.source.category': 'timer', 'screens.0.widgets.1.source.member': 1 }),
  menuCase('gps-widget', 'Place a GPS Map widget on the second screen.',
    [...widgetMenu, ['tap', 348, 361]], { 'screens.1.widgets.0.key': 'gps_map',
      'screens.1.widgets.0.maxLongitude': 1800000000 }, { compare: 'screen' }),
  menuCase('lipo-widget', 'Place a LiPo widget on the second screen.',
    [...widgetMenu, ['scroll'], ['tap', 348, 212]], { 'screens.1.widgets.0.key': 'lipo',
      'screens.1.widgets.0.voltageWarning': 300 }, { compare: 'screen' }),
  menuCase('channels-widget', 'Place a Channels widget on the second screen.',
    [...widgetMenu, ['scroll'], ['tap', 348, 262]], { 'screens.1.widgets.0.key': 'channels',
      'screens.1.widgets.0.channels': [1, 2, 3, 4] }, { compare: 'screen' }),
  menuCase('text-widget', 'Place a Text widget on the second screen.',
    [...widgetMenu, ['scroll'], ['tap', 348, 363]], { 'screens.1.widgets.0.key': 'text',
      'screens.1.widgets.0.filePath': '' }, { compare: 'screen' }),
  menuCase('timer-widget', 'Place a Timer logs widget on the second screen.',
    [...newScreen, ['tap', 310, 240], ['tap', 367, 311]], { 'screens.1.widgets.0.key': 'timer_logs',
      'screens.1.widgets.0.reversedOrder': false }, { compare: 'screen' }),
  menuCase('chart-widget', 'Place a Line chart widget on the second screen.',
    [...newScreen, ['tap', 310, 240], ['scroll'], ['tap', 355, 314]], { 'screens.1.widgets.0.key': 'line_chart',
      'screens.1.widgets.0.interval': 10, 'screens.1.widgets.0.minValue': -1024 }, { compare: 'screen' }),
  menuCase('telemetry-competition', 'Enable Competition on an empty synthetic model.',
    [...modelPage2, ['tap', 150, 173], ['tap', 707, 153], ['tap', 480, 300]], { 'telemetry.competition': true }),
  menuCase('telemetry-bluetooth', 'Enable the Bluetooth telemetry mode on an empty synthetic model.',
    [...modelPage2, ['tap', 150, 173], ['tap', 707, 212]], { 'bluetooth.mode': 'telemetry' }),
  menuCase('checklist-sa-down', 'Set the simulated SA checklist position to Down.',
    [...modelPage2, ['tap', 318, 172], ['tap', 772, 208], ['tap', 760, 326], ['tap', 348, 338]],
    { 'preflightChecks.switches': 1431655767 }),
  menuCase('checklist-sa-none', 'Set the simulated SA checklist position to No check.',
    [...modelPage2, ['tap', 318, 172], ['tap', 772, 208], ['tap', 760, 326], ['tap', 348, 190]],
    { 'preflightChecks.switches': 1431655764 }),
  menuCase('checklist-failsafe', 'Capture the second state of the failsafe reminder on the empty synthetic model.',
    [...modelPage2, ['tap', 318, 172], ['tap', 711, 151]], { 'preflightChecks.failsafeCheck': 0 }),
];

const extraServo = {
  id: 'servo-addresses', compare: 'default',
  description: 'Create 19 additional servo address/sub-ID combinations in the native firmware.',
  fixture: tool(`for _,group in ipairs({{0x6804,{0,1,2,4}}, {0x6805,{0,1,2,4}}, {0x6807,{0,1,2}},
    {0x6808,{0,1,2,4}}, {0x6809,{0,1,2,4}}}) do
  for _,sub in ipairs(group[2]) do
    local row=assert(model.createSensor({type=SENSOR_TYPE_SPORT,appId=group[1],subId=sub,band=1,physId=3}))
    row:name(string.format("S%04X%02X",group[1],sub))
    print("FX",row:name(),row:module(),row:band(),row:subId(),row:physId(),row:appId())
  end
end`),
};

const outputLimits = {
  id: 'output-limits', compare: 'default',
  description: 'Set output 1 to the full signed limits and a negative fractional center.',
  fixture: tool(`local ch=model.getChannel(1)
ch:min(-150)
ch:max(150)
ch:center(-7.5)
ch:direction(-1)
print("FX",ch:center(),ch:min(),ch:max(),ch:direction())`),
  readback: { 'channels.1.center': -7.5, 'channels.1.limitMin': -150,
    'channels.1.limitMax': 150, 'channels.1.invert': true },
};

module.exports = { cases: [...cases, ...structureCases, ...extraCases, extraServo, outputLimits], variantKeys };
