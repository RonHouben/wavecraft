import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@wavecraft/core': path.resolve(__dirname, './packages/core/src'),
      '@wavecraft/core/meters': path.resolve(__dirname, './packages/core/src/meters'),
      '@wavecraft/components': path.resolve(__dirname, './packages/components/src'),
    },
  },
});
