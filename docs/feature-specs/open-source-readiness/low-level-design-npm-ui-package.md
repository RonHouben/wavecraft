# Low-Level Design: npm UI Package Publishing

**Feature:** User Story 9 — UI Package Publishing  
**Parent Feature:** Open Source Readiness (Milestone 12)  
**Author:** Architect Agent  
**Date:** 2026-02-04  
**Status:** Draft

---

## 1. Overview

### 1.1 Problem Statement

Currently, the template project (`wavecraft-plugin-template`) includes a **full copy** of the UI code:
- `ui/src/lib/wavecraft-ipc/` — 15+ files (IPC bridge, hooks, types, transports)
- `ui/src/components/` — 4 components (Meter, ParameterSlider, VersionBadge, LatencyMonitor)

This causes:
1. **Code duplication** — Same code exists in monorepo and template
2. **Sync drift** — Template may fall behind monorepo fixes
3. **No versioning** — Users can't pin to a specific version
4. **No reusability** — Users building custom UIs must copy code manually

### 1.2 Solution

Publish two npm packages under the `@wavecraft` organization:

```bash
# Core SDK (IPC, hooks, types, utilities)
npm install @wavecraft/core

# Pre-built components (optional)
npm install @wavecraft/components
```

```typescript
// Core SDK - always needed
import { useParameter, useMeterFrame, logger } from '@wavecraft/core';

// Pre-built components - optional convenience
import { Meter, ParameterSlider } from '@wavecraft/components';
```

### 1.3 Architectural Decision: Split Packages

**Decision:** Publish two packages: `@wavecraft/core` (SDK foundation) and `@wavecraft/components` (UI widgets).

**Alternatives Considered:**

| Option | Pros | Cons |
|--------|------|------|
| **A: Single `@wavecraft/ui`** | Simple, one dependency | Poor namespace for future premium packages |
| **B: Split `@wavecraft/core` + `@wavecraft/components`** | Clean namespace, future-proof, flexible | Two packages to maintain |
| **C: Monorepo with many `@wavecraft/*` packages** | Maximum flexibility | Overkill for current scope |

**Rationale for Option B:**

1. **Future-proof namespace** — Reserves clean names for premium offerings:
   - `@wavecraft/pro` — Premium components (future, separate repo)
   - `@wavecraft/presets` — Preset management (future)
   - `@wavecraft/themes` — Theme packages (future)

2. **Clean dependency graph:**
   ```
   @wavecraft/pro ─────────┐
                           ├──► @wavecraft/core (peer dependency)
   @wavecraft/components ──┘
   ```

3. **User flexibility:**
   - Minimal SDK users: `npm install @wavecraft/core` (build custom UI)
   - Full experience: `npm install @wavecraft/core @wavecraft/components`
   - Premium users: `npm install @wavecraft/core @wavecraft/pro`

4. **Licensing clarity:**
   - `@wavecraft/core` — MIT (always free, attracts adoption)
   - `@wavecraft/components` — MIT (community can contribute)
   - `@wavecraft/pro` — Commercial license (future, separate repo)

---

## 2. Package Structure

### 2.1 Directory Layout

The UI codebase adopts a monorepo structure with npm workspaces:

```
ui/
├── package.json              # Root workspace configuration
├── packages/
│   ├── core/                 # @wavecraft/core
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   ├── tsconfig.build.json
│   │   ├── vite.lib.config.ts
│   │   ├── src/
│   │   │   ├── index.ts      # Main entry point
│   │   │   ├── meters.ts     # Subpath entry (pure utilities)
│   │   │   ├── bridge/       # IpcBridge, transports
│   │   │   ├── client/       # ParameterClient
│   │   │   ├── hooks/        # useParameter, useMeterFrame, etc.
│   │   │   ├── types/        # TypeScript definitions
│   │   │   └── utils/        # Logger, environment detection
│   │   └── dist/             # Build output (gitignored)
│   │
│   └── components/           # @wavecraft/components
│       ├── package.json
│       ├── tsconfig.json
│       ├── tsconfig.build.json
│       ├── vite.lib.config.ts
│       ├── src/
│       │   ├── index.ts      # Component exports
│       │   ├── Meter.tsx
│       │   ├── ParameterSlider.tsx
│       │   ├── VersionBadge.tsx
│       │   ├── LatencyMonitor.tsx
│       │   ├── ConnectionStatus.tsx
│       │   ├── ParameterGroup.tsx
│       │   ├── ParameterToggle.tsx
│       │   ├── ResizeHandle.tsx
│       │   └── ResizeControls.tsx
│       └── dist/             # Build output (gitignored)
│
├── src/                      # Development app (not published)
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── vite.config.ts            # Dev server config
└── vitest.config.ts          # Test config
```

