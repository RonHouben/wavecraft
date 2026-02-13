import react from '@vitejs/plugin-react';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const isSDKMode = fs.existsSync(path.resolve(currentDir, '../../engine/crates/wavecraft-core'));

export default defineConfig({
  plugins: [react()],
  base: './',
  resolve: {
    alias: isSDKMode
      ? {
          '@wavecraft/core': path.resolve(currentDir, '../../ui/packages/core/src'),
          '@wavecraft/components': path.resolve(currentDir, '../../ui/packages/components/src'),
        }
      : {},
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks: undefined,
      },
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(process.env.VITE_APP_VERSION || 'dev'),
  },
});
