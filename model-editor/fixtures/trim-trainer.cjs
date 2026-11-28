'use strict';

const tap = (x,y) => ['tap',x,y];
const scroll = count => Array.from({length:count}, () => ['scroll']);
const model = [tap(706,391),tap(226,442)];
const pageTwo = [...model,['nextPage']];
const trims = [...model,tap(486,345)];
const extra = [...trims,...scroll(12),tap(205,278)];
const trainer = [...pageTwo,tap(482,348)];
const checklist = [...pageTwo,tap(318,172)];
const chart = [tap(706,391),tap(375,444),tap(220,89),tap(301,234)];
const overflow = tap(766,447);
const plus = tap(657,447);
const minus = tap(504,447);
const fs1 = {category:'function_switch',member:0};
const pickFS1 = [tap(405,302),tap(405,303),tap(35,32)];

const profile = (id, description, actions, readback, compare='default', seed) => ({
  id, description, compare, seed, fixture:{actions}, readback,
});

function trimState(id, description, actions, values, compare='extra-trim', seed) {
  return profile(id, description, actions,
    Object.fromEntries(Object.entries(values).map(([key,val]) => ['extraTrims.0.'+key, val])), compare, seed);
}

function trainerRole(master) {
  return [...trainer,tap(766,92),tap(315,master ? 262 : 311)];
}
const master = trainerRole(true);
const slave = trainerRole(false);
const masterChannel = [...master,tap(184,272)];
const links = ['cable','bluetooth','sbus','cppm'];
function linkActions(role, index) {
  return index === 0 ? role : [...role,tap(760,152),tap(367,[190,240,288,337][index])];
}

function trainerLinks() {
  const cases = [];
  for (const isMaster of [true,false]) {
    for (const [index,link] of links.entries()) {
      if (isMaster && index === 0) continue; // Already covered by trainer.bin.
      const name = 'trainer-' + (isMaster ? 'master-' : 'slave-') + link;
      const expected = {};
      if (index === 0) expected['trainerCable.settingsType'] = 'trainer_out';
      if (index === 1) expected['bluetooth.settingsType'] = isMaster ? 'trainerIn' : 'trainerOut';
      // With a disabled external RF module, 26.2.0 omits the external trainer setting.
      // The legacy menu/reread and role/link bytes establish those cases.
      const item = profile(name, 'Select '+(isMaster ? 'Master ' : 'Slave ')+link+' in the native Trainer menu.',
        linkActions(isMaster ? master : slave,index),expected,isMaster ? 'trainer' :
        (index === 0 ? 'default' : 'trainer-slave-cable'));
      item.legacyCheck = { offset:292, bytes:[isMaster ? 1 : 2,index], resave:true };
      cases.push(item);
    }
  }
  return cases;
}

const slaveSpan = (id, description, actions, start, count) =>
  profile(id, description, actions, {'trainerCable.trainer_out':{channelsStart:start,channelsCount:count}},
    'trainer-slave-cable');
const masterSetting = (id, description, actions, key, value) => profile(id, description, actions,
  {['trainerCable.trainer_in.channels.0.'+key]:value},'trainer');
const chartCase = (id, description, actions, values, seed='chart-widget') =>
  profile(id, description, actions, Object.fromEntries(Object.entries(values).map(([k,v]) =>
    ['screens.1.widgets.0.'+k,v])),seed,seed);

