# Coding Standards — TypeScript & React

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [CSS Standards](./coding-standards-css.md) — TailwindCSS and theming
- [Testing Standards](./coding-standards-testing.md) — Testing, logging, error handling

---

## TypeScript / JavaScript

### Class-Based Architecture

**Rule:** Use class syntax for all non-React TypeScript code.

React components should use functional components with hooks, but all other TypeScript code (services, clients, utilities, state management) should use classes.

**Rationale:**
- Classes provide clear encapsulation of state and behavior
- Better IDE support (autocomplete, refactoring)
- Explicit instantiation and lifecycle management
- Easier to mock in tests
- Clear separation between static and instance members

**Do:**
```typescript
// ✅ Service classes
class IpcBridge {
  private requestId = 0;
  private pendingRequests = new Map<number, PendingRequest>();

  async invoke<T>(method: string, params?: unknown): Promise<T> {
    // ...
  }

  on<T>(event: string, callback: (data: T) => void): () => void {
    // ...
  }
}

// ✅ Client classes
class ParameterClient {
  constructor(private bridge: IpcBridge) {}

  async getParameter(id: string): Promise<ParameterInfo> {
    return this.bridge.invoke('getParameter', { id });
  }
}

// ✅ Singleton pattern when appropriate
class IpcClient {
  private static instance: IpcClient | null = null;

  static getInstance(): IpcClient {
    if (!IpcClient.instance) {
      IpcClient.instance = new IpcClient();
    }
    return IpcClient.instance;
  }

  private constructor() {
    // Private constructor for singleton
  }
}
```

**Don't:**
```typescript
// ❌ Exported functions with module-level state
let requestId = 0;
const pendingRequests = new Map();

export async function invoke<T>(method: string, params?: unknown): Promise<T> {
  // ...
}

export function on(event: string, callback: Function): () => void {
  // ...
}
```

### React Components

**Rule:** Use functional components with hooks for all React code.

```typescript
// ✅ Functional component with hooks
export function ParameterSlider({ id, name, min, max }: Props) {
  const { param, setValue } = useParameter(id);
  
  return (
    <input
      type="range"
      value={param?.value ?? 0}
      onChange={(e) => setValue(parseFloat(e.target.value))}
    />
  );
}

// ❌ Class components (avoid)
class ParameterSlider extends React.Component<Props> {
  // ...
}
```

### Custom Hooks

Custom React hooks bridge between class-based services and React components:

```typescript
// Hook wraps class-based client for React integration
export function useParameter(id: string) {
  const [param, setParam] = useState<ParameterInfo | null>(null);
  const client = ParameterClient.getInstance();

  useEffect(() => {
    client.getParameter(id).then(setParam);
  }, [id]);

  const setValue = useCallback(async (value: number) => {
    await client.setParameter(id, value);
  }, [id]);

  return { param, setValue };
}
```

### Environment-Aware Hooks

**Rule:** Hooks that depend on runtime environment must detect environment at module scope.

When hooks need different behavior based on runtime environment (e.g., browser vs WKWebView), the environment check must be evaluated once at module load time, not inside the hook body.

**Rationale:**
- React's Rules of Hooks require consistent hook call order across renders
- Conditional hook behavior inside the hook body can violate this rule
- Module-scope evaluation ensures the condition is stable

**Do:**
```typescript
// ✅ Environment detection at module scope
import { isBrowserEnvironment } from './environment';

// Evaluated once when module loads
const IS_BROWSER = isBrowserEnvironment();

export function useParameter(id: string) {
  // IS_BROWSER is stable - same value for all renders
  const [param, setParam] = useState(IS_BROWSER ? mockData : null);
  
  useEffect(() => {
    if (IS_BROWSER) return; // Safe: IS_BROWSER never changes
    // ... fetch real data
  }, [id]);
}
```

**Don't:**
```typescript
// ❌ Environment detection inside hook
export function useParameter(id: string) {
  // BAD: Called on every render, could theoretically change
  if (isBrowserEnvironment()) {
    return { param: mockData };  // Violates Rules of Hooks!
  }
  // ... rest of hook
}
```

### Build-Time Constants

**Rule:** Use Vite's `define` block for compile-time constants that come from the build system.

