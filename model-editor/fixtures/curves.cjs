'use strict';

// Each case is built by the actual 1.6.8 Lua API or its Model > Mixes UI.
// A seeded case edits an existing firmware-written model and keeps its other sections intact.
function curveCases(tool) {
  const initial = [[-100, -100], [22, 32], [100, 100]];
  const step = [[-100, -100], [-30, -75], [-30, 67.5], [50, 90], [100, 100]];
  const endpoints = [[-100, -100], [100, 100]];
  const easy = [[-100, -100], [0, -12.5], [100, 100]];
  const maximum = Array.from({length: 21}, (_, index) => [-100 + index * 10, -100 + index * 10]);
  const name = 'Bench curve';

  function points(id, compare, coordinates, easyMode, smooth, seed) {
    const values = coordinates.map(([x, y]) => `{${x},${y}}`).join(',');
    const start = seed
      ? 'local c=assert(model.getCurve("Bench curve"))'
      : 'local c=assert(model.createCurve())\nc:name("Bench curve")\nc:type(CURVE_TYPE_CUSTOM)';
    const readback = {
      'curves.0.type': 'custom', 'curves.0.name': name, 'curves.0.easyMode': easyMode,
      'curves.0.smooth': smooth, 'curves.0.points': coordinates.map(([x, y]) => ({x,y})),
    };
    if (seed) readback['mixes.0.actions.1.curve.defaultCurve'] = {index: 0, type: 'curve'};
    return {
      id, compare, seed, description: `Set the ${coordinateLabel(easyMode)} curve to ${coordinates.length} points, smooth ${smooth ? 'on' : 'off'}.`,
      fixture: tool(`${start}
c:pointsCount(${coordinates.length})
c:easyMode(${easyMode})
c:smooth(${smooth})
for i,p in ipairs({${values}}) do
  c:point(i-1,p[1],p[2])
  local x,y=c:point(i-1)
  print("FX",i-1,x,y)
end
print("FX",c:name(),c:pointsCount(),c:easyMode(),c:smooth())`),
      readback,
    };
  }

  function coordinateLabel(easyMode) {
    return easyMode ? 'equally spaced X' : 'custom X';
  }

  const referenced = {
    id: 'curve-referenced', compare: 'curve-custom', seed: 'curve-custom',
    description: 'Create a free mix and select the existing Bench curve in its second action.',
    fixture: {actions: [
      ['tap',706,391], ['tap',226,442], ['tap',652,169], ['tap',400,270], ['tap',96,98],
      ['tap',408,408], ['tap',507,152], ['tap',322,164], ['tap',502,210], ['tap',350,287],
    ]},
    readback: {
      'curves.0.points': initial.map(([x,y]) => ({x,y})),
      'mixes.0.actions.0.key':'weight', 'mixes.0.actions.1.key':'curve',
      'mixes.0.actions.1.curve.defaultCurve': {index:0, type:'curve'},
    },
  };
  const functions = ['NONE','X_POSITIVE','X_NEGATIVE','X_ABS','FUNC_POSITIVE','FUNC_NEGATIVE','FUNC_ABS'];
  const enterCurves = [['tap',706,391], ['tap',226,442], ['nextPage'], ['tap',150,346]];
  const beginMove = [...enterCurves, ['tap',182,145], ['tap',182,145], ['tap',326,260]];
  const moveOne = [...beginMove, ['tap',245,184]];
  const librarySelectors = (index) => ({
    'mixes.0.actions.1.curve.defaultCurve': index === null ? null : {index, type:'curve'},
    'mixes.0.actions.1.curve.multiCurves.0.curve': index === null ? null : {index, type:'curve'},
    'mixes.0.actions.1.curve.multiCurves.0.activeCondition': {category:'function_switch', member:0},
  });
  return [
    points('curve-easy', 'default', easy, true, false),
    points('curve-smooth', 'curve-easy', easy, true, true),
    points('curve-custom', 'default', initial, false, false),
    points('curve-stepped', 'curve-custom', step, false, false),
    points('curve-maximum', 'curve-easy', maximum, true, false),
    referenced,
    points('curve-ref-five', 'curve-referenced', step, false, false, 'curve-referenced'),
    points('curve-ref-two', 'curve-ref-five', endpoints, false, false, 'curve-ref-five'),
    {
      id:'curve-expo', compare:'default', description:'Create signed Expo curves at ±100 and one with fractional offset.',
      fixture: tool(`for _,r in ipairs({{"Lower",-100,-100,-100},{"Upper",100,100,100},{"Fractional",-12.5,-37,75},
  {"Negative",-125.5,-125,-125},{"Positive",125.5,125,125}}) do
  local c=model.createCurve()
  c:type(CURVE_TYPE_EXPO)
  c:name(r[1])
  c:offset(r[2])
  c:exponent(r[3])
  c:weight(r[4])
  print("FX",c:name(),c:offset(),c:exponent(),c:weight())
end`),
      readback: { 'curves.0.offset':-1000,'curves.0.expo':-100,'curves.0.weight':-100,
        'curves.1.offset':1000,'curves.1.expo':100,'curves.1.weight':100,
        'curves.2.offset':-125,'curves.2.expo':-37,'curves.2.weight':75,
        'curves.3.offset':-1255,'curves.3.expo':-125,'curves.3.weight':-125,
        'curves.4.offset':1255,'curves.4.expo':125,'curves.4.weight':125 },
    },
    {
      id:'curve-bounds', compare:'curve-custom',
      description:'Request ±125.5% Y and fractional X with the legacy Lua API, then record its clamping and precision.',
      fixture: tool(`local c=model.createCurve()
c:name("Bench curve")
c:type(CURVE_TYPE_CUSTOM)
c:pointsCount(3)
c:easyMode(false)
for i,p in ipairs({{-100,-125.5},{22.5,-32.5},{100,125.5}}) do
  c:point(i-1,p[1],p[2])
  local x,y=c:point(i-1)
  print("FX",i-1,x,y)
end`),
      readback:{'curves.0.points':[{x:-100,y:-100},{x:22,y:-32.5},{x:100,y:100}]},
    },
    {
      id:'curve-functions', compare:'default', description:'Exercise all seven native function-curve menu encodings.',
      fixture: tool(`for index=0,6 do
  local c=model.createCurve()
  c:type(CURVE_TYPE_FUNCTION)
  c:name("Function"..index)
  c:functionType(index)
  print("FX",c:name(),c:functionType())
end`),
      readback: Object.fromEntries(functions.map((label,i) => [`curves.${i}.function`, `CURVE_FUNCTION_${label}`])),
    },
    {
      id:'curve-mix-inline', compare:'curve-referenced', seed:'curve-custom',
      description:'Choose the inline Expo option in the free-mix curve action.',
      fixture:{actions:[...referenced.fixture.actions.slice(0,-1), ['tap',350,238]]},
      readback:{'mixes.0.actions.1.curve.defaultCurve':{type:'expo',expo:0}},
    },
    {
      id:'curve-mix-rate', compare:'curve-referenced', seed:'curve-custom',
      description:'Add FS1 as a conditional selector for the same named curve.',
      fixture:{actions:[...referenced.fixture.actions, ['tap',383,256], ['tap',222,255],
        ['tap',405,302], ['tap',405,303], ['tap',35,32], ['tap',500,256], ['tap',350,287]]},
      readback:librarySelectors(0),
    },
    {
      id:'curve-library', compare:'curve-mix-rate', seed:'curve-mix-rate',
      description:'Append four named function curves after a curve referenced twice by an existing mix.',
      fixture:tool(`for i=1,4 do
  local c=model.createCurve()
  c:type(CURVE_TYPE_FUNCTION)
  c:name("Spare "..i)
  c:functionType(i)
  print("FX",c:name(),c:functionType())
end`),
      readback:{...librarySelectors(0),'curves.length':5,'curves.0.name':'Bench curve','curves.4.name':'Spare 4'},
    },
    {
      id:'curve-library-moved-one', compare:'curve-library', seed:'curve-library',
      description:'Move the referenced curve down by one slot in the legacy Curves menu.',
      fixture:{actions:moveOne}, readback:{...librarySelectors(1),'curves.0.name':'Spare 1','curves.1.name':'Bench curve'},
    },
    {
      id:'curve-library-moved', compare:'curve-library', seed:'curve-library',
      description:'Move the first, referenced curve down through all four neighbours.',
      fixture:{actions:[...moveOne, ['tap',245,236], ['tap',245,286], ['tap',245,336]]},
      readback:{...librarySelectors(4),'curves.0.name':'Spare 1','curves.4.name':'Bench curve'},
    },
    {
      id:'curve-library-deleted', compare:'curve-library-moved', seed:'curve-library-moved',
      description:'Confirm removal of the referenced fifth curve in the isolated radio and read back both mix selectors.',
      fixture:{actions:[...enterCurves, ['tap',180,336], ['tap',180,336], ['scroll'], ['tap',316,365], ['tap',505,316]]},
      readback:{...librarySelectors(null),'curves.length':4,'curves.3.name':'Spare 4','mixes.0.actions.1.key':'curve'},
    },
  ];
}

module.exports = { curveCases };
