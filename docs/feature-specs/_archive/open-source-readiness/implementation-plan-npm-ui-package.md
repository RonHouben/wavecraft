# Implementation Plan: npm UI Package Publishing

**Feature:** User Story 9 — UI Package Publishing  
**Low-Level Design:** [low-level-design-npm-ui-package.md](low-level-design-npm-ui-package.md)  
**Author:** Planner Agent  
**Date:** 2026-02-04  
**Estimated Effort:** 5.5 days

---

## Overview

Transform the monorepo UI code into two publishable npm packages and update the template to consume them as dependencies:

- **`@wavecraft/core`** — IPC bridge, hooks, types, utilities (SDK foundation)
- **`@wavecraft/components`** — Pre-built React components (optional convenience)

This split architecture enables:
- Future premium packages (`@wavecraft/pro`) under the same namespace
- Users who want minimal SDK without pre-built components
- Clean licensing separation (all MIT for now, commercial for future premium)

---

## Phase 1: Workspace Setup

**Goal:** Set up npm workspaces monorepo structure for multiple packages.

### Step 1.1: Create Packages Directory Structure

**Action:** Create the workspace directory structure  
**Why:** Organize packages for independent publishing  
**Dependencies:** None  
**Risk:** Low

```bash
mkdir -p ui/packages/core/src
mkdir -p ui/packages/components/src
```

**Target structure:**
```
ui/
├── package.json              # Root workspace config
├── packages/
│   ├── core/                 # @wavecraft/core
│   │   ├── package.json
│   │   └── src/
│   └── components/           # @wavecraft/components
│       ├── package.json
│       └── src/
├── src/                      # Dev app (unchanged)
├── vite.config.ts            # Dev server (unchanged)
└── vitest.config.ts          # Tests (unchanged)
```

---

### Step 1.2: Update Root package.json for Workspaces

**File:** `ui/package.json`  
**Action:** Add workspaces configuration  
**Why:** Enable npm workspace features for managing multiple packages  
**Dependencies:** Step 1.1  
**Risk:** Low

Add/update these fields:
```json
{
  "name": "wavecraft-ui",
  "private": true,
  "workspaces": [
    "packages/*"
  ],
  "scripts": {
    "dev": "vite",
    "build": "npm run build --workspaces",
    "build:lib": "npm run build:lib --workspaces",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "eslint . && prettier --check .",
    "lint:fix": "eslint . --fix && prettier --write ."
  }
}
```

---

### Step 1.3: Move IPC Code to Core Package

**Action:** Move IPC library files to `packages/core/src/`  
**Why:** Core package contains SDK foundation  
**Dependencies:** Steps 1.1, 1.2  
**Risk:** Medium — Must update all import paths

**Move these files:**
```
ui/src/lib/wavecraft-ipc/ → ui/packages/core/src/
```

Reorganize into cleaner structure:
```
packages/core/src/
├── index.ts              # Main entry
├── meters.ts             # Subpath entry (pure utilities)
├── bridge/
│   ├── IpcBridge.ts
│   ├── NativeTransport.ts
│   └── WebSocketTransport.ts
├── client/
│   └── ParameterClient.ts
├── hooks/
│   ├── useParameter.ts
│   ├── useAllParameters.ts
│   ├── useParameterGroups.ts
│   ├── useMeterFrame.ts
│   ├── useConnectionStatus.ts
│   ├── useRequestResize.ts
│   └── useLatencyMonitor.ts
├── types/
│   └── index.ts
└── utils/
    ├── logger.ts
    ├── environment.ts
    └── audio-math.ts
```

---

### Step 1.4: Move Component Code to Components Package

**Action:** Move component files to `packages/components/src/`  
**Why:** Components package contains pre-built UI widgets  
**Dependencies:** Steps 1.1, 1.2  
**Risk:** Medium — Must update all import paths

**Move these files:**
```
ui/src/components/*.tsx → ui/packages/components/src/
```

```
packages/components/src/
├── index.ts              # Component exports
├── Meter.tsx
├── ParameterSlider.tsx
├── ParameterGroup.tsx
├── ParameterToggle.tsx
├── VersionBadge.tsx
├── ConnectionStatus.tsx
├── LatencyMonitor.tsx
├── ResizeHandle.tsx
└── ResizeControls.tsx
```

---

### Step 1.5: Update Dev App Imports

**File:** `ui/src/App.tsx`, `ui/src/main.tsx`  
**Action:** Update imports to use workspace packages  
**Why:** Dev app should work with new structure  
**Dependencies:** Steps 1.3, 1.4  
**Risk:** Low

