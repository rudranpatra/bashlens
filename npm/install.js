#!/usr/bin/env node
// Downloads the prebuilt bashlens binary for this platform from the GitHub
// release matching this package's version, and extracts it into bin/.
// No npm dependencies on purpose - a security tool's own install script is
// exactly the kind of thing this project exists to scrutinize, so it keeps
// to Node's standard library plus the system `tar` (present by default on
// every platform this package supports; there is no Windows build).

const fs = require("fs");
const https = require("https");
const path = require("path");
const { execFileSync } = require("child_process");

const pkg = require("./package.json");

const TARGETS = {
  "linux-x64": "x86_64-unknown-linux-musl",
  "linux-arm64": "aarch64-unknown-linux-musl",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
};

function targetTriple() {
  const key = `${process.platform}-${process.arch}`;
  const target = TARGETS[key];
  if (!target) {
    throw new Error(
      `bashlens has no prebuilt binary for ${key}. ` +
        `Supported: ${Object.keys(TARGETS).join(", ")}. ` +
        `Build from source instead: https://github.com/rudranpatra/bashlens`
    );
  }
  return target;
}

function download(url, destPath, redirectsLeft = 5) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "bashlens-npm-installer" } }, (res) => {
        if ([301, 302, 307, 308].includes(res.statusCode) && res.headers.location) {
          if (redirectsLeft <= 0) return reject(new Error("too many redirects"));
          res.resume();
          return resolve(download(res.headers.location, destPath, redirectsLeft - 1));
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`download failed: HTTP ${res.statusCode} for ${url}`));
        }
        const file = fs.createWriteStream(destPath);
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
        file.on("error", reject);
      })
      .on("error", reject);
  });
}

async function main() {
  const target = targetTriple();
  const binDir = path.join(__dirname, "bin");
  const tarPath = path.join(binDir, `bashlens-${target}.tar.gz`);
  const binPath = path.join(binDir, "bashlens-native");
  const url = `https://github.com/rudranpatra/bashlens/releases/download/v${pkg.version}/bashlens-${target}.tar.gz`;

  fs.mkdirSync(binDir, { recursive: true });
  console.log(`bashlens: downloading ${target} binary from ${url}`);
  await download(url, tarPath);

  execFileSync("tar", ["xzf", tarPath, "-C", binDir]);
  fs.renameSync(path.join(binDir, "bashlens"), binPath);
  fs.chmodSync(binPath, 0o755);
  fs.unlinkSync(tarPath);
  console.log("bashlens: installed");
}

main().catch((err) => {
  console.error(`bashlens: install failed - ${err.message}`);
  process.exit(1);
});
