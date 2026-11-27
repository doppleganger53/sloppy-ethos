'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { cases } = require('./cases.cjs');
const { changedRanges, checkModel, crc16, digest, format, resolveRuntime, runFirmware } = require('../simulator/runtime.cjs');

function canonicalFixture(bytes) {
  // The 26.1.2 firmware JSON independently identifies the final u32 as runTime, in scheduler ticks.
  const output = Buffer.from(bytes);
  output.writeUInt32LE(0, output.length - 4);
  output.writeUInt16LE(crc16(output.subarray(16)), 14);
  checkModel(output, '0.1.37');
  return output;
}

// For length changes record the differing span after removing the unchanged prefix/suffix.
function changes(before, after) {
  if (before.length === after.length) return { ranges: changedRanges(before, after) };
  let prefix = 16, suffix = 0;
  while (prefix < Math.min(before.length, after.length) && before[prefix] === after[prefix]) prefix++;
  while (suffix < Math.min(before.length, after.length) - prefix &&
         before[before.length - 1 - suffix] === after[after.length - 1 - suffix]) suffix++;
  return { headerRanges: changedRanges(before.subarray(0, 16), after.subarray(0, 16)),
    beforePayloadSpan: [prefix, before.length - suffix], afterPayloadSpan: [prefix, after.length - suffix] };
}

function selectedCases(ids) {
  if (!ids.length) return cases;
  const selected = new Set();
  const pending = [...ids];
  while (pending.length) {
    const current = pending.pop();
    if (!current || selected.has(current)) continue;
    const item = cases.find(candidate => candidate.id === current);
    if (!item) throw new Error(`Unknown fixture ${current}.`);
    selected.add(current);
    pending.push(item.compare, item.seed);
  }
  return cases.filter(item => selected.has(item.id));
}

