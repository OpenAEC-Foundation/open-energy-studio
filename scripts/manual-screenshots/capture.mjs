#!/usr/bin/env node
// Captures the full-window screenshots of the manual (shots.mjs) from the real
// app: builds the in-process kernel example, starts a temporary vite server on
// a free loopback port, runs `firefox --headless --screenshot` per shot,
// colour-reduces the result into docs/handleiding-nta8800/img/ and stops the
// server. No API server is started. See README.md.
//
//   node scripts/manual-screenshots/capture.mjs [name ...]
import { execFileSync, spawn } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, statSync } from 'node:fs';
import { createServer as createNetServer } from 'node:net';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import sharp from 'sharp';
import { SHOTS } from './shots.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const outDir = path.join(root, 'docs/handleiding-nta8800/img');
const RESERVED_PORTS = new Set([3917]);

function freePort() {
  return new Promise((resolve, reject) => {
    const probe = createNetServer();
    probe.unref();
    probe.on('error', reject);
    probe.listen(0, '127.0.0.1', () => {
      const { port } = probe.address();
      probe.close(() => (RESERVED_PORTS.has(port) ? freePort().then(resolve, reject) : resolve(port)));
    });
  });
}

function run(command, args, timeoutMs) {
  return new Promise((resolve) => {
    const child = spawn(command, args, { stdio: ['ignore', 'ignore', 'inherit'] });
    const timer = setTimeout(() => child.kill('SIGTERM'), timeoutMs);
    child.on('close', (code) => { clearTimeout(timer); resolve(code); });
    child.on('error', () => { clearTimeout(timer); resolve(-1); });
  });
}

async function main() {
  const wanted = process.argv.slice(2);
  const shots = wanted.length ? SHOTS.filter((shot) => wanted.includes(shot.name)) : SHOTS;
  if (shots.length === 0) throw new Error(`No shot named ${wanted.join(', ')}`);

  console.log('Building the in-process kernel (kernel_once example)…');
  execFileSync('cargo', ['build', '--locked', '--quiet', '--manifest-path', 'crates/nta8800-service/Cargo.toml', '--example', 'kernel_once'],
    { cwd: root, stdio: 'inherit' });

  const port = await freePort();
  const server = await createServer({ configFile: path.join(here, 'vite.config.mjs'), server: { port } });
  await server.listen();
  const base = `http://127.0.0.1:${port}`;
  console.log(`Harness at ${base}`);
  try {
    for (const shot of shots) {
      const profile = mkdtempSync(path.join(process.env.SHOT_TMPDIR || tmpdir(), 'oes-shot-profile-'));
      const raw = path.join(profile, 'shot.png');
      const [pathAndQuery, hash = ''] = shot.url.split('#');
      const url = `${base}${pathAndQuery}${pathAndQuery.includes('?') ? '&' : '?'}shot=${shot.name}${hash ? `#${hash}` : ''}`;
      // Asynchronous: the vite server in this process must keep answering while Firefox loads the page.
      const status = await run('firefox', ['--headless', '--no-remote', '--profile', profile,
        '--window-size', `${shot.width},${shot.height}`, '--screenshot', raw, url], 180000);
      if (status !== 0 || !existsSync(raw)) throw new Error(`firefox failed for ${shot.name}`);
      const target = path.join(outDir, `${shot.name}.png`);
      await sharp(raw).png({ palette: true, colours: 128, compressionLevel: 9 }).toFile(target);
      console.log(`${shot.name}.png ${shot.width}×${shot.height} ${Math.round(statSync(target).size / 1024)} KB`);
      rmSync(profile, { recursive: true, force: true });
    }
  } finally {
    await server.close();
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