**Before:**
```typescript
import { useParameter } from './lib/wavecraft-ipc';
import { Meter } from './components/Meter';
```

**After:**
```typescript
import { useParameter } from '@wavecraft/core';
import { Meter } from '@wavecraft/components';
```

---

### Step 1.6: Update Vite Config for Workspace Aliases

**File:** `ui/vite.config.ts`  
**Action:** Add resolve aliases for local development  
**Why:** Allow importing workspace packages during development  
**Dependencies:** Step 1.5  
**Risk:** Low

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@wavecraft/core': resolve(__dirname, 'packages/core/src'),
      '@wavecraft/components': resolve(__dirname, 'packages/components/src'),
    },
  },
  // ... rest of config
});
```

---

## Phase 2: Core Package Infrastructure

**Goal:** Set up build configuration for `@wavecraft/core`.

### Step 2.1: Install Build Dependencies

**Action:** Add `vite-plugin-dts` to workspace  
**Why:** Required for TypeScript declaration generation  
**Dependencies:** Phase 1 complete  
**Risk:** Low

```bash
cd ui && npm install -D vite-plugin-dts
```

---

### Step 2.2: Create Core package.json

**File:** `ui/packages/core/package.json` (NEW)  
**Action:** Create package configuration  
**Why:** Required for npm publishing  
**Dependencies:** Step 2.1  
**Risk:** Low

```json
{
  "name": "@wavecraft/core",
  "version": "0.7.0",
  "description": "Core SDK for Wavecraft audio plugins — IPC bridge, hooks, and utilities",
  "type": "module",
  "main": "./dist/index.js",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    },
    "./meters": {
      "import": "./dist/meters.js",
      "types": "./dist/meters.d.ts"
    }
  },
  "files": [
    "dist",
    "README.md"
  ],
  "sideEffects": false,
  "keywords": [
    "wavecraft",
    "vst",
    "clap",
    "audio",
    "plugin",
    "react",
    "ipc",
    "sdk"
  ],
  "author": "Ron Houben",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/RonHouben/wavecraft.git",
    "directory": "ui/packages/core"
  },
  "homepage": "https://github.com/RonHouben/wavecraft",
  "bugs": {
    "url": "https://github.com/RonHouben/wavecraft/issues"
  },
  "peerDependencies": {
    "react": "^18.0.0"
  },
  "scripts": {
    "build:lib": "vite build --config vite.lib.config.ts",
    "prepublishOnly": "npm run build:lib"
  }
}
```

---

### Step 2.3: Create Core Vite Build Config

**File:** `ui/packages/core/vite.lib.config.ts` (NEW)  
**Action:** Create library build configuration  
**Why:** Builds ESM bundle with TypeScript declarations  
**Dependencies:** Step 2.2  
**Risk:** Low

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';

export default defineConfig({
  plugins: [
    react(),
    dts({
      include: ['src'],
      exclude: ['src/**/*.test.ts', 'src/**/*.test.tsx'],
      rollupTypes: true,
    }),
  ],
  build: {
    lib: {
      entry: {
        index: resolve(__dirname, 'src/index.ts'),
        meters: resolve(__dirname, 'src/meters.ts'),
      },
      formats: ['es'],
      fileName: (format, entryName) => `${entryName}.js`,
    },
    rollupOptions: {
      external: ['react', 'react/jsx-runtime'],
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
```

---

### Step 2.4: Create Core TypeScript Build Config

**File:** `ui/packages/core/tsconfig.json` (NEW)  
**Action:** Create TypeScript configuration  
**Why:** Required for type checking and declarations  
**Dependencies:** None  
**Risk:** Low

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "declaration": true,
    "declarationDir": "./dist"
  },
  "include": ["src"],
  "exclude": ["src/**/*.test.ts", "src/**/*.test.tsx"]
}
```

---

### Step 2.5: Create Core Entry Point

**File:** `ui/packages/core/src/index.ts` (NEW)  
**Action:** Create main entry point with all exports  
**Why:** Single import path for consumers  
**Dependencies:** Step 1.3  
**Risk:** Low

```typescript
/**
 * @wavecraft/core - Core SDK for Wavecraft audio plugins
 *
 * @packageDocumentation
 */

