'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { checkModel, digest } = require('../simulator/runtime.cjs');
const { multiWizardFlags } = require('./categories.cjs');

function corpus(directory) {
  const manifest = JSON.parse(fs.readFileSync(path.join(directory, 'manifest.json'), 'utf8'));
  const files = new Map();
  for (const entry of manifest.fixtures) {
    if (!/^[a-z0-9-]+\.bin$/.test(entry.file) || files.has(entry.file)) {
      throw new Error(`Invalid or duplicate fixture name: ${entry.file}`);
    }
    const bytes = fs.readFileSync(path.join(directory, entry.file));
    if (bytes.length !== entry.bytes || digest(bytes) !== entry.sha256) {
      throw new Error(`${entry.file}: file does not match the recorded snapshot length and SHA-256.`);
    }
    checkModel(bytes, '0.1.37');
    files.set(entry.file, bytes);
  }
  const found = fs.readdirSync(directory).filter(name => name.endsWith('.bin'));
  if (found.length !== files.size || found.some(name => !files.has(name))) {
    throw new Error(`${directory}: binary inventory and manifest differ.`);
  }
  return files;
}

// This applies exclusively to the named public fixtures and their measured byte offsets.
function compareCapture(name, original, fresh) {
  checkModel(original, '0.1.37');
  checkModel(fresh, '0.1.37');
  if (original.equals(fresh)) return null;
  const offset = multiWizardFlags[name];
  if (offset === undefined || fresh.length !== original.length ||
      original[16] !== 3 || fresh[16] !== 3 || offset >= fresh.length) {
    throw new Error(`${name}: the fresh fixture differs from the recorded bytes.`);
  }
  for (let index = 0; index < original.length; index++) {
    if (fresh[index] === original[index] || index === 14 || index === 15) continue;
    if (index !== offset || ((fresh[index] ^ original[index]) & 1) !== 0) {
      throw new Error(`${name}: unexpected change at offset ${index}.`);
    }
  }
  return { offset, before:original[offset], after:fresh[offset] };
}

function verifySeed(files) {
  const stock = 'category-multi.bin';
  for (const name of ['category-multi-before.bin','category-multi-after.bin']) {
    if (!files.has(name)) continue;
    if (!files.has(stock)) throw new Error(`${name}: missing source wizard seed ${stock}.`);
    if (files.get(name)[multiWizardFlags[name]] !== files.get(stock)[multiWizardFlags[stock]]) {
      throw new Error(`${name}: the complete throttle-cut flag byte must match its stock seed.`);
    }
  }
}

// Byte numbers are independent, measured locations in these four named public fixture pairs.
// They do not participate in inspection or editing of model files.
const appOffsets = {
  glider:[577,613,658,698], heli:[711,747,792,832],
  multi:[640,676,721,761], other:[367,403,448,488],
};

function verifyRepairPairs(files) {
  for (const [category, offsets] of Object.entries(appOffsets)) {
    const afterName = `category-${category}-after.bin`;
    if (!files.has(afterName)) continue;
    const before = files.get(`category-${category}-before.bin`);
    const after = files.get(afterName);
    if (!before || before.length !== after.length) throw new Error(`${afterName}: missing or differently sized baseline.`);
    const allowed = new Set([14,15,...offsets]);
    for (let i = 0; i < before.length; i++) {
      if (before[i] !== after[i] && !allowed.has(i)) {
        throw new Error(`${afterName}: unrelated byte at offset ${i} differs from the same-seed baseline.`);
      }
    }
    for (const i of offsets) {
      if (before[i-1] !== 3 || before.readUInt16LE(i) !== 0x6803 || after.readUInt16LE(i) !== 0x6802) {
        throw new Error(`${afterName}: unexpected S.Port ID pair at offset ${i}.`);
      }
    }
  }
}

function verifyReplay(reference, replay, partial = false) {
  const expected = corpus(reference);
  const observed = corpus(replay);
  if (!partial && expected.size !== observed.size) throw new Error('The replay fixture count differs.');
  verifySeed(expected);
  verifySeed(observed);
  verifyRepairPairs(expected);
  verifyRepairPairs(observed);
  const variations = [];
  for (const [name, fresh] of observed) {
    if (!expected.has(name)) throw new Error(`Unexpected replay fixture: ${name}`);
    const change = compareCapture(name, expected.get(name), fresh);
    if (change) variations.push({ file:name, ...change });
  }
  return { files:observed.size, variations };
}

if (require.main === module) {
  const [reference, replay, option, extra] = process.argv.slice(2);
  try {
    if (!reference || !replay || extra || (option && option !== '--partial')) {
      throw new Error('Usage: node verify-replay.cjs REFERENCE_DIR REPLAY_DIR [--partial]');
    }
    console.log(JSON.stringify(verifyReplay(reference, replay, option === '--partial'), null, 2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = { compareCapture, verifyReplay, verifySeed, verifyRepairPairs };
