import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import fs from 'node:fs';

/**
 * Extract version from wavecraft-core Cargo.toml
 * In production builds, xtask sets VITE_APP_VERSION env var.
 * In development, we read directly from wavecraft-core/Cargo.toml.
 */
function getAppVersion(): string {
  if (process.env.VITE_APP_VERSION) {
    return process.env.VITE_APP_VERSION;
  }

  try {
    const cargoTomlPath = path.resolve(__dirname, '../engine/crates/wavecraft-core/Cargo.toml');
    const cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
    const versionMatch = cargoToml.match(/^\[package\]\s*\nname\s*=\s*"wavecraft-core"\s*\nversion\s*=\s*"([^"]+)"/m);
    if (versionMatch) {
      return versionMatch[1];
    }
  } catch (error) {
    console.warn('Could not read version from wavecraft-core Cargo.toml:', error);
  }

  return 'dev';
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  base: './', // Use relative paths for embedded assets
  resolve: {
    alias: {
      '@wavecraft/core': path.resolve(__dirname, './packages/core/src'),
      '@wavecraft/components': path.resolve(__dirname, './packages/components/src'),
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