// Hooks (Primary API)
export { useParameter, type UseParameterResult } from './hooks/useParameter';
export { useAllParameters, type UseAllParametersResult } from './hooks/useAllParameters';
export { useParameterGroups } from './hooks/useParameterGroups';
export { useMeterFrame } from './hooks/useMeterFrame';
export { useConnectionStatus } from './hooks/useConnectionStatus';
export { useRequestResize } from './hooks/useRequestResize';
export { useLatencyMonitor, type UseLatencyMonitorResult } from './hooks/useLatencyMonitor';

// Types
export type {
  ParameterInfo,
  ParameterGroup,
  MeterFrame,
  ConnectionStatus,
} from './types';

// IPC utilities (Advanced)
export { IpcBridge } from './bridge/IpcBridge';
export { ParameterClient } from './client/ParameterClient';
export { NativeTransport } from './bridge/NativeTransport';
export { WebSocketTransport } from './bridge/WebSocketTransport';

// Logging
export { logger, Logger, LogLevel } from './utils/logger';
```

---

### Step 2.6: Create Core Meters Subpath Entry

**File:** `ui/packages/core/src/meters.ts` (NEW)  
**Action:** Create subpath entry for pure utilities  
**Why:** Allows import without IPC side effects  
**Dependencies:** Step 1.3  
**Risk:** Low

```typescript
/**
 * @wavecraft/core/meters - Pure audio math utilities
 *
 * These utilities have no IPC side effects and can be used
 * in tests or standalone applications.
 *
 * @packageDocumentation
 */

export { getMeterFrame, linearToDb, dbToLinear } from './utils/audio-math';
export type { MeterFrame } from './types';
```

---

## Phase 3: Components Package Infrastructure

**Goal:** Set up build configuration for `@wavecraft/components`.

### Step 3.1: Create Components package.json

**File:** `ui/packages/components/package.json` (NEW)  
**Action:** Create package configuration  
**Why:** Required for npm publishing  
**Dependencies:** Phase 2 complete  
**Risk:** Low

```json
{
  "name": "@wavecraft/components",
  "version": "0.7.0",
  "description": "Pre-built React components for Wavecraft audio plugins",
  "type": "module",
  "main": "./dist/index.js",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "files": [
    "dist",
    "README.md"
  ],
  "sideEffects": false,
  "keywords": [
    "wavecraft",
    "vst",
    "audio",
    "plugin",
    "react",
    "components",
    "meter",
    "slider",
    "ui"
  ],
  "author": "Ron Houben",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/RonHouben/wavecraft.git",
    "directory": "ui/packages/components"
  },
  "homepage": "https://github.com/RonHouben/wavecraft",
  "bugs": {
    "url": "https://github.com/RonHouben/wavecraft/issues"
  },
  "peerDependencies": {
    "@wavecraft/core": "^0.7.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0"
  },
  "peerDependenciesMeta": {
    "react-dom": {
      "optional": true
    }
  },
  "scripts": {
    "build:lib": "vite build --config vite.lib.config.ts",
    "prepublishOnly": "npm run build:lib"
  }
}
```

---

### Step 3.2: Create Components Vite Build Config

**File:** `ui/packages/components/vite.lib.config.ts` (NEW)  
**Action:** Create library build configuration  
**Why:** Builds ESM bundle with TypeScript declarations  
**Dependencies:** Step 3.1  
**Risk:** Low

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';

export default defineConfig({
  plugins: [
    react(),
    dts({
      include: ['src'],
      exclude: ['src/**/*.test.ts', 'src/**/*.test.tsx'],
      rollupTypes: true,
    }),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      formats: ['es'],
      fileName: () => 'index.js',
    },
    rollupOptions: {
      external: [
        'react',
        'react-dom',
        'react/jsx-runtime',
        '@wavecraft/core',
      ],
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
```

---

### Step 3.3: Create Components TypeScript Config

**File:** `ui/packages/components/tsconfig.json` (NEW)  
**Action:** Create TypeScript configuration  
**Why:** Required for type checking and declarations  
**Dependencies:** None  
**Risk:** Low

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "declaration": true,
    "declarationDir": "./dist"
  },
  "include": ["src"],
  "exclude": ["src/**/*.test.ts", "src/**/*.test.tsx"]
}
```

---

### Step 3.4: Create Components Entry Point

**File:** `ui/packages/components/src/index.ts` (NEW)  
**Action:** Create main entry point with all component exports  
**Why:** Single import path for consumers  
**Dependencies:** Step 1.4  
**Risk:** Low

```typescript
/**
 * @wavecraft/components - Pre-built React components for Wavecraft audio plugins
 *
 * @packageDocumentation
 */

