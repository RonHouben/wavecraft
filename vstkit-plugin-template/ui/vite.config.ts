import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import path from 'node:path';

export default defineConfig({
  plugins: [react()],
  base: './',
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
    '__APP_VERSION__': JSON.stringify(process.env.VITE_APP_VERSION || 'dev'),
  },
  resolve: {
    alias: {
      '@vstkit/ipc': path.resolve(__dirname, './src/lib/vstkit-ipc'),
      '@vstkit/ipc/meters': path.resolve(__dirname, './src/lib/vstkit-ipc/meters'),
    },
  },
});
