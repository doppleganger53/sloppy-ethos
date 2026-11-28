'use strict';

const { setTimeout: delay } = require('node:timers/promises');
const { checkModel, failure } = require('./runtime.cjs');

const fixtureIcon = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAAAAADhZOFXAAAADklEQVR4nGP4DwUMlDEA98A/wbI0QbsAAAAASUVORK5CYII=',
  'base64',
);

async function until(check, message) {
  const deadline = Date.now() + 8000;
  while (Date.now() < deadline) {
    const value = check();
    if (value) return value;
    await delay(25);
  }
  throw failure('error', message);
}

async function act(firmware, action) {
  const [kind, x, y] = action;
  if (kind === 'tap') {
    firmware._onMouseDown(x, y);
    await delay(100);
    firmware._onMouseUp(x, y);
  } else if (kind === 'nextPage' || kind === 'scroll') {
    const horizontal = kind === 'nextPage';
    const start = horizontal ? [696, 230] : [415, 355];
    const end = horizontal ? [100, 230] : [415, 147];
    firmware._onMouseDown(...start);
    await delay(90);
    for (let step = 1; step <= 10; step++) {
      const x = Math.round(start[0] + (end[0] - start[0]) * step / 10);
      const y = Math.round(start[1] + (end[1] - start[1]) * step / 10);
      firmware._onMouseMove(x, y);
      await delay(30);
    }
    firmware._onMouseUp(...end);
    await delay(280);
  } else {
    throw failure('error', 'Unknown fixture simulator action.');
  }
  await delay(135);
}

async function execute({ runtime, input, fixture, collectJson }) {
  const logs = [];
  let snapshot;
  const log = value => logs.push(String(value).replace(/\x1b\[[0-9;]*m/g, '').replace(/\r$/, ''));
  const factory = require(runtime.jsFile);
  const firmware = await factory({
    print: log, printErr: log,
    updateCanvas() {}, updateCanvasTop() {}, updateCanvasGlasses() {},
    setModelJson: json => { snapshot = JSON.parse(json); },
  });
  const root = firmware.ccall('getCurrentModel', 'string').replace(/\/$/, '');
  const modelPath = root + '/models/model00.bin';
  // A development fixture may create the second model through the native wizard.
  const outputName = fixture?.createsModel ? 'model01.bin' : 'model00.bin';
  const outputPath = root + '/models/' + outputName;
  firmware.FS.mkdirTree(root + '/models');
  firmware._writeDefaultSettingsAndModel();
  const original = Buffer.from(firmware.FS.readFile(modelPath));
  if (input) firmware.FS.writeFile(modelPath, input);
  if (fixture?.lua) {
    const scripts = root + '/scripts/FxLab';
    firmware.FS.mkdirTree(scripts);
    firmware.FS.writeFile(scripts + '/icon.png', fixtureIcon);
    firmware.FS.writeFile(scripts + '/main.lua', fixture.lua);
  }

  logs.length = 0;
  if (fixture?.actions?.length) firmware._setAnalogPosition(2, -1024);
  firmware._start();
  const readFailure = () => logs.find(line =>
    /Model load error|ModelData::read.*(?:failed|error)|model file.*(?:invalid|cannot)|invalid model|corrupt(?:ed)? model/i.test(line));
  await until(() => readFailure() || logs.some(line => line.includes('Home::doPreflightChecks()')),
    'The firmware did not confirm opening the input model.');
  if (readFailure()) throw failure('rejected', readFailure());
  if (!logs.some(line => line.includes(`X20PROAW Ethos ${runtime.release} @ ${runtime.revision} started!`)) ||
      !logs.some(line => line.includes("ModelData::read('USER:/models/model00.bin')"))) {
    throw failure('error', 'ETHOS did not confirm the selected board, firmware revision and model path.');
  }

  if (fixture?.actions?.length) {
    // The initial preflight dialog and the touch targets below finish rendering after the load message.
    await delay(335);
    for (const action of fixture.actions) await act(firmware, action);
    if (fixture.createsModel) {
      const created = () => firmware.FS.readdir(root + '/models').includes(outputName);
      for (let page = 0; page < 12 && !created(); page++) {
        await act(firmware, ['tap', 733, 450]);
      }
      if (!created()) throw failure('error', 'The firmware wizard did not create the requested model.');
    }
    if (fixture.lua && !logs.includes('FIXTURE OK')) {
      const diagnostic = logs.find(line => /lua.*error|main\.lua:\d+:/i.test(line));
      throw failure('error', diagnostic || 'The firmware did not confirm the requested fixture action.');
    }
  }
  if (collectJson) {
    if (!firmware._setModelJsonNeeded) throw failure('error', 'This release does not export model JSON.');
    firmware._setModelJsonNeeded(1);
    await until(() => snapshot, 'The firmware did not return its decoded model.');
    firmware._setModelJsonNeeded(0);
  }

  const beforeStop = logs.length;
  firmware._stop();
  await until(() => logs.slice(beforeStop).some(line => line.includes(`@ ${runtime.revision} stopped!`)),
    'ETHOS did not finish shutting down its disposable model.');
  if (!logs.slice(beforeStop).some(line => line.includes(`ModelData::write('USER:/models/${outputName}')`))) {
    throw failure('error', 'No fresh firmware write of the selected model was observed.');
  }
  const saved = Buffer.from(firmware.FS.readFile(outputPath));
  checkModel(saved, runtime.outputFormat);
  const diagnostics = logs.filter(line => /ModelData::(?:read|write)|will be converted|Ethos .* (started|stopped)!/.test(line))
    .map(line => line.replace(/^\d+ /, ''));
  const warnings = [...new Set(logs.filter(line => /warning|not found|unavailable|missing/i.test(line) &&
    !line.includes("playFile(") && !line.includes('f_opendir(')).map(line => line.replace(/^\d+ /, '')))];
  return {
    original, saved, warnings, diagnostics: [...new Set(diagnostics)], snapshot,
    accepted: logs.filter(line => line.startsWith('FX\t')).map(line => line.slice(3).split('\t')),
  };
}

process.once('message', request => {
  execute(request).then(
    result => process.send(result),
    error => process.send({ error: error.message, status: error.status || 'error' }),
  );
});
