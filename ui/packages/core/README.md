# @wavecraft/core

Core SDK for Wavecraft audio plugins — IPC bridge, hooks, and utilities.

## Installation

```bash
npm install @wavecraft/core
```

## Quick Start

```tsx
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
```

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
| `useLatencyMonitor()` | Monitor IPC roundtrip latency |

### Utilities

```typescript
import { linearToDb, dbToLinear } from '@wavecraft/core/meters';

linearToDb(0.5);  // → -6.02 dB
dbToLinear(-6);   // → 0.501
```

### Advanced: IPC Bridge

For custom implementations:

```typescript
import { IpcBridge, ParameterClient } from '@wavecraft/core';
```

## Requirements

- React 18+

## License

MIT