### 2.2 Workspace Configuration

**Root `package.json`:**
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
    "lint": "eslint . && prettier --check ."
  }
}
```

### 2.3 Entry Points

**Core entry (`packages/core/src/index.ts`):**
```typescript
// Hooks (Primary API)
export { useParameter, type UseParameterResult } from './hooks/useParameter';
export { useAllParameters, type UseAllParametersResult } from './hooks/useAllParameters';
export { useParameterGroups } from './hooks/useParameterGroups';
export { useMeterFrame } from './hooks/useMeterFrame';
export { useConnectionStatus } from './hooks/useConnectionStatus';
export { useRequestResize } from './hooks/useRequestResize';
export { useLatencyMonitor, type UseLatencyMonitorResult } from './hooks/useLatencyMonitor';

// Types
export type { ParameterInfo, ParameterGroup, MeterFrame, ConnectionStatus } from './types';

// IPC utilities (Advanced)
export { IpcBridge } from './bridge/IpcBridge';
export { ParameterClient } from './client/ParameterClient';
export { NativeTransport } from './bridge/NativeTransport';
export { WebSocketTransport } from './bridge/WebSocketTransport';

// Logging
export { logger, Logger, LogLevel } from './utils/logger';
```

**Core meters subpath (`packages/core/src/meters.ts`):**
```typescript
// Pure utilities with no IPC side effects
export { getMeterFrame, linearToDb, dbToLinear } from './utils/audio-math';
export type { MeterFrame } from './types';
```

**Components entry (`packages/components/src/index.ts`):**
```typescript
export { Meter, type MeterProps } from './Meter';
export { ParameterSlider, type ParameterSliderProps } from './ParameterSlider';
export { ParameterGroup, type ParameterGroupProps } from './ParameterGroup';
export { ParameterToggle, type ParameterToggleProps } from './ParameterToggle';
export { VersionBadge, type VersionBadgeProps } from './VersionBadge';
export { ConnectionStatus } from './ConnectionStatus';
export { LatencyMonitor } from './LatencyMonitor';
export { ResizeHandle } from './ResizeHandle';
export { ResizeControls, type ResizeControlsProps } from './ResizeControls';
```

---

## 3. Package Configuration

### 3.1 @wavecraft/core package.json

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
    "audio",
    "plugin",
    "react",
    "webview",
    "ipc",
    "sdk"
  ],
  "repository": {
    "type": "git",
    "url": "https://github.com/RonHouben/wavecraft.git",
    "directory": "ui/packages/core"
  },
  "license": "MIT",
  "peerDependencies": {
    "react": "^18.0.0"
  },
  "scripts": {
    "build:lib": "vite build --config vite.lib.config.ts && tsc --project tsconfig.build.json --emitDeclarationOnly",
    "prepublishOnly": "npm run build:lib"
  }
}
```

