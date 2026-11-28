'use strict';

const targets = [
  {id:'glider', label:'Glider', x:240, storedType:1, oracleType:1, stockMixes:3},
  {id:'heli', label:'Heli', x:400, storedType:2, oracleType:2, stockMixes:6},
  {id:'multi', label:'Multi', x:558, storedType:3, oracleType:3, stockMixes:5},
  {id:'other', label:'Other', x:710, storedType:4, oracleType:99, stockMixes:0},
];

// Verified synthetic observations, used only to compare independent fixture runs.
const multiWizardFlags = Object.freeze({
  'category-multi.bin': 517, 'category-multi-before.bin': 528,
  'category-multi-after.bin': 528, 'category-multi-channel.bin': 517,
});

function categoryCases(tool) {
  const result = [];
  const options = {
    glider: {id:'category-glider-flaps', description:'Select four flap channels in the glider wizard.',
      actions:[['tap',733,450], ['tap',733,450], ['tap',550,394], ['tap',350,338]],
      readback:{flapsCount:4,'channels.5.name':'Flap1','mixes.length':4}},
    heli: {id:'category-heli-swash', description:'Choose the three-servo 135 degree swash assembly in the Heli wizard.',
      actions:[['tap',733,450], ['tap',625,288], ['tap',350,363]],
      readback:{swashType:4,'channels.0.name':'CYC1','mixes.length':5}},
    multi: {id:'category-multi-channel', description:'Select channel 6 for Roll in the Multi wizard.',
      actions:[['tap',733,450], ['tap',80,355], ['scroll'], ['tap',350,214]],
      readback:{'channels.5.name':'Roll','mixes.length':5}},
  };
  for (const target of targets) {
    const stock = 'category-' + target.id;
    const before = stock + '-before';
    result.push({
      id:stock, compare:'default', category:target.storedType,
      description:'Create a fresh '+target.label+' model using all default choices in the native 1.6.8 wizard.',
      fixture:{createsModel:true, actions:[
        ['tap',706,391], ['tap',226,442], ['tap',150,167], ['tap',238,193],
        ['tap',362,236], ['tap',target.x,224],
      ]},
      readback:{type:target.oracleType, 'mixes.length':target.stockMixes, 'telemetry.sensors.length':0},
    });
    for (const [suffix, application] of [['before',0x6803],['after',0x6802]]) {
      result.push({
        id:stock+'-'+suffix, compare:suffix==='before' ? stock : before, seed:stock,
        category:target.storedType, verifyTelemetry:true,
        description:'Open the stock '+target.label+' model, name it and create four S.Port measurements at Physical 03 and Application '+application.toString(16)+'.',
        fixture:tool(`model.name("Bench ${target.label}")
for _,entry in ipairs({{0,"Throttle amps"},{1,"Throttle volts"},{2,"Throttle temp"},{4,"Throttle pos"}}) do
  local sensor=assert(model.createSensor({type=SENSOR_TYPE_SPORT,module=0,band=1,
    subId=entry[1],physId=3,appId=${application}}))
  sensor:name(entry[2])
  print("FX",sensor:name(),sensor:module(),sensor:band(),sensor:subId(),sensor:physId(),sensor:appId())
end`),
        readback:{type:target.oracleType, name:'Bench '+target.label, 'mixes.length':target.stockMixes,
          'telemetry.sensors.length':4, 'telemetry.sensors.3.key.appId':application},
      });
    }
    const variation = options[target.id];
    if (variation) {
      const wizard = result.find(item => item.id === stock).fixture;
      result.push({
        id:variation.id, compare:stock, category:target.storedType, description:variation.description,
        fixture:{createsModel:true, actions:[...wizard.actions,...variation.actions]},
        readback:{type:target.oracleType,...variation.readback},
      });
    }
  }
  return result;
}

module.exports = { categoryCases, multiWizardFlags };