const trimTrainerCases = [
  profile('extra-trim','Add the first extra trim to a fresh model.',extra,
    {'extraTrims.length':1,'extraTrims.0.name':'','extraTrims.0.range':25,'extraTrims.0.step':2}),
  trimState('extra-trim-name','Name the extra trim AZ using the radio keyboard.',
    [...extra,tap(624,165),tap(81,354),tap(159,401),tap(733,450)],{name:'AZ'}),
  trimState('extra-trim-up','Select FS1 as the Up trigger.',[...extra,tap(760,224),...pickFS1],
    {triggerUp:fs1,triggerDown:null}),
  trimState('extra-trim-down','Select FS1 as the Down trigger.',[...extra,tap(760,271),...pickFS1],
    {triggerUp:null,triggerDown:fs1}),
  trimState('extra-trim-medium','Select the Medium extra trim step.',
    [...extra,tap(763,389),tap(336,313)],{step:3}),
  trimState('extra-trim-range-min','Select the minimum extra trim range.',
    [...extra,tap(707,330),overflow,tap(332,239)],{range:0}),
  trimState('extra-trim-range-max','Select the maximum extra trim range.',
    [...extra,tap(707,330),overflow,tap(332,288)],{range:100}),
  trimState('extra-trim-flight','Choose independent per flight mode.',
    [...extra,tap(763,447),tap(333,238)],{mode:1,lines:[{activeCondition:null,mode:1,value:0}]}),
  trimState('extra-trim-custom','Choose the Custom extra trim mode.',
    [...extra,tap(763,447),tap(333,287)],{mode:2,lines:[]}),
  trimState('extra-trim-off','Select the Off extra trim mode.',
    [...extra,tap(763,447),tap(333,339)],{mode:3,lines:[]}),
  profile('extra-trim-two','Append another extra trim after the named first row.',
    [...trims,...scroll(16),tap(205,278)],{'extraTrims.length':2,'extraTrims.0.name':'AZ','extraTrims.1.name':''},
    'extra-trim-name','extra-trim-name'),
  profile('trim-range-min','Select the minimum physical rudder-trim range.',
    [...trims,tap(705,152),overflow,tap(334,239)],{'trims.0.range':0}),
  profile('trim-range-max','Select the maximum physical rudder-trim range.',
    [...trims,tap(705,152),overflow,tap(334,289)],{'trims.0.range':100}),
  ...trainerLinks(),
  slaveSpan('trainer-slave-span','Set channels 3 through 11.',
    [...slave,tap(644,211),plus,plus,tap(745,209),plus],2,9),
  slaveSpan('trainer-slave-upper-start','Select the highest start for an eight-channel span.',
    [...slave,tap(644,211),overflow,tap(332,288)],56,8),
  slaveSpan('trainer-slave-count-min','Select the minimum output span.',
    [...slave,tap(745,209),overflow,tap(332,240)],0,4),
  slaveSpan('trainer-slave-count-max','Select the maximum output span.',
    [...slave,tap(745,209),overflow,tap(332,288)],0,16),
  masterSetting('trainer-master-weight-min','Select the minimum master channel 1 weight.',
    [...masterChannel,tap(710,210),overflow,tap(332,213)],'weight',-125),
  masterSetting('trainer-master-weight-max','Select the maximum master channel 1 weight.',
    [...masterChannel,tap(710,210),overflow,tap(332,263)],'weight',125),
  masterSetting('trainer-master-channel-add','Set master channel 1 to Add.',
    [...masterChannel,tap(763,151),tap(350,263)],'calculationMode',1),
  masterSetting('trainer-master-channel-off','Set master channel 1 to Off.',
    [...masterChannel,tap(763,151),tap(350,210)],'calculationMode',0),
  masterSetting('trainer-master-destination','Change the channel 1 destination to throttle.',
    [...masterChannel,tap(764,267),tap(500,263),tap(500,263),tap(35,32)],'destination',{category:'analog',member:2}),
  profile('trainer-master-condition','Choose FS1 as the master activation condition.',
    [...master,tap(765,210),...pickFS1],{'trainerCable.trainer_in.activeCondition':fs1},'trainer'),
  {...profile('trainer-off-from-bluetooth','Switch a master Bluetooth model to Off.',
    [...trainer,tap(765,92),tap(315,214)],{trainerCable:{mode:'none'}},'trainer-master-bluetooth','trainer-master-bluetooth'),
    legacyCheck:{offset:292,bytes:[0,1],resave:true}},
  profile('checklist-throttle-min','Select the minimum throttle threshold in the simulated checklist.',
    [...checklist,tap(748,92),overflow,tap(334,212)],{'preflightChecks.throttleValue':-100}),
  profile('checklist-throttle-max','Select the maximum throttle threshold in the simulated checklist.',
    [...checklist,tap(748,92),overflow,tap(334,263)],{'preflightChecks.throttleValue':100}),
  chartCase('chart-period-min','Select the minimum line-chart log period.',
    [...chart,tap(708,267),overflow,tap(332,239)],{interval:5}),
  chartCase('chart-period-max','Select the maximum line-chart log period.',
    [...chart,tap(708,267),overflow,tap(332,289)],{interval:100}),
  chartCase('chart-inverted','Enable chart inversion.',[...chart,tap(709,325)],{inverted:true}),
  chartCase('chart-auto','Enable automatic chart range.',[...chart,tap(709,384)],{autoRange:true}),
  chartCase('chart-manual','Return a previously saved automatic chart to manual range.',[...chart,tap(709,384)],
    {autoRange:false,minValue:-1024,maxValue:1024},'chart-auto'),
  chartCase('chart-min-high','Select the maximum possible manual minimum for an unassigned source.',
    [...chart,['scroll'],tap(707,384),overflow,tap(335,263)],{minValue:1024,maxValue:1024}),
  chartCase('chart-max-low','Select the minimum possible manual maximum for an unassigned source.',
    [...chart,['scroll'],tap(707,444),overflow,tap(334,213)],{minValue:-1024,maxValue:-1024}),
  chartCase('chart-min-step','Increase the lower chart bound by one.',[...chart,['scroll'],tap(707,384),plus],
    {minValue:-1023,maxValue:1024}),
  chartCase('chart-max-step','Decrease the upper chart bound by one.',[...chart,['scroll'],tap(707,444),minus],
    {minValue:-1024,maxValue:1023}),
];

for (const [suffix, app] of [['before',0x6803],['after',0x6802]]) {
  const base = 'throttle-' + suffix;
  const trimmed = 'extra-trim-sensors-' + suffix;
  const result = 'trainer-trim-sensors-' + suffix;
  trimTrainerCases.push(
    profile(trimmed, 'Add an extra trim to the existing four-sensor '+suffix+' model.',extra,
      {'extraTrims.length':1,'telemetry.sensors.length':4,'telemetry.sensors.3.key.appId':app},
      suffix === 'before' ? base : 'extra-trim-sensors-before',base),
    {...profile(result, 'Select external SBUS slave trainer on the extra-trim '+suffix+' model.',
      linkActions(slave,2),{'extraTrims.length':1,'telemetry.sensors.length':4,'telemetry.sensors.3.key.appId':app},
      suffix === 'before' ? trimmed : 'trainer-trim-sensors-before',trimmed),
      legacyCheck:{offset:307,bytes:[2,2],resave:true}},
  );
}

module.exports = { trimTrainerCases };
