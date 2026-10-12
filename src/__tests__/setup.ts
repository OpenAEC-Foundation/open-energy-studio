import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { configure } from '@testing-library/react';
import { UI_WAIT_MS } from './timeouts';

configure({ asyncUtilTimeout: UI_WAIT_MS });

// Mock Tauri APIs that components may import
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => false),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
    startDragging: vi.fn(),
    isMaximized: vi.fn().mockResolvedValue(false),
    onResized: vi.fn().mockResolvedValue(vi.fn()),
    onCloseRequested: vi.fn().mockResolvedValue(vi.fn()),
    destroy: vi.fn(),
  }),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
  ask: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
}));
