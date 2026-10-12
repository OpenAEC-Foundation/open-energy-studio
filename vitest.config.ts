import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { TEST_TIMEOUT_MS } from './src/__tests__/timeouts';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/__tests__/setup.ts'],
    testTimeout: TEST_TIMEOUT_MS,
    css: true,
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