// Core plugin UI components
export { Meter, type MeterProps } from './Meter';
export { ParameterSlider, type ParameterSliderProps } from './ParameterSlider';
export { ParameterGroup, type ParameterGroupProps } from './ParameterGroup';
export { ParameterToggle, type ParameterToggleProps } from './ParameterToggle';
export { VersionBadge, type VersionBadgeProps } from './VersionBadge';

// Connection and status components
export { ConnectionStatus } from './ConnectionStatus';
export { LatencyMonitor } from './LatencyMonitor';

// Resize components
export { ResizeHandle } from './ResizeHandle';
export { ResizeControls, type ResizeControlsProps } from './ResizeControls';
```

---

### Step 3.5: Update Component Imports

**Files:** All files in `ui/packages/components/src/`  
**Action:** Update imports to use `@wavecraft/core`  
**Why:** Components depend on core for hooks and types  
**Dependencies:** Step 3.4  
**Risk:** Medium — Must update all internal imports

**Before:**
```typescript
import { useParameter, useMeterFrame } from '../lib/wavecraft-ipc';
```

**After:**
```typescript
import { useParameter, useMeterFrame } from '@wavecraft/core';
```

---

## Phase 4: Build Verification

**Goal:** Validate both packages build correctly.

### Step 4.1: Build Core Package

**Action:** Execute build and verify output  
**Why:** Catch build errors early  
**Dependencies:** Phase 2 complete  
**Risk:** Medium

```bash
cd ui/packages/core
npm run build:lib
```

**Expected output in `dist/`:**
- `index.js` — Main ESM bundle
- `index.d.ts` — TypeScript declarations
- `meters.js` — Meters subpath bundle  
- `meters.d.ts` — Meters type declarations

---

### Step 4.2: Build Components Package

**Action:** Execute build and verify output  
**Why:** Catch build errors early  
**Dependencies:** Step 4.1 (core must build first for peer dep resolution)  
**Risk:** Medium

```bash
cd ui/packages/components
npm run build:lib
```

**Expected output in `dist/`:**
- `index.js` — Main ESM bundle
- `index.d.ts` — TypeScript declarations

---

### Step 4.3: Verify Package Contents

**Action:** Check what would be published for both packages  
**Why:** Ensure only intended files are included  
**Dependencies:** Steps 4.1, 4.2  
**Risk:** Low

```bash
cd ui/packages/core
npm pack --dry-run

