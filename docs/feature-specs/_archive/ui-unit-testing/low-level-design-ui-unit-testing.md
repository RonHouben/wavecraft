# UI Unit Testing Framework — Low-Level Design

## Overview

This document describes the technical design for adding unit testing infrastructure to the VstKit React UI layer.

---

## Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **Test Runner** | Vitest 3.x | Native Vite integration, Jest-compatible API, ESM-first, fast HMR watch mode |
| **Testing Library** | React Testing Library | Tests behavior over implementation, encourages accessible queries |
| **DOM Environment** | happy-dom | 2-3x faster than jsdom, sufficient DOM coverage for component tests |
| **Assertions** | Vitest built-in + @testing-library/jest-dom | Standard matchers plus DOM-specific assertions |

---

## File Structure

### Test File Location (Co-located)

Tests live alongside source files for discoverability:

```
ui/src/
├── components/
│   ├── Meter.tsx
│   ├── Meter.test.tsx          # ← Test file next to component
│   ├── ParameterSlider.tsx
│   └── ParameterSlider.test.tsx
├── lib/
│   ├── ipc.ts
│   ├── ipc.test.ts             # ← Hook tests
│   ├── audio-math.ts
│   └── audio-math.test.ts      # ← Utility tests
└── test/
    ├── setup.ts                # ← Global test setup
    └── mocks/
        └── ipc.ts              # ← Centralized IPC mocks
```

### Configuration Files

```
ui/
├── vitest.config.ts            # ← Vitest configuration
├── vitest.setup.ts             # ← Global setup (imports, mocks)
├── package.json                # ← Test scripts
└── tsconfig.json               # ← Include test types
```

---

## Vitest Configuration

```typescript
// ui/vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      exclude: ['src/test/**', '**/*.test.{ts,tsx}'],
    },
  },
});
```

---

## Mock Strategy

### IPC Mock Module

The core challenge is mocking the IPC layer so components can be tested without the Rust engine.

```typescript
// ui/src/test/mocks/ipc.ts

// Mock state (controllable from tests)
let mockParameters: Map<number, number> = new Map();
let mockMeterFrame: MeterFrame | null = null;

// Test configuration API
export function setMockParameter(id: number, value: number): void {
  mockParameters.set(id, value);
}

export function setMockMeterFrame(frame: MeterFrame): void {
  mockMeterFrame = frame;
}

export function resetMocks(): void {
  mockParameters.clear();
  mockMeterFrame = null;
}

// Mock hook implementations
export function useParameter(id: number): [number, (value: number) => void] {
  const value = mockParameters.get(id) ?? 0;
  const setValue = (newValue: number) => mockParameters.set(id, newValue);
  return [value, setValue];
}

export function useMeter(): MeterFrame | null {
  return mockMeterFrame;
}
```

### Global Setup

```typescript
// ui/src/test/setup.ts
import '@testing-library/jest-dom';
import { resetMocks } from './mocks/ipc';

// Reset mock state before each test
beforeEach(() => {
  resetMocks();
});
```

### Mock Injection

Use Vitest's module mocking to inject mocks:

```typescript
// In test files
vi.mock('../lib/ipc', () => import('./test/mocks/ipc'));
```

Or configure globally in `vitest.config.ts`:

```typescript
test: {
  alias: {
    '@/lib/ipc': './src/test/mocks/ipc.ts',
  },
}
```

---

## Example Test Patterns

### Component Test (ParameterSlider)

```typescript
// ui/src/components/ParameterSlider.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, beforeEach } from 'vitest';
import { setMockParameter } from '../test/mocks/ipc';
import { ParameterSlider } from './ParameterSlider';

describe('ParameterSlider', () => {
  beforeEach(() => {
    setMockParameter(1, 0.5); // Initialize parameter
  });

  it('renders with current value', () => {
    render(<ParameterSlider parameterId={1} label="Volume" />);
    expect(screen.getByRole('slider')).toHaveValue('0.5');
  });

  it('displays label', () => {
    render(<ParameterSlider parameterId={1} label="Volume" />);
    expect(screen.getByText('Volume')).toBeInTheDocument();
  });

  it('updates value on change', async () => {
    render(<ParameterSlider parameterId={1} label="Volume" />);
    const slider = screen.getByRole('slider');
    
    fireEvent.change(slider, { target: { value: '0.8' } });
    
    expect(slider).toHaveValue('0.8');
  });
});
```

### Component Test (Meter)

