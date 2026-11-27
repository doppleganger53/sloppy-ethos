'use strict';

// Touch positions are for the pinned 800x480 X20 Pro AW FCC 1.6.8 model menu.
const tap = (x,y) => ['tap',x,y];
const modelPage = [tap(706,391), tap(226,442), ['nextPage']];
const page = [...modelPage, tap(486,173)];
const picker = [...page, tap(400,270), tap(760,151)];
const advanced = [...picker, ['scroll'], ['scroll']];
const member = [tap(763,268), tap(500,303), tap(500,303), tap(500,303), tap(35,32)];
const plus = tap(657,447);
const minus = tap(504,447);
const ellipsis = tap(766,447);
const analog = {category:'analog', member:0};
const sa = {category:'switch_position', member:0};
const sb = {category:'switch_position', member:3};
const fs1 = {category:'function_switch', member:0};

function logic(id, key, values, actions, compare='logic-switch', expected={}, seed) {
  return {
    id, compare, seed, description:`Select or edit the legacy ${key} function (${id}).`,
    fixture:{actions}, readback:{'logicSwitches.0.key':key, 'logicSwitches.0.values':values, ...expected},
  };
}

const gt = [...picker,tap(325,263)];
const timer = [...advanced,tap(325,363)];
const and = [...advanced,tap(325,210),...member];
const addOperand = [...and,tap(580,327),tap(580,327)];
const editFirst = [...page,tap(184,144),tap(184,144),tap(320,163)];

const logicCases = [
  logic('logic-eq','eq',[analog,0],[...picker,tap(325,210)]),
  logic('logic-gt','gt',[analog,3],[...gt,tap(704,269),plus,plus,plus]),
  logic('logic-lt','lt',[analog,-2],[...picker,tap(325,313),tap(704,269),minus,minus]),
  logic('logic-gt-min','gt',[analog,-100],[...gt,tap(705,269),ellipsis,tap(334,211)],'logic-gt'),
  logic('logic-gt-max','gt',[analog,100],[...gt,tap(705,269),ellipsis,tap(334,262)],'logic-gt'),
  logic('logic-inverted','gt',[analog,0],[...gt,tap(399,152)],'logic-switch',{'logicSwitches.0.negative':true}),
  logic('logic-delay-on','gt',[analog,0],[...gt,tap(709,384),plus,plus],'logic-switch',{'logicSwitches.0.delayOn':2}),
  logic('logic-delay-off','gt',[analog,0],[...gt,tap(709,447),plus,plus,plus],'logic-switch',{'logicSwitches.0.delayOff':3}),
  logic('logic-and','and',[sa,sb],and),
  logic('logic-or','or',[sa,sb],[...advanced,tap(325,263),...member],'logic-and'),
  logic('logic-xor','xor',[sa,sb],[...advanced,tap(325,313),...member],'logic-and'),
  logic('logic-and-three','and',[sa,sb,null],addOperand,'logic-and'),
  logic('logic-and-fs1','and',[sa,sb,fs1],[...editFirst,tap(760,324),tap(405,302),tap(405,303),tap(35,32)],
    'logic-and-three',{},'logic-and-three'),
  logic('logic-timer','timer',[10,10],timer),
  logic('logic-timer-changed','timer',[13,8],[...timer,tap(705,210),plus,plus,plus,tap(704,269),minus,minus],'logic-timer'),
  logic('logic-timer-bounds','timer',[1,1750],[...timer,tap(705,210),ellipsis,tap(334,239),
    tap(705,269),ellipsis,tap(332,287)],'logic-timer'),
];

module.exports = { logicCases };