cd ../components
npm pack --dry-run
```

**Expected files (each package):**
- `package.json`
- `README.md`
- `dist/*.js`
- `dist/*.d.ts`

---

### Step 4.4: Local Install Test

**Action:** Install both packages locally in a temp project  
**Why:** Verify packages install and imports work  
**Dependencies:** Steps 4.1, 4.2  
**Risk:** Medium

```bash
# Create test consumer
cd /tmp
mkdir wavecraft-pkg-test && cd wavecraft-pkg-test
npm init -y

# Install both packages
npm install /path/to/wavecraft/ui/packages/core
npm install /path/to/wavecraft/ui/packages/components

# Test imports
cat > test.mjs << 'EOF'
import { useParameter, useMeterFrame, logger } from '@wavecraft/core';
import { linearToDb } from '@wavecraft/core/meters';
import { Meter, ParameterSlider } from '@wavecraft/components';

console.log('Core exports:', {
  useParameter: typeof useParameter,
  useMeterFrame: typeof useMeterFrame,
  logger: typeof logger,
  linearToDb: typeof linearToDb,
});

console.log('Component exports:', {
  Meter: typeof Meter,
  ParameterSlider: typeof ParameterSlider,
});

console.log('✅ All imports successful!');
EOF

node test.mjs
```

---

### Step 4.5: TypeScript Compilation Test

**Action:** Verify TypeScript types work in consumer  
**Why:** Ensure DTS files are correct  
**Dependencies:** Step 4.4  
**Risk:** Medium

```bash
cd /tmp/wavecraft-pkg-test

# Add TypeScript
npm install -D typescript @types/react

# Create tsconfig
cat > tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "bundler",
    "target": "ES2020",
    "strict": true,
    "noEmit": true,
    "jsx": "react-jsx"
  }
}
EOF

# Create type test file
cat > test.ts << 'EOF'
import {
  useParameter,
  useMeterFrame,
  logger,
  ParameterInfo,
  MeterFrame,
} from '@wavecraft/core';
import { linearToDb, dbToLinear } from '@wavecraft/core/meters';
import { Meter, ParameterSlider, MeterProps } from '@wavecraft/components';

// Type tests
const param: ParameterInfo | undefined = undefined;
const db: number = linearToDb(0.5);
const linear: number = dbToLinear(-6);

console.log('Types compile correctly!');
EOF

npx tsc --noEmit
```

---

## Phase 5: npm Organization Setup

**Goal:** Register npm organization and prepare for publishing.

### Step 5.1: Create npm Account (if needed)

**Action:** Register or login to npm  
**Why:** Required for publishing  
**Dependencies:** None  
**Risk:** Low

```bash
npm login
```

---

### Step 5.2: Create @wavecraft Organization

**Action:** Register npm organization  
**Why:** Required for scoped packages  
**Dependencies:** Step 5.1  
**Risk:** Low — Fallback to unscoped names if taken

**Option A (Web):**
1. Go to https://www.npmjs.com/org/create
2. Organization name: `wavecraft`
3. Choose "Unlimited public packages" (free)

**Option B (CLI):**
```bash
npm org create wavecraft
```

**Fallback:** If `wavecraft` is taken:
- `wavecraft-core` (unscoped)
- `wavecraft-components` (unscoped)

---

### Step 5.3: Test Publish (Dry Run)

**Action:** Verify publish would succeed for both packages  
**Why:** Catch issues before actual publish  
**Dependencies:** Steps 5.1, 5.2, Phase 4  
**Risk:** Low

```bash
cd ui/packages/core
npm publish --access public --dry-run

cd ../components
npm publish --access public --dry-run
```

---

## Phase 6: Template Migration

**Goal:** Update CLI template to use npm packages instead of copied source.

### Step 6.1: Update Template package.json

**File:** `cli/template/ui/package.json`  
**Action:** Add both @wavecraft packages as dependencies  
**Why:** Template should consume published packages  
**Dependencies:** Phase 5  
**Risk:** Medium

```json
{
  "name": "{{plugin_name}}-ui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "typecheck": "tsc --noEmit",
    "lint": "eslint . --max-warnings 0",
    "lint:fix": "eslint . --fix",
    "format": "prettier --write \"src/**/*.{ts,tsx,css}\"",
    "format:check": "prettier --check \"src/**/*.{ts,tsx,css}\""
  },
  "dependencies": {
    "@wavecraft/core": "^0.7.0",
    "@wavecraft/components": "^0.7.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@eslint/js": "^9.17.0",
    "@types/react": "^18.3.18",
    "@types/react-dom": "^18.3.5",
    "@vitejs/plugin-react": "^4.3.4",
    "autoprefixer": "^10.4.20",
    "eslint": "^9.17.0",
    "eslint-plugin-react-hooks": "^5.1.0",
    "eslint-plugin-react-refresh": "^0.4.16",
    "globals": "^15.14.0",
    "postcss": "^8.4.49",
    "prettier": "^3.4.2",
    "prettier-plugin-tailwindcss": "^0.6.9",
    "tailwindcss": "^3.4.17",
    "typescript": "^5.6.3",
    "typescript-eslint": "^8.18.1",
    "vite": "^6.0.5"
  }
}
```

---

### Step 6.2: Update Template App.tsx

**File:** `cli/template/ui/src/App.tsx`  
**Action:** Import from @wavecraft packages with clear separation  
**Why:** Use npm package exports  
**Dependencies:** Step 6.1  
**Risk:** Low

```tsx
// Core SDK - hooks and utilities
import {
  useAllParameters,
  useParameterGroups,
  useConnectionStatus,
} from '@wavecraft/core';

// Pre-built components
import {
  Meter,
  ParameterSlider,
  ParameterGroup,
  VersionBadge,
  ConnectionStatus,
} from '@wavecraft/components';

function App() {
  const { params, loading, error } = useAllParameters();
  const groups = useParameterGroups();
  const status = useConnectionStatus();

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-plugin-dark">
        <p className="text-gray-400">Loading parameters...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-screen items-center justify-center bg-plugin-dark">
        <p className="text-red-400">Error: {error}</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-plugin-dark p-4">
      <header className="mb-4 flex items-center justify-between">
        <h1 className="text-xl font-bold text-gray-200">{{plugin_name_title}}</h1>
        <div className="flex items-center gap-2">
          <ConnectionStatus />
          <VersionBadge />
        </div>
      </header>

      <Meter />

      <div className="mt-4 space-y-4">
        {groups.length > 0 ? (
          groups.map((group) => (
            <ParameterGroup key={group.name} name={group.name}>
              {group.parameters.map((p) => (
                <ParameterSlider key={p.id} id={p.id} />
              ))}
            </ParameterGroup>
          ))
        ) : (
          params?.map((p) => <ParameterSlider key={p.id} id={p.id} />)
        )}
      </div>
    </div>
  );
}