Values that need to be injected at build time (e.g., version from Cargo.toml) should use Vite's `define` configuration rather than environment variables.

**Configuration (`vite.config.ts`):**
```typescript
export default defineConfig({
  define: {
    // Compile-time replacement, falls back to 'dev' for local npm run dev
    '__APP_VERSION__': JSON.stringify(process.env.VITE_APP_VERSION || 'dev'),
  },
});
```

**Type Declaration (`vite-env.d.ts`):**
```typescript
declare const __APP_VERSION__: string;
```

**Usage:**
```typescript
// ✅ Use the constant directly - it's replaced at build time
<span>v{__APP_VERSION__}</span>
```

**Why `define` over `.env` files:**
- Build system (xtask) can inject values via environment variables
- `define` creates compile-time replacement (zero runtime cost)
- No risk of `.env` files drifting from source of truth
- Clear TypeScript typing via declaration file

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Classes | PascalCase | `IpcBridge`, `ParameterClient` |
| Interfaces | PascalCase | `ParameterInfo`, `IpcError` |
| Type aliases | PascalCase | `EventCallback`, `RequestId` |
| Methods | camelCase | `getParameter`, `setReceiveCallback` |
| Private members | camelCase (no underscore prefix) | `private requestId` |
| Constants | UPPER_SNAKE_CASE | `DEFAULT_TIMEOUT_MS` |
| React components | PascalCase | `ParameterSlider` |
| React hooks | camelCase with `use` prefix | `useParameter` |

### File Organization

The UI codebase is organized as an npm workspace with publishable packages:

```
ui/
├── packages/                      # Published npm packages
│   ├── core/                      # @wavecraft/core
│   │   ├── src/
│   │   │   ├── index.ts           # Main entry (re-exports all public API)
│   │   │   ├── meters.ts          # /meters subpath (pure audio math)
│   │   │   ├── hooks/             # React hooks (domain folder)
│   │   │   │   ├── useParameter.ts
│   │   │   │   ├── useAllParameters.ts
│   │   │   │   ├── useParameterGroups.ts
│   │   │   │   ├── useConnectionStatus.ts
│   │   │   │   ├── useLatencyMonitor.ts
│   │   │   │   ├── useMeterFrame.ts
│   │   │   │   ├── useRequestResize.ts
│   │   │   │   └── useWindowResizeSync.ts
│   │   │   ├── ipc/               # IPC classes (domain folder)
│   │   │   │   ├── IpcBridge.ts   # Low-level IPC bridge
│   │   │   │   └── ParameterClient.ts # High-level parameter API
│   │   │   ├── types/             # TypeScript types (domain folder)
│   │   │   │   ├── ipc.ts         # IPC protocol types
│   │   │   │   ├── parameters.ts  # Parameter types
│   │   │   │   └── metering.ts    # Meter types
│   │   │   ├── utils/             # Utilities (domain folder)
│   │   │   │   ├── environment.ts # Runtime detection
│   │   │   │   └── audio-math.ts  # linearToDb, dbToLinear
│   │   │   ├── transports/        # Transport implementations
│   │   │   └── logger/            # Structured logging
│   │   ├── dist/                  # Built ESM bundle + DTS
│   │   └── package.json           # npm package config
│   └── components/                # @wavecraft/components
│       ├── src/
│       │   ├── index.ts           # Component exports
│       │   ├── Meter.tsx          # Audio level meter
│       │   ├── ParameterSlider.tsx# Slider control
│       │   ├── ParameterGroup.tsx # Grouped parameters
│       │   └── VersionBadge.tsx   # Version display
│       ├── dist/                  # Built ESM bundle + DTS
│       └── package.json           # npm package config
├── src/                           # Development app (internal)
│   ├── App.tsx                    # Dev app entry
│   └── main.tsx                   # Dev app bootstrap
├── test/
│   ├── setup.ts                   # Global test setup
│   └── mocks/
│       └── ipc.ts                 # IPC mock module
└── package.json                   # Workspace root (workspaces: ["packages/*"])
```

