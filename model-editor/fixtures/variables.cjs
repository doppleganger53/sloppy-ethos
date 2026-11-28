'use strict';

const tap = (x,y) => ['tap',x,y];
const scroll = ['scroll'];
const page = [tap(706,391), tap(226,442), ['nextPage'], tap(315,338)];
const create = [...page, tap(400,270)];
const edit = [...page, tap(183,139), tap(183,139), tap(302,189), scroll, scroll];
const plus = tap(657,447);
const chooseSource = [tap(405,302), tap(405,303), tap(35,32)];
const newRate = [...create, tap(602,425), scroll];
const newAction = [...create, scroll, scroll, tap(604,451), scroll, scroll];
const conditionAndValue = [tap(241,404), ...chooseSource, tap(700,404), plus, plus, plus];
const sa = {category:'switch_position', member:0};
const fs1 = {category:'function_switch', member:0};
const emptyRate = {condition:null, value:0};
const switchedRate = {condition:sa, value:3};
const action = (condition, op, value, step=10) => ({condition, function:op, source:value, step});
const emptyAction = action(null,1,0);
const add = action(fs1,1,3);

function variable(id, description, actions, rates, actionList, compare='variable', seed) {
  return { id, description, compare, seed, fixture:{actions},
    readback:{'vars.0.values.rates':rates, 'vars.0.actions':actionList} };
}

const actionsByName = [
  ['assign',0,164,false], ['subtract',2,263,false], ['multiply',3,313,false],
  ['divide',4,363,false], ['percent',5,214,true], ['minimum',6,263,true],
  ['maximum',7,313,true], ['repurpose',8,363,true],
];
const trim = (member) => ({category:'trim', member});
const menuActions = actionsByName.map(([name, op, row, secondPage]) =>
  variable('var-'+name, 'Select the '+name+' operator on the existing FS1 variable action.',
    [...edit, tap(413,405), ...(secondPage ? [scroll] : []), tap(332,row)], [],
    [action(fs1,op,op === 8 ? trim(0) : 3)], 'var-action-switch', 'var-action-switch'));
const stepMenu = [...edit, tap(754,404), tap(766,447)];
const variableCases = [
  variable('var-rate-empty','Append the first conditional value.',newRate,[emptyRate],[]),
  variable('var-rate-switch','Select SA Up and set a rate of 0.3%.',
    [...newRate,tap(409,296),...chooseSource,tap(700,296),plus,plus,plus],[switchedRate],[],'var-rate-empty'),
  variable('var-rate-two','Append a second, blank rate without changing the first.',[...edit,tap(602,346)],
    [switchedRate,emptyRate],[],'var-rate-switch','var-rate-switch'),
  variable('var-rate-dropped','Confirm removal of the first rate. The previously second rate remains.',
    [...edit,tap(33,249),tap(523,288)],[emptyRate],[],'var-rate-two','var-rate-two'),
  variable('var-action-empty','Append the first default Add action.',newAction,[],[emptyAction]),
  variable('var-action-switch','Select FS1 and set the Add source to 0.3.',
    [...newAction,...conditionAndValue],[],[add],'var-action-empty'),
  variable('var-action-two','Append a second, blank Add action without changing the first.',
    [...edit,tap(604,451)],[],[add,emptyAction],'var-action-switch','var-action-switch'),
  variable('var-action-dropped','Confirm removal of the first action. The previously second action remains.',
    [...edit,tap(33,355),tap(523,288)],[],[emptyAction],'var-action-two','var-action-two'),
  variable('var-combined','Keep the existing SA rate while appending an FS1 Add action.',
    [...edit,tap(604,451),scroll,scroll,...conditionAndValue],[switchedRate],[add],'var-rate-switch','var-rate-switch'),
  ...menuActions,
  variable('var-repurpose-elevator','Select Elevator trim and a 1.2 step for the existing Repurpose action.',
    [...edit,tap(610,404),tap(336,212),tap(754,404),plus,plus],[],[action(fs1,8,trim(1),12)],
    'var-repurpose','var-repurpose'),
  variable('var-step-min','Use the legacy numeric menu to select the minimum Repurpose step.',
    [...stepMenu,tap(332,211)],[],[action(fs1,8,trim(0),-1000)],'var-repurpose','var-repurpose'),
  variable('var-step-max','Use the legacy numeric menu to select the maximum Repurpose step.',
    [...stepMenu,tap(332,263)],[],[action(fs1,8,trim(0),1000)],'var-repurpose','var-repurpose'),
];

module.exports = { variableCases };
