'use strict';

const fs = require('node:fs');
const { changedRanges, checkModel, failure, format, resolveRuntime, runFirmware } = require('./runtime.cjs');

function writeExclusive(filename, bytes) {
  const fd = fs.openSync(filename, 'wx');
  try {
    fs.writeFileSync(fd, bytes);
    fs.fsyncSync(fd);
  } catch (error) {
    fs.closeSync(fd);
    fs.unlinkSync(filename);
    throw error;
  }
  fs.closeSync(fd);
}

async function validate(args) {
  const report = {
    status: 'error', sourceFormat: null, resultFormat: null, converted: false,
    runtime: null, warnings: [], changedRanges: [], diagnostics: [], message: '',
  };
  try {
    if (args.length !== 2 && !(args.length === 4 && args[2] === '--readback')) {
      throw failure('error', 'Usage: node validate.cjs RUNTIME_DIRECTORY INPUT.bin [--readback NEW_FILE.bin]');
    }
    const [directory, inputPath] = args;
    if (!fs.statSync(inputPath).isFile()) throw failure('rejected', 'The input must be a model file.');
    const input = fs.readFileSync(inputPath);
    report.sourceFormat = format(input);
    checkModel(input, '0.1.37');
    const runtime = resolveRuntime(directory);
    report.runtime = `X20 Pro AW FCC, ETHOS ${runtime.release} @ ${runtime.revision}`;
    const result = await runFirmware(runtime, { input });
    report.resultFormat = format(result.saved);
    report.converted = report.sourceFormat !== report.resultFormat;
    report.changedRanges = changedRanges(input, result.saved);
    report.warnings = result.warnings;
    report.diagnostics = result.diagnostics;
    if (args.length === 4) writeExclusive(args[3], result.saved);
    report.status = report.converted ? 'converted' : 'ok';
    report.message = report.converted
      ? `ETHOS loaded the model and converted its disposable copy to ${report.resultFormat}.`
      : `ETHOS loaded and resaved the disposable copy in ${report.resultFormat}.`;
  } catch (error) {
    report.status = error.status || 'error';
    report.message = error.message;
  }
  return report;
}

if (require.main === module) {
  validate(process.argv.slice(2)).then(report => {
    process.stdout.write(JSON.stringify(report) + '\n');
    process.exitCode = ['ok', 'converted'].includes(report.status) ? 0 : 1;
  });
}

module.exports = { validate };
