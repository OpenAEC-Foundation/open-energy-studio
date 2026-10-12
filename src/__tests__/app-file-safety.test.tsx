// The desktop save path in the real app: a write that fails (disk full,
// read-only folder) is shown as an error and is never reported as a save
// through the browser-download fallback.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import App from '../App';
import { SLOW_UI_TEST_MS } from './timeouts';

const windowMock = vi.hoisted(() => ({
  handler: null as null | ((event: { preventDefault: () => void }) => Promise<void> | void),
  destroy: null as null | ReturnType<typeof import('vitest').vi.fn>,
}));

vi.mock('@tauri-apps/api/window', async () => {
  const { vi: v } = await import('vitest');
  windowMock.destroy = v.fn();
  return {
    getCurrentWindow: () => ({
      minimize: v.fn(),
      toggleMaximize: v.fn(),
      close: v.fn(),
      startDragging: v.fn(),
      isMaximized: v.fn().mockResolvedValue(false),
      onResized: v.fn().mockResolvedValue(v.fn()),
      onCloseRequested: v.fn(async (handler: NonNullable<typeof windowMock.handler>) => {
        windowMock.handler = handler;
        return v.fn();
      }),
      destroy: windowMock.destroy,
    }),
  };
});

describe('saving in the desktop app', () => {
  let createObjectURL: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    localStorage.clear();
    vi.mocked(isTauri).mockReturnValue(true);
    createObjectURL = vi.fn(() => 'blob:x');
    Object.assign(URL, { createObjectURL, revokeObjectURL: vi.fn() });
  });

  afterEach(() => {
    vi.mocked(isTauri).mockReturnValue(false);
    vi.mocked(invoke).mockReset();
    vi.mocked(save).mockReset();
  });

  it('shows a failed write and offers no download', async () => {
    vi.mocked(save).mockResolvedValue('/alleen-lezen/woning.oes.json');
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'save_project') throw 'project_save_failed: Read-only file system (os error 30)';
      throw new Error(`no kernel in this test: ${command}`);
    });
    render(<App />);
    fireEvent.keyDown(window, { key: 'n', ctrlKey: true });
    fireEvent.keyDown(window, { key: 's', ctrlKey: true });

    await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalledWith('save_project', expect.objectContaining({
      path: '/alleen-lezen/woning.oes.json',
    })), { timeout: 10_000 });
    expect(await screen.findByText(/Read-only file system/, undefined, { timeout: 10_000 })).toBeInTheDocument();
    expect(createObjectURL).not.toHaveBeenCalled();
  }, SLOW_UI_TEST_MS);

  it('listens for the window closing and lets a clean window close', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('no kernel in this test'));
    render(<App />);
    fireEvent.keyDown(window, { key: 'n', ctrlKey: true });
    await waitFor(() => expect(windowMock.handler).not.toBeNull());
    const preventDefault = vi.fn();
    await windowMock.handler!({ preventDefault });
    // Nothing unsaved: the close goes ahead without a question.
    expect(preventDefault).not.toHaveBeenCalled();
    expect(windowMock.destroy).not.toHaveBeenCalled();
  }, SLOW_UI_TEST_MS);
});
