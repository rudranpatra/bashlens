#!/usr/bin/env node
// Thin exec wrapper - the actual analysis is the native binary installed by
// ../install.js. This file only forwards argv/stdio and the exit code.

const path = require("path");
const { spawnSync } = require("child_process");

const binPath = path.join(__dirname, "bashlens-native");
const result = spawnSync(binPath, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(`bashlens: failed to run native binary - ${result.error.message}`);
  console.error("Try reinstalling: npm install bashlens");
  process.exit(1);
}
process.exit(result.status ?? 1);
