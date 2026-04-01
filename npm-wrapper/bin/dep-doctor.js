#!/usr/bin/env node
/**
 * Thin shim — just invokes the native binary with all args forwarded.
 */

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const binName = process.platform === "win32" ? "dep-doctor.exe" : "dep-doctor";
const binPath = path.join(__dirname, binName);

if (!fs.existsSync(binPath)) {
  console.error(
    "dep-doctor binary not found. Try reinstalling: npm install -g dep-doctor\n" +
    "Or build from source: cargo install dep-doctor"
  );
  process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), { stdio: "inherit" });
process.exit(result.status ?? 1);