**Domain folder conventions:**
- **No internal barrel files** — Domain folders do not have `index.ts` barrels
- The main `src/index.ts` imports directly from each file (e.g., `./hooks/useParameter`)
- One file per hook/class/type domain
- External imports use `@wavecraft/core` or `@wavecraft/components`

### Barrel Files

**Rule:** Use barrel files only for published package entry points, not for internal folders.

**Rationale:**
- Internal barrels can defeat tree-shaking in application code
- They can mask circular dependencies
- They slow down TypeScript IDE performance
- Published packages are pre-bundled, so the main entry barrel is acceptable

**Do:**
```typescript
// ✅ Main entry point (src/index.ts) - this IS the public API
export { useParameter } from './hooks/useParameter';
export { IpcBridge } from './ipc/IpcBridge';
export type { ParameterInfo } from './types/parameters';

// ✅ Internal imports use direct paths
import { IpcBridge } from './ipc/IpcBridge';
import type { MeterFrame } from './types/metering';
```

**Don't:**
```typescript
// ❌ Internal barrel file (hooks/index.ts)
export { useParameter } from './useParameter';
export { useAllParameters } from './useAllParameters';
// ...then importing from barrel
import { useParameter } from './hooks';  // Avoid this pattern
```

### Import Aliases

**Rule:** Use npm package imports for the SDK, not relative paths.

The UI SDK is distributed as npm packages. User plugins and the internal dev app both import from these packages:

| Package | Exports | Usage |
|---------|---------|-------|
| `@wavecraft/core` | IPC, hooks, Logger, types | Core SDK functionality |
| `@wavecraft/core/meters` | `linearToDb`, `dbToLinear` | Pure audio math utilities |
| `@wavecraft/components` | `Meter`, `ParameterSlider`, etc. | Pre-built React components |

**Do:**
```typescript
// ✅ Import from npm packages
import { useParameter, useAllParameters, Logger } from '@wavecraft/core';
import { linearToDb, dbToLinear } from '@wavecraft/core/meters';
import { Meter, ParameterSlider } from '@wavecraft/components';
```

**Don't:**
```typescript
// ❌ Relative imports (no longer applicable for SDK usage)
import { useParameter } from '../lib/wavecraft-ipc';
import { Meter } from '../../components/Meter';
```

**Subpath Exports:**

The `@wavecraft/core/meters` subpath provides access to pure utility functions (`linearToDb`, `dbToLinear`, `getMeterFrame`) without triggering IPC initialization side effects. Use this subpath:
- In unit tests for pure functions
- When you only need math utilities, not hooks or clients

**Rationale:**
- Standard npm package consumption pattern
- Version management via package.json
- Clear separation between SDK and user code
- Tree-shaking support via ESM exports
- Subpath exports avoid initialization side effects in tests

### Global Object Access

**Rule:** Use `globalThis` instead of `window` for accessing the global object.

This applies to:
- TypeScript/JavaScript source files
- JavaScript strings embedded in Rust code (e.g., `evaluate_script` calls)

**Do:**
```typescript
// ✅ Use globalThis in TypeScript/JavaScript
globalThis.wavecraft?.invoke('getParameter', { id });
globalThis.addEventListener('message', handler);
```

```rust
// ✅ Use globalThis in embedded JavaScript (Rust)
let js = format!(
    "if (globalThis.__WAVECRAFT_IPC__ && globalThis.__WAVECRAFT_IPC__._onParamUpdate) {{ \
        globalThis.__WAVECRAFT_IPC__._onParamUpdate({{ id: '{}', value: {} }}); \
    }}",
    id, value
);
webview.evaluate_script(&js);
```

**Don't:**
```typescript
// ❌ Using window in TypeScript/JavaScript
window.wavecraft?.invoke('getParameter', { id });
window.addEventListener('message', handler);
```

```rust
// ❌ Using window in embedded JavaScript (Rust)
let js = format!(
    "if (window.__WAVECRAFT_IPC__) {{ window.__WAVECRAFT_IPC__._onParamUpdate(...); }}"
);
```

**Rationale:**
- `globalThis` is the standardized way to access the global object (ES2020+)
- Works consistently across all JavaScript environments (browser, Node.js, Web Workers, etc.)
- Plugin UI may run in different contexts where `window` is not available
- Future-proofs code against environment changes
