#!/usr/bin/env node
/**
 * Package proxy-server.bundle.cjs as a Tauri sidecar binary (no system Node required).
 * Output: src-tauri/binaries/proxy-server-{target-triple}[.exe]
 */

import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const binariesDir = path.join(root, 'src-tauri/binaries');
const bundlePath = path.join(root, 'src-tauri/scripts/proxy-server.bundle.cjs');

if (!fs.existsSync(bundlePath)) {
  console.error('Missing proxy bundle. Run: pnpm build:proxy');
  process.exit(1);
}

fs.mkdirSync(binariesDir, { recursive: true });

const ext = process.platform === 'win32' ? '.exe' : '';
const targetTriple = execSync('rustc --print host-tuple', { cwd: root })
  .toString()
  .trim();

if (!targetTriple) {
  console.error('Failed to determine Rust target triple');
  process.exit(1);
}

const outputPath = path.join(binariesDir, `proxy-server-${targetTriple}${ext}`);
const stagingPath = path.join(binariesDir, `_proxy-server-staging${ext}`);

if (fs.existsSync(stagingPath)) fs.unlinkSync(stagingPath);

console.log(`Building sidecar for ${targetTriple}…`);

const pkgCache = path.join(root, '.pkg-cache');
fs.mkdirSync(pkgCache, { recursive: true });

execSync(
  `pnpm exec pkg "${bundlePath}" --output "${stagingPath}" --compress GZip`,
  {
    stdio: 'inherit',
    cwd: root,
    env: { ...process.env, PKG_CACHE_PATH: pkgCache },
  },
);

if (fs.existsSync(outputPath)) fs.unlinkSync(outputPath);
fs.renameSync(stagingPath, outputPath);

// Tauri dev copies sidecar without triple suffix next to the app binary
const devCopy = path.join(binariesDir, `proxy-server${ext}`);
fs.copyFileSync(outputPath, devCopy);

console.log(`Sidecar ready: ${outputPath}`);