export default App;
```

---

### Step 6.3: Update Template tailwind.config.js

**File:** `cli/template/ui/tailwind.config.js`  
**Action:** Add @wavecraft/components to content paths  
**Why:** TailwindCSS must scan npm package for used classes  
**Dependencies:** None  
**Risk:** Low

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
    './node_modules/@wavecraft/components/dist/**/*.js',
  ],
  theme: {
    extend: {
      colors: {
        'plugin-dark': '#1a1a1a',
        'plugin-surface': '#2a2a2a',
        'plugin-border': '#444444',
        accent: '#4a9eff',
        'accent-light': '#6bb0ff',
        'meter-safe': '#4caf50',
        'meter-warning': '#ffeb3b',
        'meter-clip': '#ff1744',
      },
    },
  },
  plugins: [],
};
```

---

### Step 6.4: Update Template vite.config.ts

**File:** `cli/template/ui/vite.config.ts`  
**Action:** Remove local path aliases (no longer needed)  
**Why:** Imports come from npm packages  
**Dependencies:** Step 6.1  
**Risk:** Low

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: './',
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: true,
    minify: 'esbuild',
    target: 'es2020',
  },
});
```

---

### Step 6.5: Update Template tsconfig.json

**File:** `cli/template/ui/tsconfig.json`  
**Action:** Remove local path aliases  
**Why:** No longer using internal module paths  
**Dependencies:** Step 6.1  
**Risk:** Low

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true
  },
  "include": ["src"]
}
```

---

### Step 6.6: Remove Copied Source Files from Template

**Action:** Delete copied library and component source files  
**Why:** Now consumed from npm packages  
**Dependencies:** Steps 6.1-6.5  
**Risk:** Medium — Must verify template still builds

**Delete these directories/files:**
- `cli/template/ui/src/lib/wavecraft-ipc/` (entire directory)
- `cli/template/ui/src/components/Meter.tsx`
- `cli/template/ui/src/components/ParameterSlider.tsx`
- `cli/template/ui/src/components/VersionBadge.tsx`
- `cli/template/ui/src/components/LatencyMonitor.tsx`
- `cli/template/ui/src/components/ConnectionStatus.tsx`
- `cli/template/ui/src/components/ParameterGroup.tsx`
- `cli/template/ui/src/components/ParameterToggle.tsx`
- `cli/template/ui/src/components/ResizeHandle.tsx`
- `cli/template/ui/src/components/ResizeControls.tsx`

**Keep:**
- `cli/template/ui/src/App.tsx` (updated in Step 6.2)
- `cli/template/ui/src/main.tsx`
- `cli/template/ui/src/index.css`
- `cli/template/ui/src/vite-env.d.ts`

---

## Phase 7: Package Documentation

**Goal:** Create documentation for npm package consumers.

### Step 7.1: Create Core Package README

**File:** `ui/packages/core/README.md` (NEW)  
**Action:** Write comprehensive README  
**Why:** Displayed on npm registry page  
**Dependencies:** None  
**Risk:** Low

```markdown
# @wavecraft/core

Core SDK for Wavecraft audio plugins — IPC bridge, hooks, and utilities.

## Installation

\`\`\`bash
npm install @wavecraft/core
\`\`\`

## Quick Start

\`\`\`tsx
import { useParameter, useMeterFrame, logger } from '@wavecraft/core';

function MyComponent() {
  const { param, setValue } = useParameter('gain');
  const meterFrame = useMeterFrame();
  
  return (
    <input
      type="range"
      value={param?.value ?? 0}
      onChange={(e) => setValue(parseFloat(e.target.value))}
    />
  );
}
\`\`\`

## API Reference

### Hooks

| Hook | Description |
|------|-------------|
| `useParameter(id)` | Get/set a single parameter |
| `useAllParameters()` | Get all plugin parameters |
| `useParameterGroups()` | Get parameters organized by group |
| `useMeterFrame()` | Get current audio meter levels |
| `useConnectionStatus()` | Monitor IPC connection status |
| `useRequestResize()` | Request plugin window resize |

### Utilities

\`\`\`typescript
import { linearToDb, dbToLinear } from '@wavecraft/core/meters';

linearToDb(0.5);  // → -6.02 dB
dbToLinear(-6);   // → 0.501
\`\`\`

### Advanced: IPC Bridge

For custom implementations:

\`\`\`typescript
import { IpcBridge, ParameterClient } from '@wavecraft/core';
\`\`\`

## Requirements

- React 18+

## License

MIT
```

