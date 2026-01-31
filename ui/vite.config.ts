import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  base: './', // Use relative paths for embedded assets
  resolve: {
    alias: {
      '@vstkit/ipc': path.resolve(__dirname, './src/lib/vstkit-ipc'),
      '@vstkit/ipc/meters': path.resolve(__dirname, './src/lib/vstkit-ipc/meters'),
    },
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    // Generate source maps for debugging
    sourcemap: true,
    // Optimize for smaller bundle size
    minify: 'esbuild',
    target: 'es2020',
  },
});
