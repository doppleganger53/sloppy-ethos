'use strict';

// All module operations below are menu inputs in an isolated X20 Pro AW FCC 1.6.8 simulator.
const tap = (x, y) => ['tap', x, y];
const menu = [tap(706, 391), tap(226, 442), tap(652, 342)];
const cases = [];
function example(id, seed, description, actions, slot, type, protocol, count, extra = {}) {
  const module = 'rf.modules.' + slot + '.';
  cases.push({
    id, seed, compare: seed, description, fixture: { actions: [...menu, ...actions] },
    readback: {
      ['modulesType.' + slot]: type, [module + 'enable']: true,
      [module + 'type']: type, [module + 'protocol']: protocol,
      [module + 'channelsCount']: count, 'telemetry.sensors.length': 4,
      'telemetry.sensors.0.key.appId': 0x6803, ...extra,
    },
  });
}

example('rf-internal-access', 'throttle-before', 'Enable the internal module in ACCESS/ACCST D16 mode.',
  [tap(772, 151), tap(715, 209)], 0, 11, 0, 16);
for (const [name, protocol, y, band] of [['td', 1, 264, 1], ['tw', 2, 314, 2]]) {
  example('rf-internal-' + name, 'rf-internal-access', 'Select internal ' + name.toUpperCase() + ' mode.',
    [tap(762, 268), tap(310, y)], 0, 11, protocol, 16, { ['rf.modules.0.bands.' + band + '.enable']: 1 });
}
example('rf-external-twin', 'throttle-before', 'Enable the external Twin Lite Pro module in ACCESS mode.',
  [tap(772, 211), tap(715, 265)], 1, 10, 0, 16, { 'rf.modules.1.bands.0.enable': 1, 'rf.modules.1.bands.2.enable': 0 });
for (const [name, protocol, y, band0, band2] of [
  ['accst', 1, 240, 1, 0], ['elrs', 2, 290, 0, 1], ['tw', 3, 340, 1, 1],
]) {
  example('rf-twin-' + name, 'rf-external-twin', 'Select the Twin Lite Pro ' + name.toUpperCase() + ' protocol.',
    [tap(762, 385), tap(310, y)], 1, 10, protocol, 16,
    { 'rf.modules.1.bands.0.enable': band0, 'rf.modules.1.bands.2.enable': band2 });
}

for (const [id, label, type, count, y, scroll] of [
  ['xjt', 'XJT Lite', 2, 16, 165, false], ['r9-lite', 'R9M Lite', 5, 16, 215, false],
  ['r9-access', 'R9M Lite ACCESS', 6, 16, 265, false], ['r9-pro-access', 'R9M Lite Pro ACCESS', 7, 16, 315, false],
  ['ppm', 'PPM', 13, 8, 315, true], ['sbus', 'SBUS', 18, 16, 365, true],
]) {
  const select = [tap(762, 324)];
  if (scroll) select.push(['scroll']);
  select.push(tap(310, y));
  example('rf-external-' + id, 'rf-external-twin', 'Switch the external module type to ' + label + '.',
    select, 1, type, 0, count, { 'rf.modules.1.channelsStart': 0 });
}

for (const [id, seed, label, type, protocol, y, count] of [
  ['rf-xjt-d8', 'rf-external-xjt', 'D8', 2, 1, 265, 8],
  ['rf-xjt-lr12', 'rf-external-xjt', 'LR12', 2, 2, 315, 12],
  ['rf-r9-eu', 'rf-external-r9-lite', 'EU', 5, 1, 240, 16],
  ['rf-r9-flex868', 'rf-external-r9-lite', 'FLEX 868MHz', 5, 2, 290, 16],
  ['rf-r9-flex915', 'rf-external-r9-lite', 'FLEX 915MHz', 5, 3, 340, 16],
]) {
  example(id, seed, 'Select the external ' + label + ' protocol.',
    [tap(762, 385), tap(310, y)], 1, type, protocol, count);
}

for (const [id, seed, type, firstField, maximum, start, count] of [
  ['rf-ppm-min', 'rf-external-ppm', 13, false, false, 0, 4],
  ['rf-ppm-max', 'rf-external-ppm', 13, false, true, 0, 16],
  ['rf-ppm-last-four', 'rf-ppm-min', 13, true, true, 60, 4],
  ['rf-sbus-last', 'rf-external-sbus', 18, true, true, 48, 16],
]) {
  example(id, seed, 'Use the native channel selector to set CH' + (start + 1) + ' through CH' + (start + count) + '.',
    [tap(firstField ? 650 : 749, 384), tap(767, 448), tap(325, maximum ? 286 : 238)],
    1, type, 0, count, { 'rf.modules.1.channelsStart': start });
}

for (const [id, label, mode, y] of [
  ['hold', 'Hold', 1, 215], ['custom', 'Custom', 2, 265],
  ['no-pulses', 'No pulses', 3, 315], ['receiver', 'Receiver', 4, 365],
]) {
  example('rf-xjt-failsafe-' + id, 'rf-external-xjt', 'Select the XJT Lite D16 failsafe display mode ' + label + '.',
    [['scroll'], tap(762, 385), tap(310, y)], 1, 2, 0, 16, { 'rf.modules.1.failsafeMode': mode });
}

module.exports = { radioCases: cases };
