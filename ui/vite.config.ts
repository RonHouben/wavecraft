import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import fs from 'node:fs';

/**
 * Extract version from engine/Cargo.toml
 * In production builds, xtask sets VITE_APP_VERSION env var.
 * In development, we read directly from Cargo.toml.
 */
function getAppVersion(): string {
  if (process.env.VITE_APP_VERSION) {
    return process.env.VITE_APP_VERSION;
  }

  try {
    const cargoTomlPath = path.resolve(__dirname, '../engine/Cargo.toml');
    const cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
    const versionMatch = cargoToml.match(/^\[workspace\.package\]\s*\nversion\s*=\s*"([^"]+)"/m);
    if (versionMatch) {
      return versionMatch[1];
    }
  } catch (error) {
    console.warn('Could not read version from Cargo.toml:', error);
  }

  return 'dev';
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  base: './', // Use relative paths for embedded assets
  resolve: {
    alias: {
      '@wavecraft/ipc': path.resolve(__dirname, './src/lib/wavecraft-ipc'),
      '@wavecraft/ipc/meters': path.resolve(__dirname, './src/lib/wavecraft-ipc/meters'),
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(getAppVersion()),
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
