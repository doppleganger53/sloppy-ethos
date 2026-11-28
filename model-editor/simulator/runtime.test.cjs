'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { validate } = require('./validate.cjs');
const { checkModel, changedRanges, crc16, digest } = require('./runtime.cjs');
const { compareCapture, verifySeed } = require('../fixtures/verify-replay.cjs');

const fixtures = path.join(__dirname, '..', 'fixtures');
const input = path.join(fixtures, 'throttle-after.bin');
const manifest = JSON.parse(fs.readFileSync(path.join(fixtures, 'manifest.json'), 'utf8'));

function scratch(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'ethos-editor-validation-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}

test('fixture provenance, header, exact lengths, CRC and digests', () => {
  assert.equal(crc16(Buffer.from('123456789')), 0x31c3);
  assert.deepEqual(fs.readdirSync(fixtures).filter(name => name.endsWith('.bin')).sort(),
    manifest.fixtures.map(item => item.file).sort());
  for (const item of manifest.fixtures) {
    const bytes = fs.readFileSync(path.join(fixtures, item.file));
    assert.equal(bytes.length, item.bytes, item.file);
    assert.equal(digest(bytes), item.sha256, item.file);
    checkModel(bytes, '0.1.37');
  }
  const before = fs.readFileSync(path.join(fixtures, 'throttle-before.bin'));
  assert.deepEqual(changedRanges(before, fs.readFileSync(input)), [
    { start: 14, end: 16 }, { start: 366, end: 367 }, { start: 402, end: 403 },
    { start: 447, end: 448 }, { start: 487, end: 488 },
  ]);
});

test('missing and incompatible runtimes return diagnostics without executing selected JS', async t => {
  const directory = scratch(t);
  assert.equal((await validate([directory, input])).status, 'runtime-unavailable');
  fs.writeFileSync(path.join(directory, 'X20PROAW_FCC.js'), 'throw new Error("unexpected execution")');
  fs.writeFileSync(path.join(directory, 'X20PROAW_FCC.wasm'), 'unrecognized');
  assert.equal((await validate([directory, input])).status, 'runtime-incompatible');
});

test('Multi wizard replay accepts only the documented high bits on the exact capture', () => {
  const name = 'category-multi.bin';
  const original = fs.readFileSync(path.join(fixtures, name));
  function changed(bytes, offset, mask) {
    const result = Buffer.from(bytes);
    result[offset] ^= mask;
    result.writeUInt16LE(crc16(result.subarray(16)), 14);
    return result;
  }
  const alternate = changed(original, 517, 0x50);
  assert.deepEqual(compareCapture(name, original, alternate),
    { offset:517, before:original[517], after:alternate[517] });
  assert.throws(() => compareCapture(name, original, changed(original, 517, 1)), /offset 517/);
  assert.throws(() => compareCapture(name, original, changed(alternate, 518, 0x50)), /offset 518/);
  assert.throws(() => compareCapture('category-glider.bin', original, alternate), /differs/);
  const corrupt = Buffer.from(alternate);
  corrupt[14] ^= 4;
  assert.throws(() => compareCapture(name, original, corrupt), /CRC/);
  const before = fs.readFileSync(path.join(fixtures, 'category-multi-before.bin'));
  const pair = new Map([[name, original], ['category-multi-before.bin', changed(before, 528, 0x50)]]);
  assert.throws(() => verifySeed(pair), /complete throttle-cut flag byte/);
  pair.set(name, alternate);
  assert.doesNotThrow(() => verifySeed(pair));
});

test('corrupt checksum or unsupported source format is rejected before firmware startup', async t => {
  const directory = scratch(t);
  const corrupt = path.join(directory, 'corrupt.bin');
  const bytes = fs.readFileSync(input);
  bytes[14] ^= 1;
  fs.writeFileSync(corrupt, bytes);
  assert.match((await validate([directory, corrupt])).message, /CRC/);
  bytes[7] = 22;
  fs.writeFileSync(corrupt, bytes);
  const rejected = await validate([directory, corrupt]);
  assert.equal(rejected.status, 'rejected');
  assert.equal(rejected.sourceFormat, '0.1.22');
});

test('ETHOS 1.6.8 reopens and resaves an isolated copy', {
  skip: !process.env.ETHOS_EDITOR_SIM_168 && 'Set ETHOS_EDITOR_SIM_168 to the pinned extracted runtime.',
}, async t => {
  const directory = scratch(t);
  const readback = path.join(directory, 'readback.bin');
  const args = [process.env.ETHOS_EDITOR_SIM_168, input, '--readback', readback];
  const original = fs.readFileSync(input);
  const report = await validate(args);
  assert.equal(report.status, 'ok', report.message);
  assert.equal(report.sourceFormat, '0.1.37');
  assert.equal(report.resultFormat, '0.1.37');
  assert.equal(report.converted, false);
  const saved = fs.readFileSync(readback);
  assert.deepEqual(report.changedRanges, changedRanges(original, saved));
  checkModel(saved, '0.1.37');
  assert.equal((await validate(args)).status, 'error');
  assert.deepEqual(fs.readFileSync(readback), saved);
  assert.deepEqual(fs.readFileSync(input), original);
});

test('ETHOS 26.1.2 identifies a conversion on disposable storage', {
  skip: !process.env.ETHOS_EDITOR_SIM_2612 && 'Set ETHOS_EDITOR_SIM_2612 to the pinned extracted runtime.',
}, async () => {
  const report = await validate([process.env.ETHOS_EDITOR_SIM_2612, input]);
  assert.equal(report.status, 'converted', report.message);
  assert.equal(report.resultFormat, '0.2.22');
  assert.equal(report.converted, true);
});

test('ETHOS 26.2.0 identifies its own output format', {
  skip: !process.env.ETHOS_EDITOR_SIM_2620 && 'Set ETHOS_EDITOR_SIM_2620 to the pinned extracted runtime.',
}, async () => {
  const report = await validate([process.env.ETHOS_EDITOR_SIM_2620, input]);
  assert.equal(report.status, 'converted', report.message);
  assert.equal(report.resultFormat, '0.2.29');
});