---

### Step 7.2: Create Components Package README

**File:** `ui/packages/components/README.md` (NEW)  
**Action:** Write comprehensive README  
**Why:** Displayed on npm registry page  
**Dependencies:** None  
**Risk:** Low

```markdown
# @wavecraft/components

Pre-built React components for Wavecraft audio plugins.

## Installation

\`\`\`bash
npm install @wavecraft/core @wavecraft/components
\`\`\`

> Note: `@wavecraft/core` is a peer dependency and must be installed.

## Quick Start

\`\`\`tsx
import { useAllParameters } from '@wavecraft/core';
import { Meter, ParameterSlider } from '@wavecraft/components';

function PluginUI() {
  const { params } = useAllParameters();

  return (
    <div className="p-4 bg-plugin-dark">
      <Meter />
      {params?.map((p) => (
        <ParameterSlider key={p.id} id={p.id} />
      ))}
    </div>
  );
}
\`\`\`

## TailwindCSS Configuration

Components use TailwindCSS utility classes. Configure your `tailwind.config.js`:

\`\`\`javascript
module.exports = {
  content: [
    './src/**/*.{js,ts,jsx,tsx}',
    './node_modules/@wavecraft/components/dist/**/*.js',
  ],
  theme: {
    extend: {
      colors: {
        'plugin-dark': '#1a1a1a',
        'plugin-surface': '#2a2a2a',
        'plugin-border': '#444444',
        'accent': '#4a9eff',
        'accent-light': '#6bb0ff',
        'meter-safe': '#4caf50',
        'meter-warning': '#ffeb3b',
        'meter-clip': '#ff1744',
      },
    },
  },
};
\`\`\`

## Components

| Component | Description |
|-----------|-------------|
| `Meter` | Audio level meter with peak/RMS display |
| `ParameterSlider` | Slider control for a parameter |
| `ParameterGroup` | Group of related parameters |
| `ParameterToggle` | Boolean parameter toggle |
| `VersionBadge` | Displays plugin version |
| `ConnectionStatus` | Shows IPC connection state |
| `LatencyMonitor` | Displays IPC latency stats |
| `ResizeHandle` | Draggable resize corner |
| `ResizeControls` | Preset size buttons |

## Requirements

- React 18+
- `@wavecraft/core` ^0.7.0
- TailwindCSS 3.x (for styling)

## License

MIT
```

---

## Phase 8: Documentation Updates

**Goal:** Update SDK documentation for npm package workflow.

### Step 8.1: Update SDK Getting Started Guide

**File:** `docs/guides/sdk-getting-started.md`  
**Action:** Update UI section to reference npm packages  
**Why:** External developers need correct instructions  
**Dependencies:** Phase 6  
**Risk:** Low

**Changes:**
- Remove instructions about copying UI code
- Add `npm install @wavecraft/core @wavecraft/components`
- Update import examples to show split packages
- Add TailwindCSS configuration section

---

### Step 8.2: Update High-Level Design

**File:** `docs/architecture/high-level-design.md`  
**Action:** Document npm package architecture  
**Why:** Architecture docs should reflect current state  
**Dependencies:** Phase 6  
**Risk:** Low

**Add section about:**
- Package split rationale
- Dependency graph (`@wavecraft/components` → `@wavecraft/core`)
- Future package namespace (`@wavecraft/pro`, etc.)

---

## Phase 9: Publishing

**Goal:** Publish both packages to npm registry.

### Step 9.1: Final Pre-Publish Checklist

**Action:** Verify all prerequisites  
**Dependencies:** Phases 1-8  
**Risk:** Low

- [ ] npm org `@wavecraft` created
- [ ] `npm login` completed
- [ ] Core package `npm run build:lib` succeeds
- [ ] Components package `npm run build:lib` succeeds
- [ ] Both `npm pack --dry-run` show correct files
- [ ] Local install test passes
- [ ] TypeScript types compile
- [ ] Both README.md files complete
- [ ] Version is 0.7.0 for both packages

---

### Step 9.2: Publish Core Package

**Action:** Publish @wavecraft/core first  
**Why:** Components depends on core  
**Dependencies:** Step 9.1  
**Risk:** Medium — Cannot unpublish within 72 hours

