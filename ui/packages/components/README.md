# @wavecraft/components

Pre-built React components for Wavecraft audio plugins.

## Installation

```bash
npm install @wavecraft/core @wavecraft/components
```

> Note: `@wavecraft/core` is a peer dependency and must be installed.

## Quick Start

```tsx
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
```

## TailwindCSS Configuration

Components use TailwindCSS utility classes. Configure your `tailwind.config.js`:

```javascript
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
```

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