### 3.2 @wavecraft/components package.json

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
    "slider"
  ],
  "repository": {
    "type": "git",
    "url": "https://github.com/RonHouben/wavecraft.git",
    "directory": "ui/packages/components"
  },
  "license": "MIT",
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
    "build:lib": "vite build --config vite.lib.config.ts && tsc --project tsconfig.build.json --emitDeclarationOnly",
    "prepublishOnly": "npm run build:lib"
  }
}
```

### 3.3 Key Configuration Decisions

| Field | Value | Rationale |
|-------|-------|-----------|
| `type: "module"` | ESM-only | Modern standard, better tree-shaking |
| `sideEffects: false` | Enable tree-shaking | Bundlers can eliminate unused exports |
| `peerDependencies` | React 18+, @wavecraft/core (for components) | Users provide their own React; components depend on core |
| `exports` | Subpath exports | Allows `@wavecraft/core/meters` import |
| `files` | `["dist", "README.md"]` | Only publish built assets |

### 3.4 Build Configuration (`packages/core/vite.lib.config.ts`)

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
    sourcemap: true,
    minify: false,
  },
});
```

### 3.5 Build Configuration (`packages/components/vite.lib.config.ts`)

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
    sourcemap: true,
    minify: false,
  },
});
```

### 3.6 TypeScript Build Configuration

Each package has its own `tsconfig.build.json`:

```json
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "declaration": true,
    "declarationDir": "./dist",
    "emitDeclarationOnly": true,
    "noEmit": false
  },
  "include": ["src"],
  "exclude": ["src/**/*.test.ts", "src/**/*.test.tsx"]
}
```

---

## 4. Export Strategy

### 4.1 @wavecraft/core Public API

**Hooks (Primary API):**
```typescript
// Parameter management
useParameter(id: string): UseParameterResult
useAllParameters(): UseAllParametersResult
useParameterGroups(): ParameterGroup[]

// Metering
useMeterFrame(): MeterFrame | null

// Connection
useConnectionStatus(): ConnectionStatus

// Resize
useRequestResize(): (width: number, height: number) => Promise<void>

// Debugging
useLatencyMonitor(): UseLatencyMonitorResult
```

**IPC Utilities (Advanced):**
```typescript
// Bridge and client (for custom implementations)
IpcBridge: class
ParameterClient: class

// Transports
NativeTransport: class
WebSocketTransport: class

// Logging
logger: Logger
Logger: class
LogLevel: enum
```

**Pure Utilities (Subpath Export):**
```typescript
// Available via '@wavecraft/core/meters'
getMeterFrame(): Promise<MeterFrame>
linearToDb(linear: number): number
dbToLinear(db: number): number
```

### 4.2 @wavecraft/components Public API

**Components:**
```typescript
// Core components
Meter: React.FC<MeterProps>
ParameterSlider: React.FC<ParameterSliderProps>
ParameterGroup: React.FC<ParameterGroupProps>
ParameterToggle: React.FC<ParameterToggleProps>
VersionBadge: React.FC<VersionBadgeProps>