async function generate(runtimeDir, outputDir, oracleDir, settingsOracleDir, caseIds) {
  const experiments = selectedCases(caseIds);
  const runtime = resolveRuntime(runtimeDir);
  if (runtime.release !== '1.6.8') throw new Error('The legacy fixture generator requires the pinned ETHOS 1.6.8 runtime.');
  const oracle = oracleDir && resolveRuntime(oracleDir);
  if (oracle && oracle.release !== '26.1.2') throw new Error('JSON cross-checks require the pinned 26.1.2 runtime.');
  const settingsOracle = settingsOracleDir && resolveRuntime(settingsOracleDir);
  if (settingsOracle && settingsOracle.release !== '26.2.0') throw new Error('Extended model JSON checks require 26.2.0.');
  fs.mkdirSync(outputDir); // New directory only, so an interrupted run cannot replace user data.
  const outputs = new Map();
  const manifest = {
    board: 'X20 Pro AW FCC', release: runtime.release, revision: runtime.revision,
    source: 'https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/1.6.8/X20PROAW-FCC-WebSimulator.zip',
    archiveSha256: runtime.archive, jsSha256: runtime.js, wasmSha256: runtime.wasm,
    inputFormat: '0.1.37', outputFormat: '0.1.37', normalized: 'Final four model runTime bytes zeroed and payload CRC recomputed after matching-firmware save.',
    fixtures: [],
  };
  for (const item of experiments) {
    const seed = item.seed && outputs.get(item.seed);
    if (item.seed && !seed) throw new Error(`${item.id}: required seed ${item.seed} has not been generated.`);
    const run = await runFirmware(runtime, { input: seed, fixture: item.fixture });
    const bytes = canonicalFixture(run.saved);
    if (item.id === 'default' && !bytes.equals(run.original)) {
      throw new Error('A default model save changed bytes outside the independently identified runtime field.');
    }
    if (item.id === 'default') {
      const repeat = await runFirmware(runtime, {});
      if (!canonicalFixture(repeat.saved).equals(bytes)) throw new Error('Default model was not reproducible.');
    }
    if (oracle) {
      const result = await runFirmware(oracle, { input: bytes, collectJson: true });
      if (!result.snapshot || result.snapshot.runTime !== 0) throw new Error(`${item.id}: normalized field failed independent JSON read-back.`);
      if (item.id.startsWith('throttle-') || ['sensor-variants', 'servo-addresses', 'id-conflict', 'layout-references'].includes(item.id)) {
        const observed = result.snapshot.telemetry.sensors;
        const live = run.accepted.filter(row => row.length === 6);
        if (observed.length !== live.length || live.some(([name, module, band, sub, physical, app], i) => {
          const row = observed[i], key = row?.key;
          return !key || row.name !== name || key.module !== Number(module) || key.band !== Number(band) ||
            key.subId !== Number(sub) || key.physId !== Number(physical) || key.appId !== Number(app);
        })) throw new Error(`${item.id}: persisted telemetry differs from the live firmware readings.`);
        if (item.id === 'layout-references') {
          const reference = result.snapshot.timers[0].startCondition;
          if (reference?.category !== 'telemetry_sensor' || reference.member !== 1) throw new Error('Timer source slot failed the persisted read-back.');
        }
      }
      const model = result.snapshot;
      switch (item.id) {
        case 'name-short':
        case 'name-long':
          assert.equal(model.name, run.accepted[0][0]);
          break;
        case 'timer':
          assert.deepEqual([model.timers[1].name, model.timers[1].mode, model.timers[1].startValue, model.timers[1].alarmValue],
            ['Flight timer', 1, 173, 37]);
          break;
        case 'channel':
          assert.deepEqual(model.channels[1], { name: 'Flap left', center: 3, invert: true, limitMin: -72, limitMax: 83 });
          break;
        case 'curve':
          assert.equal(model.curves[0].name, 'Bench rate');
          assert.equal(model.curves[0].type, 'none');
          break;
        case 'diy':
          assert.equal(model.telemetry.sensors[0].name, 'Bench DIY');
          break;
        case 'throttle-before': {
          const raw = await runFirmware(oracle, { input: run.saved, collectJson: true });
          const ticks = run.saved.readUInt32LE(run.saved.length - 4);
          assert.ok(ticks > 0);
          assert.equal(raw.snapshot.runTime, ticks);
          break;
        }
      }
      if (item.id.startsWith('calculated-') && result.snapshot.telemetry.sensors.length !== 1) {
        throw new Error(`${item.id}: expected one persisted calculated sensor.`);
      }
      if (item.id === 'calculated-operation' && result.snapshot.telemetry.sensors[0].operations?.length !== 1) {
        throw new Error('The custom calculated sensor did not retain the requested operation.');
      }
    }

    const entry = {
      file: item.id + '.bin', requested: item.description, accepted: run.accepted,
      bytes: bytes.length, sha256: digest(bytes), format: format(bytes), diagnostics: run.diagnostics,
    };
    if (item.readback) {
      entry.expectedSettings = item.readback;
      if (settingsOracle) {
        const result = await runFirmware(settingsOracle, { input: bytes, collectJson: true,
          fixture: item.fixture?.lua ? { lua: item.fixture.lua } : undefined });
        for (const [field, expected] of Object.entries(item.readback)) {
          const actual = field.split('.').reduce((value, part) => value?.[part], result.snapshot);
          assert.deepEqual(actual, expected, `${item.id}: unexpected persisted value at ${field}`);
        }
      }
    }
    if (item.compare) {
      entry.compareWith = item.compare + '.bin';
      entry.changes = changes(outputs.get(item.compare), bytes);
    }
    if (seed) {
      entry.seed = item.seed + '.bin';
      entry.seedSha256 = digest(seed);
    }
    fs.writeFileSync(path.join(outputDir, entry.file), bytes, { flag: 'wx' });
    manifest.fixtures.push(entry);
    outputs.set(item.id, bytes);
    console.log(entry.file + ' ' + entry.sha256);
  }
  fs.writeFileSync(path.join(outputDir, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n', { flag: 'wx' });
}

if (require.main === module) {
  const args = process.argv.slice(2);
  const caseIds = [];
  while (args.at(-2) === '--case') caseIds.push(args.splice(-2)[1]);
  const [runtimeDir, outputDir, oracleDir, settingsOracleDir, extra] = args;
  if (!runtimeDir || !outputDir || extra) {
    console.error('Usage: node generate.cjs LEGACY_RUNTIME NEW_OUTPUT_DIR [26.1.2_RUNTIME] [26.2.0_RUNTIME] [--case ID]');
    process.exitCode = 1;
  } else {
    generate(runtimeDir, outputDir, oracleDir, settingsOracleDir, caseIds).catch(error => { console.error(error.message); process.exitCode = 1; });
  }
}

module.exports = { canonicalFixture, changes };
