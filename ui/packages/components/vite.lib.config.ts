import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'node:path';
import dts from 'vite-plugin-dts';

export default defineConfig({
  plugins: [
    react(),
    dts({
      include: ['src'],
      exclude: ['src/**/*.test.ts', 'src/**/*.test.tsx', 'src/**/*.typecheck.ts'],
      logLevel: 'silent',
      rollupTypes: true,
    }),
  ],
  build: {
    lib: {
      entry: {
        index: resolve(__dirname, 'src/index.ts'),
        icons: resolve(__dirname, 'src/icons/index.ts'),
      },
      formats: ['es'],
      fileName: (_, entryName) => `${entryName}.js`,
    },
    rollupOptions: {
      external: ['react', 'react-dom', 'react/jsx-runtime', '@wavecraft/core'],
      output: {
        preserveModules: false,
        exports: 'named',
      },
    },
    outDir: 'dist',
    sourcemap: true,
    minify: false,
    emptyOutDir: true,
  },
});