```typescript
// ui/src/components/Meter.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setMockMeterFrame } from '../test/mocks/ipc';
import { Meter } from './Meter';

describe('Meter', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('displays peak level', () => {
    setMockMeterFrame({ peakL: 0.8, peakR: 0.8, rmsL: 0.5, rmsR: 0.5 });
    render(<Meter />);
    
    vi.advanceTimersByTime(50); // Trigger meter poll
    
    expect(screen.getByTestId('peak-left')).toHaveStyle({ height: '80%' });
  });

  it('shows clip indicator when peak exceeds threshold', () => {
    setMockMeterFrame({ peakL: 1.0, peakR: 1.0, rmsL: 0.9, rmsR: 0.9 });
    render(<Meter />);
    
    vi.advanceTimersByTime(50);
    
    expect(screen.getByTestId('clip-indicator')).toBeVisible();
  });
});
```

### Pure Function Test

```typescript
// ui/src/lib/audio-math.test.ts
import { describe, it, expect } from 'vitest';
import { linearToDb, dbToLinear } from './audio-math';

describe('linearToDb', () => {
  it('converts 1.0 to 0 dB', () => {
    expect(linearToDb(1.0)).toBeCloseTo(0, 2);
  });

  it('converts 0.5 to approximately -6 dB', () => {
    expect(linearToDb(0.5)).toBeCloseTo(-6.02, 1);
  });

  it('returns -Infinity for 0', () => {
    expect(linearToDb(0)).toBe(-Infinity);
  });
});

describe('dbToLinear', () => {
  it('converts 0 dB to 1.0', () => {
    expect(dbToLinear(0)).toBeCloseTo(1.0, 5);
  });

  it('converts -6 dB to approximately 0.5', () => {
    expect(dbToLinear(-6)).toBeCloseTo(0.501, 2);
  });
});
```

---

## xtask Integration

### Command Structure

Extend `cargo xtask` with a `test` subcommand:

```
cargo xtask test           # Run all tests (engine + UI)
cargo xtask test --ui      # Run UI tests only
cargo xtask test --engine  # Run engine tests only
```

### Implementation

```rust
// engine/xtask/src/main.rs

#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    Test(TestArgs),
}

#[derive(Args)]
struct TestArgs {
    /// Run only UI tests
    #[arg(long)]
    ui: bool,
    
    /// Run only engine tests
    #[arg(long)]
    engine: bool,
}

fn cmd_test(args: TestArgs) -> Result<()> {
    let run_ui = args.ui || (!args.ui && !args.engine);
    let run_engine = args.engine || (!args.ui && !args.engine);
    
    if run_engine {
        println!("Running engine tests...");
        cmd!("cargo", "test", "--workspace")
            .dir(engine_dir())
            .run()?;
    }
    
    if run_ui {
        println!("Running UI tests...");
        cmd!("npm", "test")
            .dir(ui_dir())
            .run()?;
    }
    
    Ok(())
}
```

---

## CI Integration

### GitHub Actions Workflow

Add UI test step to existing workflow or create new one:

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: ui/package-lock.json
      
      - name: Install UI dependencies
        run: npm ci
        working-directory: ui
      
      - name: Run UI tests
        run: npm test
        working-directory: ui
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run engine tests
        run: cargo test --workspace
        working-directory: engine
```

---

## Package.json Scripts

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "test:ui": "vitest --ui"
  }
}
```

---

## TypeScript Configuration

Update `tsconfig.json` to include Vitest types:

```json
{
  "compilerOptions": {
    "types": ["vitest/globals", "@testing-library/jest-dom"]
  }
}
```

---

## Dependencies to Install

```bash
npm install -D vitest happy-dom @testing-library/react @testing-library/jest-dom @testing-library/user-event
```

| Package | Purpose |
|---------|---------|
| `vitest` | Test runner |
| `happy-dom` | Fast DOM environment |
| `@testing-library/react` | Component testing utilities |
| `@testing-library/jest-dom` | DOM assertion matchers |
| `@testing-library/user-event` | User interaction simulation |

---

## Constraints & Considerations

### Real-Time Safety
- Tests run in Node.js, not in the plugin context
- No real audio processing occurs during tests
- Mock data is static; tests don't validate real-time behavior

### Performance
- Target: < 10 seconds for initial test suite
- happy-dom chosen specifically for speed
- Parallel test execution enabled by default in Vitest

### Coverage
- No mandatory coverage thresholds initially
- Coverage reports available via `npm run test:coverage`
- Can add thresholds later as test suite matures

---

## Open Questions

1. **Snapshot testing?** — Deferred. Visual regression testing is planned for M6 with Playwright.
2. **E2E with real engine?** — Out of scope. M6 (Browser Testing) addresses this with WebSocket bridge.
3. **Test data fixtures?** — Start simple with inline test data; extract fixtures if patterns emerge.
