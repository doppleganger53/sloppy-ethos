'use strict';

const { fork } = require('node:child_process');
const { createHash } = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const runtimes = [
  {
    release: '1.6.8', revision: 'a6b30b5e', outputFormat: '0.1.37',
    archive: 'efe70e0e2c529951772108d8fe4535059273464276e740640963ba8d6cd0ab12',
    js: 'cbcc6a04ea0fbab56c7d19f8637d7f3fb95b0a99d25e9a2ffe5228ae9060e77d',
    wasm: 'a81ceb4de5c282a60da708986d6dd4c2afcf4871d848e75ac4f3eaa2562826ea',
  },
  {
    release: '26.1.2', revision: 'ca464ac3', outputFormat: '0.2.22',
    archive: '4e4c3588fce7a5ff8407a34cd41471cdf2016ea3ca7e48faca7353399f2fa82d',
    js: '52b138ee343606fb2ad98383fa366dedb238a66c446b47ab7a07288a48d1d32b',
    wasm: 'ef6a3e71ca1ec70a69be57f7c9cbe415097741ffc3f19b2493b7d6e8d5ca657d',
  },
  {
    release: '26.2.0', revision: '0a6b337c', outputFormat: '0.2.29',
    archive: '5df9c74f9267bccb121a7c145109d59ed89ae977a2deca5645c3be0a5528cbf9',
    js: 'ea834f4464f90a8bc9976342a8a03b91a85e0bc5d6d9c029de403af1fc88126b',
    wasm: 'aa10be07396b66e52f4c06096cfaedbabac9edd38698dfb9dcb84e746127f37d',
  },
];

function digest(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function failure(status, message) {
  return Object.assign(new Error(message), { status });
}

function resolveRuntime(directory) {
  const root = path.resolve(directory);
  const js = path.join(root, 'X20PROAW_FCC.js');
  const wasm = path.join(root, 'X20PROAW_FCC.wasm');
  let jsHash, wasmHash;
  try {
    jsHash = digest(fs.readFileSync(js));
    wasmHash = digest(fs.readFileSync(wasm));
  } catch {
    throw failure('runtime-unavailable', 'Choose the extracted X20 Pro AW FCC simulator directory containing both the .js and .wasm files.');
  }
  const pin = runtimes.find(item => item.js === jsHash && item.wasm === wasmHash);
  if (!pin) {
    throw failure('runtime-incompatible', 'The selected files do not match a verified X20 Pro AW FCC runtime. Use ETHOS 1.6.8, 26.1.2 or 26.2.0.');
  }
  return { ...pin, jsFile: js };
}

function crc16(payload) {
  let crc = 0;
  for (const byte of payload) {
    crc ^= byte << 8;
    for (let bit = 0; bit < 8; bit++) {
      crc = ((crc << 1) ^ ((crc & 0x8000) ? 0x1021 : 0)) & 0xffff;
    }
  }
  return crc;
}

function format(bytes) {
  return bytes.length >= 8 ? [bytes[5], bytes[6], bytes[7]].join('.') : null;
}

function checkModel(bytes, expectedFormat) {
  if (bytes.length < 16 || !bytes.subarray(0, 4).equals(Buffer.from('FRSK')) ||
      bytes[4] !== 1 || bytes.readUInt16LE(12) !== 0x1381) {
    throw failure('rejected', 'Expected an X20 Pro AW model with a complete FRSK header.');
  }
  if (format(bytes) !== expectedFormat) {
    throw failure('rejected', `Expected model format ${expectedFormat}, received ${format(bytes)}.`);
  }
  if (bytes.readUInt32LE(8) !== bytes.length - 16 ||
      bytes.readUInt16LE(14) !== crc16(bytes.subarray(16))) {
    throw failure('rejected', 'Model payload length or CRC-16/XMODEM does not match its header.');
  }
}

function changedRanges(before, after) {
  const result = [];
  let start = -1;
  const total = Math.max(before.length, after.length);
  for (let offset = 0; offset <= total; offset++) {
    if (offset < total && before[offset] !== after[offset]) {
      if (start === -1) start = offset;
    } else if (start !== -1) {
      result.push({ start, end: offset });
      start = -1;
    }
  }
  return result;
}

// Run each model in its own firmware process and unmounted, in-memory radio filesystem.
function runFirmware(runtime, request) {
  return new Promise((resolve, reject) => {
    const child = fork(path.join(__dirname, 'worker.cjs'), [], {
      stdio: ['ignore', 'ignore', 'ignore', 'ipc'], serialization: 'advanced',
      execArgv: [], windowsHide: true,
    });
    const timer = setTimeout(() => finish(failure('error', 'ETHOS did not return a persisted model within 30 seconds.')), 30000);
    let settled = false;
    function finish(error, value) {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      child.kill();
      if (error) reject(error);
      else resolve(value);
    }
    child.once('error', finish);
    child.once('exit', code => finish(failure('error', `ETHOS stopped before producing a validation result (exit ${code}).`)));
    child.once('message', response => {
      if (response.error) finish(failure(response.status || 'error', response.error));
      else finish(null, response);
    });
    child.send({ runtime, ...request }, error => { if (error) finish(error); });
  });
}

module.exports = { runtimes, digest, failure, resolveRuntime, format, crc16, checkModel, changedRanges, runFirmware };