// Utility components
ConnectionStatus: React.FC
LatencyMonitor: React.FC
ResizeHandle: React.FC
ResizeControls: React.FC<ResizeControlsProps>
```

### 4.3 What NOT to Export

- Test utilities (`src/test/mocks/`)
- Test files (`*.test.ts`, `*.test.tsx`)
- App-specific code (`App.tsx`, `main.tsx`)
- Development styles (`index.css`) — Users bring their own TailwindCSS

---

## 5. Styling Strategy

### 5.1 Decision: No Bundled CSS

**Decision:** Do NOT bundle CSS with the npm package.

**Rationale:**
1. Components use TailwindCSS utility classes
2. Users likely have their own Tailwind setup
3. Bundling CSS would cause conflicts and bloat
4. TailwindCSS JIT compiles only used classes

**User Responsibility:**
Users must configure TailwindCSS to scan `@wavecraft/components`:

```javascript
// tailwind.config.js
module.exports = {
  content: [
    './src/**/*.{js,ts,jsx,tsx}',
    './node_modules/@wavecraft/components/dist/**/*.js', // Add this
  ],
  // ... rest of config
};
```

### 5.2 Theme Tokens

Document required theme tokens in README:

```javascript
// tailwind.config.js
module.exports = {
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
```

---

## 6. Version Strategy

### 6.1 Version Synchronization

**Rule:** Both `@wavecraft/core` and `@wavecraft/components` versions MUST match the SDK version.

| SDK Version | @wavecraft/core | @wavecraft/components |
|-------------|-----------------|----------------------|
| v0.7.0 | 0.7.0 | 0.7.0 |
| v0.7.1 | 0.7.1 | 0.7.1 |
| v0.8.0 | 0.8.0 | 0.8.0 |

**Enforcement:**
- `cli/` embeds the expected npm versions in template `package.json`
- CI validates version consistency before release
- Both packages published together in same release

### 6.2 Breaking Change Policy

| Version Bump | When |
|--------------|------|
| Patch (0.7.x) | Bug fixes, docs, internal refactoring |
| Minor (0.x.0) | New features, non-breaking API additions |
| Major (x.0.0) | Breaking API changes (post-1.0) |

### 6.3 Future Package Compatibility

When `@wavecraft/pro` is introduced (separate repo, commercial license):

| Package | Peer Dependency |
|---------|-----------------|
| `@wavecraft/components` | `@wavecraft/core: ^0.7.0` |
| `@wavecraft/pro` | `@wavecraft/core: ^0.7.0` |

This allows users to mix free and premium components with the same core.

---

## 7. Template Integration

### 7.1 Before (Current State)

```
wavecraft-plugin-template/ui/
├── src/
│   ├── lib/
│   │   └── wavecraft-ipc/     # FULL COPY (~15 files)
│   └── components/
│       ├── Meter.tsx          # FULL COPY
│       ├── ParameterSlider.tsx
│       └── ...
└── package.json               # No @wavecraft/ui dependency
```

### 7.2 After (Target State)

```
wavecraft-plugin-template/ui/
├── src/
│   ├── App.tsx                # Imports from @wavecraft packages
│   ├── main.tsx
│   └── index.css              # User's TailwindCSS
└── package.json               # Has @wavecraft/* dependencies
```

**Template `package.json`:**
```json
{
  "dependencies": {
    "@wavecraft/core": "^0.7.0",
    "@wavecraft/components": "^0.7.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  }
}
```

**Template `App.tsx`:**
```tsx
import {
  useParameter,
  useAllParameters,
  useConnectionStatus,
} from '@wavecraft/core';

import {
  Meter,
  ParameterSlider,
  VersionBadge,
  ConnectionStatus,
} from '@wavecraft/components';

function App() {
  const { params } = useAllParameters();
  const connectionStatus = useConnectionStatus();

  return (
    <div className="p-4 bg-plugin-dark min-h-screen">
      <VersionBadge />
      <ConnectionStatus />
      <Meter />
      {params?.map((p) => (
        <ParameterSlider key={p.id} {...p} />
      ))}
    </div>
  );
}
```

### 7.3 CLI Template Updates

The `wavecraft new` CLI must:
1. Generate `package.json` with `@wavecraft/core` and `@wavecraft/components` dependencies
2. Generate simplified `App.tsx` that imports from npm packages
3. Generate `tailwind.config.js` with content path for `@wavecraft/components`
4. NOT include `src/lib/wavecraft-ipc/` directory
5. NOT include component source files

---

## 8. npm Organization Setup

### 8.1 Organization Registration

**Required:** Create `@wavecraft` organization on npm.

**Steps:**
1. Create npm account (if not exists)
2. Create organization: https://www.npmjs.com/org/create
3. Organization name: `wavecraft`
4. Add team members (optional)

**Fallback:** If `@wavecraft` is taken, use `wavecraft-ui` (unscoped).

### 8.2 Publishing Workflow

**Manual Publishing (Initial):**
```bash
# Build both packages
cd ui
npm run build:lib --workspaces

# Publish core first (components depends on it)
cd packages/core && npm publish --access public
cd ../components && npm publish --access public
```

**Automated Publishing (Future):**
```yaml
# .github/workflows/npm-publish.yml
name: Publish to npm
on:
  release:
    types: [published]
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - run: cd ui && npm ci
      - run: cd ui && npm run build:lib --workspaces
      - run: cd ui/packages/core && npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
      - run: cd ui/packages/components && npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

---

## 9. Testing Strategy

### 9.1 Pre-Publish Validation

```bash
# 1. Build both packages
cd ui && npm run build:lib --workspaces

# 2. Verify dist contents
ls -la packages/core/dist/
# Expected: index.js, index.d.ts, meters.js, meters.d.ts

ls -la packages/components/dist/
# Expected: index.js, index.d.ts

# 3. Test with npm pack (dry run)
cd packages/core && npm pack --dry-run
cd ../components && npm pack --dry-run
# Verify only dist/ and README.md are included

# 4. Local install test
cd /tmp && mkdir test-consumer && cd test-consumer
npm init -y
npm install /path/to/wavecraft/ui/packages/core
npm install /path/to/wavecraft/ui/packages/components

# 5. Verify imports work
node -e "import('@wavecraft/core').then(m => console.log('core:', Object.keys(m)))"
node -e "import('@wavecraft/components').then(m => console.log('components:', Object.keys(m)))"
```

### 9.2 Integration Test

Create a minimal consumer app that:
1. Installs both packages from local tarballs
2. Imports all public exports
3. Renders components in a test
4. Verifies TypeScript types compile
5. Verifies components can use hooks from core

---

## 10. Documentation

### 10.1 Package READMEs

Each package needs its own README.

**@wavecraft/core README:**

1. **Installation**
   ```bash
   npm install @wavecraft/core
   ```

2. **Quick Start**
   ```tsx
   import { useParameter, useMeterFrame, logger } from '@wavecraft/core';
   ```

3. **API Reference**
   - Hooks with signatures
   - IPC utilities
   - Logger API

4. **Compatibility**
   - React 18+ required
   - Works with Vite, webpack, Next.js

**@wavecraft/components README:**

1. **Installation**
   ```bash
   npm install @wavecraft/core @wavecraft/components
   ```

2. **Quick Start**
   ```tsx
   import { Meter, ParameterSlider } from '@wavecraft/components';
   ```

3. **TailwindCSS Configuration**
   - Required content paths
   - Theme token configuration

4. **Component Reference**
   - All components with props

5. **Compatibility**
   - Requires `@wavecraft/core` as peer dependency
   - Requires TailwindCSS 3.x

### 10.2 SDK Getting Started Updates

Update `docs/guides/sdk-getting-started.md`:
- Remove references to copying UI code
- Add `npm install` step
- Update import examples
- Document TailwindCSS setup

---

## 11. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| npm org `@wavecraft` taken | Low | Medium | Fallback to `wavecraft-core` / `wavecraft-components` (unscoped) |
| Package version drift | Medium | High | CI validation, single release process, workspace tooling |
| TailwindCSS conflicts | Medium | Medium | Document required setup clearly |
| Circular dependency between packages | Low | High | Components only imports from core, never vice versa |
| Breaking changes in React | Low | Medium | Peer dependency range, testing matrix |
| Two packages harder to maintain | Medium | Low | Workspace tooling, shared configs, automated versioning |

---

## 12. Implementation Phases

### Phase 1: Workspace Setup (0.5 day)
- [ ] Create `ui/packages/` directory structure
- [ ] Set up npm workspaces in root `package.json`
- [ ] Move IPC code to `packages/core/src/`
- [ ] Move component code to `packages/components/src/`
- [ ] Update import paths in dev app (`ui/src/`)

### Phase 2: Core Package Infrastructure (1 day)
- [ ] Create `packages/core/package.json`
- [ ] Create `packages/core/vite.lib.config.ts`
- [ ] Create `packages/core/tsconfig.build.json`
- [ ] Add `packages/core/src/index.ts` entry point
- [ ] Add `packages/core/src/meters.ts` subpath entry
- [ ] Add `vite-plugin-dts` dependency

### Phase 3: Components Package Infrastructure (1 day)
- [ ] Create `packages/components/package.json`
- [ ] Create `packages/components/vite.lib.config.ts`
- [ ] Create `packages/components/tsconfig.build.json`
- [ ] Add `packages/components/src/index.ts` entry point
- [ ] Configure peer dependency on `@wavecraft/core`

### Phase 4: Build Verification (0.5 day)
- [ ] Run `npm run build:lib --workspaces`
- [ ] Verify `dist/` output for both packages
- [ ] Test with `npm pack --dry-run`
- [ ] Local install test in temp directory
- [ ] Verify components can import from core

### Phase 5: npm Organization (0.5 day)
- [ ] Create npm account (if needed)
- [ ] Create `@wavecraft` organization
- [ ] Test publish with `--dry-run`

### Phase 6: Template Migration (1 day)
- [ ] Update CLI template `package.json` (both deps)
- [ ] Update CLI template `App.tsx` (split imports)
- [ ] Update CLI template `tailwind.config.js`
- [ ] Remove copied source files from template
- [ ] Update template README

### Phase 7: Documentation (0.5 day)
- [ ] Write `@wavecraft/core` README
- [ ] Write `@wavecraft/components` README
- [ ] Update SDK Getting Started guide
- [ ] Update high-level design docs

### Phase 8: Publishing (0.5 day)
- [ ] Final `npm pack` verification for both packages
- [ ] Publish `@wavecraft/core@0.7.0`
- [ ] Publish `@wavecraft/components@0.7.0`
- [ ] Verify install from npm registry
- [ ] Test template with published packages

**Total Estimated Effort:** 5.5 days

---

## 13. Success Criteria

1. **`npm install @wavecraft/core` works** from public registry
2. **`npm install @wavecraft/components` works** from public registry
3. **All exports importable** with TypeScript support
4. **Template uses npm packages** instead of copied code
5. **Tree-shaking works** — unused exports excluded from bundle
6. **Documentation complete** — READMEs, SDK guide updated
7. **Versions match SDK** — @wavecraft/core@0.7.0 and @wavecraft/components@0.7.0 with engine v0.7.0
8. **Components peer-depend on core** — Clean dependency graph established

---

## 14. Open Questions

| Question | Owner | Status |
|----------|-------|--------|
| Should we include source maps in npm packages? | Architect | Open |
| Do we need a CHANGELOG.md in each package? | PO | Open |
| Should we set up automated npm publish on release? | Coder | Deferred to post-M12 |
| Use npm workspaces or pnpm/turborepo for monorepo? | Architect | Resolved: npm workspaces (simpler) |

---

## 15. Future Roadmap

### Reserved Package Names

| Package | Purpose | Timeline |
|---------|---------|----------|
| `@wavecraft/core` | IPC, hooks, types, utilities | M12 (this feature) |
| `@wavecraft/components` | Free UI components | M12 (this feature) |
| `@wavecraft/pro` | Premium components | Future (separate repo) |
| `@wavecraft/presets` | Preset management | Future |
| `@wavecraft/themes` | UI themes | Future |

### Dependency Graph (Future State)

```
┌─────────────────────┐     ┌───────────────────────┐
│  @wavecraft/pro     │     │ @wavecraft/components │
│  (Commercial)       │     │ (MIT)                 │
└─────────┬───────────┘     └───────────┬───────────┘
          │                             │
          │    peerDependency           │ peerDependency
          │                             │
          └──────────┬──────────────────┘
                     │
                     ▼
          ┌─────────────────────┐
          │   @wavecraft/core   │
          │   (MIT)             │
          └─────────────────────┘
```

---

## 16. References

- [User Story 9: UI Package Publishing](user-stories.md#user-story-9-ui-package-publishing)
- [Roadmap: Milestone 12](../../roadmap.md#milestone-12-open-source-readiness-)
- [Coding Standards: Import Aliases](../../architecture/coding-standards.md#import-aliases)
- [High-Level Design: UI Architecture](../../architecture/high-level-design.md)
