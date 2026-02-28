# Fan-out Inventory — ui-ux-refactor (Phase 4.1)

## Prior hook-coupled presentational components (migrated)

- `ui/packages/components/src/ParameterSlider.tsx`
- `ui/packages/components/src/ParameterSelect.tsx`
- `ui/packages/components/src/ParameterToggle.tsx`
- `ui/packages/components/src/Processor.tsx`
- `ui/packages/components/src/ParameterGroup.tsx`
- `ui/packages/components/src/Meter.tsx`
- `ui/packages/components/src/ConnectionStatus.tsx`
- `ui/packages/components/src/LatencyMonitor.tsx`
- `ui/packages/components/src/OscilloscopeProcessor.tsx`
- `ui/packages/components/src/ResizeHandle.tsx`
- `ui/packages/components/src/ResizeControls.tsx`

## Smart-container ownership introduced in sdk-template

- `sdk-template/ui/src/App.tsx` owns app-level hooks for:
  - connection status
  - audio status
  - latency monitor
  - meter frame
  - resize requests
- `sdk-template/ui/src/processors/SmartProcessor.tsx` owns processor parameter subscriptions and mutations.
- `sdk-template/ui/src/processors/OscilloscopeProcessor.tsx` owns oscilloscope subscriptions and visibility.

## Result

- Presentational components in `@wavecraft/components` now render from props and no longer import `@wavecraft/core` in runtime source files.
- Parameter subscription fan-out is consolidated into smart containers in `sdk-template/ui`.
