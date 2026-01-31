import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';

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
  define: {
    __APP_VERSION__: JSON.stringify(process.env.VITE_APP_VERSION || 'dev'),
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
