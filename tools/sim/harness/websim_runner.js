#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

function parseArgs(argv) {
  const args = {};
  for (let index = 2; index < argv.length; index += 1) {
    const key = argv[index];
    if (!key.startsWith("--")) continue;
    args[key.slice(2)] = argv[index + 1];
    index += 1;
  }
  return args;
}

function ensureDir(FS, targetPath) {
  const parts = targetPath.split("/").filter(Boolean);
  let current = "";
  for (const part of parts) {
    current += `/${part}`;
    try {
      FS.mkdir(current);
    } catch (_error) {
      // Existing directories are fine; later writes will surface real failures.
    }
  }
}

function writeHostTree(FS, hostRoot, relative = "") {
  const current = path.join(hostRoot, relative);
  for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
    const childRelative = path.join(relative, entry.name);
    const simPath = `/${childRelative.replaceAll(path.sep, "/")}`;
    if (entry.isDirectory()) {
      ensureDir(FS, simPath);
      writeHostTree(FS, hostRoot, childRelative);
      continue;
    }
    if (entry.isFile()) {
      ensureDir(FS, path.posix.dirname(simPath));
      FS.writeFile(simPath, fs.readFileSync(path.join(hostRoot, childRelative)));
    }
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function unique(values) {
  return [...new Set(values)];
}

function findScriptErrors(lines) {
  const patterns = [
    /\bSLERR\b/i,
    /\bPANIC\b/i,
    /\babort\b/i,
    /\bsyntax error\b/i,
    /\bstack traceback\b/i,
    /\bscript\b.*\berror\b/i,
    /\blua\b.*\berror\b/i,
    /\bRuntimeError\b/i,
  ];
  return unique(lines.filter((line) => patterns.some((pattern) => pattern.test(line))));
}

async function main() {
  const args = parseArgs(process.argv);
  const runtimeJs = path.resolve(args["runtime-js"]);
  const runtimeDir = path.resolve(args["runtime-dir"]);
  const persist = path.resolve(args.persist);
  const project = args.project || "SensorList";
  const startupMs = Number(args["startup-ms"] || 1000);
  const settleMs = Number(args["settle-ms"] || 1500);
  const stdout = [];
  const stderr = [];
  let canvasUpdates = 0;
  let modelJsonCallbacks = 0;
  const progress = (message) => process.stderr.write(`[sim-harness] ${message}\n`);

  const result = {
    status: "startup_failure",
    project,
    runtimeJs,
    persist,
    started: false,
    reloaded: false,
    canvasUpdates: 0,
    modelJsonCallbacks: 0,
    errors: [],
    messages: [],
  };

  try {
    progress("loading runtime");
    const factory = require(runtimeJs);
    progress("initializing runtime");
    const module = await factory({
      noInitialRun: true,
      locateFile: (fileName) => path.join(runtimeDir, fileName),
      print: (line) => stdout.push(String(line)),
      printErr: (line) => stderr.push(String(line)),
      updateCanvas: () => {
        canvasUpdates += 1;
      },
      setModelJson: () => {
        modelJsonCallbacks += 1;
      },
      onDebugEvent: (name, value) => stdout.push(`debug:${name}:${value}`),
      onRecordEvent: (name, value) => stdout.push(`record:${name}:${value}`),
    });

    progress("staging persist tree");
    ensureDir(module.FS, "/models");
    ensureDir(module.FS, "/scripts");
    writeHostTree(module.FS, persist);
    if (args["write-default-model"] === "true" || args["write-default-model"] === "1") {
      progress("writing default settings and model");
      module._writeDefaultSettingsAndModel();
    }
    progress("starting simulator");
    module._start();
    result.started = true;
    progress("waiting after start");
    await sleep(startupMs);
    if (typeof module._reloadScripts === "function") {
      progress("reloading scripts");
      module._reloadScripts();
      result.reloaded = true;
      progress("waiting after reload");
    } else {
      const message = {
        level: "info",
        code: "reloadScripts_unavailable",
        export: "_reloadScripts",
        message: "module._reloadScripts is unavailable; continuing without script reload.",
      };
      result.messages.push(message);
      progress(`${message.code}: ${message.message}`);
      progress("waiting after start settle");
    }
    await sleep(settleMs);
    const errors = findScriptErrors([...stdout, ...stderr]);
    result.status = errors.length ? "script_failure" : "success";
    result.errors = errors;
    result.stdout = stdout.slice(-100);
    result.stderr = stderr.slice(-100);
    result.canvasUpdates = canvasUpdates;
    result.modelJsonCallbacks = modelJsonCallbacks;
  } catch (error) {
    result.status = result.started ? "script_failure" : "startup_failure";
    result.errors = [error && error.stack ? error.stack : String(error)];
    result.stdout = stdout.slice(-100);
    result.stderr = stderr.slice(-100);
    result.canvasUpdates = canvasUpdates;
    result.modelJsonCallbacks = modelJsonCallbacks;
  }

  console.log(JSON.stringify(result));
  process.exit(result.status === "success" ? 0 : result.status === "script_failure" ? 20 : 10);
}

main();
