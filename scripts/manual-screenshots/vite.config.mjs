// Vite config for the manual screenshot harness (see README.md).
// Serves the real app from the repository root. The app's `/api/…` calls are
// answered in-process by the Rust kernel through the `kernel_once` example
// (one process per request, no server, no port), and boot.js is loaded
// before the app to set up each shot.
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
// Where cargo puts the example: CARGO_TARGET_DIR when set (relative to the repository root), else the crate's target.
const targetDir = process.env.CARGO_TARGET_DIR
  ? path.resolve(root, process.env.CARGO_TARGET_DIR) : path.join(root, 'crates/nta8800-service/target');
export const KERNEL_ONCE = path.join(targetDir, 'debug/examples/kernel_once');

function readBody(request) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    request.on('data', (chunk) => chunks.push(chunk));
    request.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    request.on('error', reject);
  });
}

function runKernel(method, apiPath, body) {
  return new Promise((resolve, reject) => {
    const child = spawn(KERNEL_ONCE, [method, apiPath], { stdio: ['pipe', 'pipe', 'inherit'] });
    let out = '';
    child.stdout.on('data', (chunk) => { out += chunk; });
    child.on('error', reject);
    child.on('close', () => {
      try { resolve(JSON.parse(out)); } catch (error) { reject(error); }
    });
    child.stdin.end(body);
  });
}

function shotHarness() {
  let release = null;
  let ready = false;
  return {
    name: 'manual-screenshot-harness',
    transformIndexHtml(html) {
      return html.replace('<script type="module" src="/src/main.tsx">',
        '<script type="module" src="/scripts/manual-screenshots/boot.js"></script>\n    <script type="module" src="/src/main.tsx">');
    },
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        const url = request.url ?? '';
        if (url.startsWith('/__shot/wait')) {
          // Held until the page reports ready (or a minute passes), which delays the load event.
          const answer = () => { response.setHeader('content-type', 'image/gif'); response.end(Buffer.from('R0lGODlhAQABAAAAACw=', 'base64')); };
          if (ready) { ready = false; answer(); return; }
          const timer = setTimeout(answer, 60000);
          release = () => { clearTimeout(timer); answer(); };
          return;
        }
        if (url.startsWith('/__shot/ready')) {
          if (release) { release(); release = null; } else { ready = true; }
          response.end('ok');
          return;
        }
        if (url.startsWith('/api/')) {
          try {
            const body = request.method === 'GET' ? '' : await readBody(request);
            const outcome = await runKernel(request.method ?? 'GET', url.slice('/api'.length), body);
            response.statusCode = outcome.status;
            response.setHeader('content-type', 'application/json');
            response.end(JSON.stringify(outcome.body));
          } catch (error) {
            response.statusCode = 500;
            response.end(String(error));
          }
          return;
        }
        next();
      });
    },
  };
}

export default defineConfig({
  root,
  configFile: false,
  plugins: [react(), shotHarness()],
  clearScreen: false,
  logLevel: 'warn',
  server: { host: '127.0.0.1', strictPort: true, watch: { ignored: ['**/src-tauri/**', '**/target/**', '**/crates/**'] } },
  envPrefix: ['VITE_', 'TAURI_'],
});
