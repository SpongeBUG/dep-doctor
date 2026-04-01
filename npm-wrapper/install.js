#!/usr/bin/env node
/**
 * postinstall script — downloads the correct dep-doctor binary
 * for the current platform from GitHub Releases.
 *
 * Pattern: same as @biomejs/biome, @tailwindcss/standalone, prisma, etc.
 */

const { execSync } = require("child_process");
const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const { version } = require("./package.json");

const REPO = "YOUR_USERNAME/dep-doctor";
const BIN_DIR = path.join(__dirname, "bin");
const BIN_PATH = path.join(BIN_DIR, process.platform === "win32" ? "dep-doctor.exe" : "dep-doctor");

// Platform → GitHub release asset name
function getAssetName() {
  const platform = process.platform;
  const arch = process.arch;

  const matrix = {
    "linux-x64":   `dep-doctor-x86_64-unknown-linux-musl`,
    "linux-arm64": `dep-doctor-aarch64-unknown-linux-musl`,
    "darwin-x64":  `dep-doctor-x86_64-apple-darwin`,
    "darwin-arm64":`dep-doctor-aarch64-apple-darwin`,
    "win32-x64":   `dep-doctor-x86_64-pc-windows-msvc.exe`,
  };

  const key = `${platform}-${arch}`;
  const asset = matrix[key];
  if (!asset) {
    throw new Error(`Unsupported platform: ${key}. Build from source: cargo install dep-doctor`);
  }
  return asset;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    function get(u) {
      https.get(u, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          return get(res.headers.location);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`HTTP ${res.statusCode} for ${u}`));
        }
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
      }).on("error", reject);
    }
    get(url);
  });
}

async function main() {
  if (fs.existsSync(BIN_PATH)) {
    return; // already installed (e.g. cached by npm)
  }

  const asset = getAssetName();
  const url = `https://github.com/${REPO}/releases/download/v${version}/${asset}`;

  console.log(`dep-doctor: downloading binary for ${process.platform}/${process.arch}...`);
  fs.mkdirSync(BIN_DIR, { recursive: true });

  await download(url, BIN_PATH);

  if (process.platform !== "win32") {
    fs.chmodSync(BIN_PATH, 0o755);
  }

  console.log("dep-doctor: installed successfully.");
}

main().catch((err) => {
  console.error("dep-doctor install failed:", err.message);
  console.error("You can install manually via: cargo install dep-doctor");
  process.exit(0); // Don't fail npm install — degrade gracefully
});
