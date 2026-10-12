// Injected before the app by the harness vite config (vite.config.mjs).
// Sets the browser storage of a shot, holds the page's load event until the
// app is idle and the shot's action has run, then tells the harness server
// the page is ready: Firefox's --screenshot is taken at the load event.
import { SHOTS, STORAGE } from './shots.mjs';

const params = new URLSearchParams(window.location.search);
const shot = SHOTS.find((candidate) => candidate.name === params.get('shot'));

for (const [key, value] of Object.entries(STORAGE)) localStorage.setItem(key, value);

// An image added before the load event delays it; the server answers it only after /__shot/ready.
const hold = new Image();
hold.src = `/__shot/wait?t=${Date.now()}`;
hold.style.display = 'none';
document.documentElement.appendChild(hold);

// Track the app's requests to the (in-process) kernel, to know when it is idle.
let pending = 0;
let lastActivity = Date.now();
const realFetch = window.fetch.bind(window);
window.fetch = async (...args) => {
  pending += 1;
  lastActivity = Date.now();
  try {
    return await realFetch(...args);
  } finally {
    pending -= 1;
    lastActivity = Date.now();
  }
};

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function idle(quietMs = 1200, timeoutMs = 60000) {
  const start = Date.now();
  await sleep(400);
  while (Date.now() - start < timeoutMs) {
    if (pending === 0 && Date.now() - lastActivity >= quietMs) return;
    await sleep(100);
  }
}

async function waitFor(test, timeoutMs = 60000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (test()) return true;
    await sleep(100);
  }
  return false;
}

function typeInto(input, text) {
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
  setter.call(input, text);
  input.dispatchEvent(new Event('input', { bubbles: true }));
}

const ACTIONS = {
  async closeAllTabs() {
    for (let i = 0; i < 10; i += 1) {
      const close = document.querySelector('.document-tab-close');
      if (!close) return;
      close.click();
      await sleep(300);
    }
  },
  async openPalette(definition) {
    document.querySelector('.top-bar-search')?.click();
    await sleep(300);
    const input = document.querySelector('[role="dialog"] input');
    if (input && definition.query) typeInto(input, definition.query);
  },
};

const wantedHash = window.location.hash;

async function run() {
  // The first load compiles the app in the dev server; wait for the shell before judging idleness.
  await waitFor(() => document.querySelector('.top-bar'));
  await idle();
  // An example opened from `?example=` starts on its own route, so navigate once it is open.
  if (wantedHash && window.location.hash !== wantedHash) {
    window.location.hash = wantedHash;
    await idle(800);
  }
  if (shot?.action) {
    await ACTIONS[shot.action](shot);
    await idle(800);
  }
  // Let fonts and transitions settle.
  await document.fonts?.ready;
  await sleep(600);
  await realFetch('/__shot/ready', { method: 'POST' });
}

void run();
