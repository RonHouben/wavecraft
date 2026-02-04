import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'node:path';

export default defineConfig({
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify('dev'),
  },
  test: {
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}', 'packages/*/src/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      exclude: ['src/test/**', '**/*.test.{ts,tsx}', '**/*.d.ts'],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@wavecraft/core': path.resolve(__dirname, './packages/core/src'),
      '@wavecraft/core/meters': path.resolve(__dirname, './packages/core/src/meters'),
      '@wavecraft/components': path.resolve(__dirname, './packages/components/src'),
      '@test': path.resolve(__dirname, './src/test'),
    },
  },
});
