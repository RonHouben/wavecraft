# Coding Standards

This document defines the coding standards and conventions for the VstKit project.

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

```
src/
├── lib/
│   └── vstkit-ipc/
│       ├── index.ts          # Public exports
│       ├── types.ts          # Type definitions
│       ├── IpcBridge.ts      # Class: low-level bridge
│       ├── ParameterClient.ts # Class: high-level API
│       └── hooks.ts          # React hooks (functional)
├── components/
│   ├── ParameterSlider.tsx   # Functional component
│   └── LatencyMonitor.tsx    # Functional component
└── App.tsx
```

**Note:** Class files should be named with PascalCase matching the class name.

---

## Rust

### Module Organization

Follow the existing crate structure:
- `protocol` — Shared contracts and types
- `dsp` — Pure DSP code (no framework dependencies)
- `plugin` — nih-plug integration
- `bridge` — IPC handling

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `IpcHandler`, `AppState` |
| Traits | PascalCase | `ParameterHost` |
| Functions | snake_case | `handle_request`, `get_parameter` |
| Methods | snake_case | `fn set_sample_rate(&mut self)` |
| Constants | UPPER_SNAKE_CASE | `const WINDOW_WIDTH: u32` |
| Modules | snake_case | `mod params`, `mod handler` |

### Real-Time Safety

Code running on the audio thread must:
- Never allocate (`Box::new`, `Vec::push`, `String::from`)
- Never lock (`Mutex`, `RwLock`)
- Never make system calls that can block
- Use atomic types for shared state
- Use SPSC ring buffers for data transfer

---

## General

### Comments and Documentation

- Use `///` doc comments for public APIs
- Include examples in doc comments where helpful
- Keep comments up-to-date with code changes

### Error Handling

- TypeScript: Use explicit error types, avoid `any`
- Rust: Use `Result<T, E>` with descriptive error types
- Always handle errors explicitly; avoid silent failures
