import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import siteAppTools from './src/core/site/siteAppTools.json';

export default defineConfig({
  plugins: [
    react(),
    // The definitions of the assistant tools (window.__OPDS_APP__) next to the bundle, for OpenAEC SuperCloud.
    { name: 'mcp-tools', apply: 'build', generateBundle() { this.emitFile({ type: 'asset', fileName: 'mcp-tools.json', source: JSON.stringify(siteAppTools, null, 2) }); } },
  ],
  clearScreen: false,
  server: {
    port: 3006,
    open: false,
    strictPort: false,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3007',
        changeOrigin: false,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