```bash
cd ui/packages/core
npm publish --access public
```

---

### Step 9.3: Publish Components Package

**Action:** Publish @wavecraft/components  
**Why:** Depends on core being published  
**Dependencies:** Step 9.2  
**Risk:** Medium

```bash
cd ui/packages/components
npm publish --access public
```

---

### Step 9.4: Verify Published Packages

**Action:** Test installation from registry  
**Why:** Confirm publish succeeded  
**Dependencies:** Step 9.3  
**Risk:** Low

```bash
cd /tmp
mkdir verify-publish && cd verify-publish
npm init -y
npm install @wavecraft/core @wavecraft/components

# Verify packages
node -e "
import('@wavecraft/core').then(core => {
  console.log('✅ @wavecraft/core exports:', Object.keys(core).length, 'items');
  return import('@wavecraft/components');
}).then(components => {
  console.log('✅ @wavecraft/components exports:', Object.keys(components).length, 'items');
  console.log('✅ Both packages published successfully!');
});
"
```

---

### Step 9.5: Test Template with Published Packages

**Action:** Generate project with CLI and build  
**Why:** End-to-end validation  
**Dependencies:** Step 9.4  
**Risk:** Medium

```bash
cd /tmp
wavecraft new test-plugin --vendor "Test" --email "test@example.com"
cd test-plugin/ui
npm install
npm run build
# Should succeed without errors
```

---

## Phase 10: Cleanup and Finalization

**Goal:** Update tracking and documentation.

### Step 10.1: Update Roadmap

**File:** `docs/roadmap.md`  
**Action:** Mark npm publishing tasks as complete  
**Why:** Track progress  
**Dependencies:** Phase 9  
**Risk:** Low

---

### Step 10.2: Update Implementation Progress

**File:** `docs/feature-specs/open-source-readiness/implementation-progress.md`  
**Action:** Update progress tracking  
**Why:** Document completion  
**Dependencies:** Phase 9  
**Risk:** Low

---

## Summary

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| 1. Workspace Setup | 6 steps | 0.5 day |
| 2. Core Package Infrastructure | 6 steps | 1 day |
| 3. Components Package Infrastructure | 5 steps | 1 day |
| 4. Build Verification | 5 steps | 0.5 day |
| 5. npm Organization Setup | 3 steps | 0.5 day |
| 6. Template Migration | 6 steps | 1 day |
| 7. Package Documentation | 2 steps | 0.25 day |
| 8. Documentation Updates | 2 steps | 0.25 day |
| 9. Publishing | 5 steps | 0.25 day |
| 10. Cleanup | 2 steps | 0.25 day |
| **Total** | **42 steps** | **5.5 days** |

---

## Dependencies Graph

```
Phase 1 (Workspace Setup)
    │
    ├─────────────────────────┐
    ▼                         ▼
Phase 2 (Core Infra)     Phase 3 (Components Infra)
    │                         │
    └──────────┬──────────────┘
               ▼
         Phase 4 (Build Verification)
               │
    ┌──────────┼──────────────┐
    ▼          ▼              ▼
Phase 5    Phase 6         Phase 7
(npm Org)  (Template)      (Pkg Docs)
    │          │              │
    └──────────┼──────────────┘
               ▼
         Phase 8 (Doc Updates)
               │
               ▼
         Phase 9 (Publishing)
               │
               ▼
         Phase 10 (Cleanup)
```

---

## Risk Summary

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Build fails with workspaces | Medium | High | Test thoroughly in Phase 4 |
| npm org taken | Low | Medium | Fallback to unscoped names |
| Circular dependency | Low | High | Components only imports core, never vice versa |
| Type export issues | Medium | Medium | TypeScript test in Phase 4 |
| Template build fails | Medium | High | Test before removing source |
| Version drift between packages | Medium | Medium | Single release process, CI validation |
| Missing exports | Low | Medium | Verify all imports in test |

---

## Success Criteria

- [ ] `npm install @wavecraft/core` works from public registry
- [ ] `npm install @wavecraft/components` works from public registry
- [ ] All hooks, types, and utilities importable from core
- [ ] All components importable from components package
- [ ] TypeScript types included and working for both
- [ ] Components can import from core (peer dep works)
- [ ] Template project builds with both npm packages
- [ ] Package sizes reasonable (core < 30KB, components < 30KB)
- [ ] Documentation complete on npm and in SDK guide
- [ ] Versions synchronized at 0.7.0
