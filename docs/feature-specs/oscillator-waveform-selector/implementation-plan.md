# Implementation Plan: Oscillator Waveform Selector

## Overview

Extend the Oscillator processor to support selectable waveform types (Sine, Square, Saw, Triangle). This requires adding a new `ParamRange::Enum` variant to the DSP type system, threading enum variant labels through the protocol layer, and building a `ParameterSelect` UI component for enum parameters. The wire format stays numeric (for DAW host compatibility), while DSP code uses a real Rust enum and the UI displays human-readable variant labels.

## Requirements

- Users can select a waveform type (Sine, Square, Saw, Triangle) for the Oscillator
- The UI displays meaningful variant labels (not opaque integer values)
- DAW hosts continue to see numeric parameters (VST3/CLAP automation compatibility)
- The solution is generic — any future processor can define enum parameters using the same pattern
- Existing parameters (enabled, frequency, level) remain unchanged

## Architecture Changes

| Layer           | File                                                     | Change                                                   |
| --------------- | -------------------------------------------------------- | -------------------------------------------------------- |
| DSP types       | `engine/crates/wavecraft-dsp/src/traits.rs`              | Add `ParamRange::Enum { variants }` variant              |
| Protocol        | `engine/crates/wavecraft-protocol/src/ipc.rs`            | Add `variants: Option<Vec<String>>` to `ParameterInfo`   |
| nih-plug bridge | `engine/crates/wavecraft-nih_plug/src/lib.rs`            | Handle `ParamRange::Enum` in `param_spec_to_info()`      |
| Plugin macro    | `engine/crates/wavecraft-macros/src/plugin.rs`           | Handle `ParamRange::Enum` → `FloatRange::Linear` mapping |
| Derive macro    | `engine/crates/wavecraft-macros/src/processor_params.rs` | Support `#[param(variants = ...)]` attribute syntax      |
| Oscillator      | `engine/crates/wavecraft-processors/src/oscillator.rs`   | Add `Waveform` enum, implement all 4 waveforms           |
| TS types        | `ui/packages/core/src/types/parameters.ts`               | Add `variants?: string[]` to `ParameterInfo`             |
| UI component    | `ui/packages/components/src/ParameterSelect.tsx`         | New dropdown component for enum parameters               |
| UI auto-render  | `ui/packages/components/src/ParameterGroup.tsx`          | Add `enum` → `ParameterSelect` rendering branch          |
| Oscillator UI   | `ui/packages/components/src/OscillatorControl.tsx`       | Bind waveform selector                                   |

## Implementation Steps

### Phase 1: DSP + Protocol Type System

> **Goal:** Establish `ParamRange::Enum` as a first-class parameter type across Rust layers.
> **Risk:** Low — additive changes only, no existing behavior modified.

#### Step 1.1 — Add `ParamRange::Enum` variant

**File:** `engine/crates/wavecraft-dsp/src/traits.rs`

**Action:** Add a new variant to the `ParamRange` enum:

```rust
pub enum ParamRange {
    Linear { min: f64, max: f64 },
    Skewed { min: f64, max: f64, factor: f64 },
    Stepped { min: i32, max: i32 },
    /// Enumerated parameter with named variants.
    /// Index 0 corresponds to the first variant, 1 to the second, etc.
    Enum { variants: &'static [&'static str] },
}
```

**Why:** This is the foundation — all other layers derive behavior from `ParamRange`. Using `&'static [&'static str]` allows const/static initialization (required by `param_specs()` returning `&'static [ParamSpec]`).

**Dependencies:** None — this is the root change.

#### Step 1.2 — Add `variants` field to protocol `ParameterInfo`

**File:** `engine/crates/wavecraft-protocol/src/ipc.rs`

**Action:** Add an optional `variants` field to the `ParameterInfo` struct:

```rust
pub struct ParameterInfo {
    // ... existing fields ...

    /// Variant labels for enum parameters (e.g., ["Sine", "Square", "Saw", "Triangle"]).
    /// Only present when `param_type` is `Enum`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<String>>,
}
```

**Why:** Carries human-readable labels from Rust to the UI over IPC. `Option<Vec<String>>` keeps the JSON payload lean for non-enum parameters (omitted via `skip_serializing_if`).

**Dependencies:** None.

#### Step 1.3 — Bridge `ParamRange::Enum` in `param_spec_to_info()`

**File:** `engine/crates/wavecraft-nih_plug/src/lib.rs` (inside `__internal` module)

**Action:** Update the `param_spec_to_info()` function to handle the new variant. Currently the match is:

```rust
let (min, max, param_type) = match spec.range { ... };
```

This needs to become a tuple of `(min, max, param_type, variants)`:

```rust
let (min, max, param_type, variants) = match spec.range {
    ParamRange::Linear { min, max } => (min as f32, max as f32, ParameterType::Float, None),
    ParamRange::Skewed { min, max, .. } => (min as f32, max as f32, ParameterType::Float, None),
    ParamRange::Stepped { min, max } => {
        let param_type = if min == 0 && max == 1 {
            ParameterType::Bool
        } else {
            ParameterType::Enum
        };
        (min as f32, max as f32, param_type, None)
    }
    ParamRange::Enum { variants } => {
        let max = (variants.len() as f32) - 1.0;
        let variant_labels = variants.iter().map(|v| v.to_string()).collect();
        (0.0, max, ParameterType::Enum, Some(variant_labels))
    }
};
```

And set `variants` in the returned `ParameterInfo`:

```rust
ParameterInfo {
    // ... existing fields ...
    variants,
}
```

**Why:** `min` and `max` are derived from the variant count (0-based indexing). This keeps the wire format numeric while enriching it with labels.

**Dependencies:** Steps 1.1 and 1.2.

#### Step 1.4 — Handle `ParamRange::Enum` in `wavecraft_plugin!` macro

**File:** `engine/crates/wavecraft-macros/src/plugin.rs` (around line 376)

**Action:** Add a match arm in the `from_processor_specs()` method where `ParamRange` is matched to `FloatRange`:

```rust
ParamRange::Enum { variants } => {
    let len = variants.len() as f32;
    #krate::__nih::FloatRange::Linear {
        min: 0.0,
        max: len - 1.0,
    }
}
```

**Why:** DAW hosts see enum parameters as a `FloatParam` with range `0.0..=(N-1)` — same as the existing `Stepped` mapping but with the count derived from variants.

**Dependencies:** Step 1.1.

#### Step 1.5 — Support `#[param(variants = ...)]` in derive macro

**File:** `engine/crates/wavecraft-macros/src/processor_params.rs`

**Action:** Extend the `parse_param_attr()` function to accept a `variants` key. When present, it generates `ParamRange::Enum { variants: &[...] }` instead of requiring `range`.

Add parsing for the new key in the match block:

```rust
"variants" => {
    // Parse variants = "Sine, Square, Saw, Triangle"
    let value: Expr = meta.value()?.parse()?;
    if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = value {
        variants = Some(lit_str.value()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>());
    } else {
        return Err(meta.error("Expected string literal for variants"));
    }
}
```

When `variants` is `Some`, generate the range tokens:

```rust
if let Some(ref variant_list) = variants {
    let variant_strs = variant_list.iter().map(|v| v.as_str());
    range_tokens = quote! {
        ::wavecraft::ParamRange::Enum {
            variants: &[#(#variant_strs),*],
        }
    };
    default_val = default.unwrap_or(0.0);
}
```

Update validation: when `variants` is present, `range` should be **not** required (they are mutually exclusive).

**Why:** Enables ergonomic parameter declaration:

```rust
#[derive(ProcessorParams, Default)]
struct OscillatorParams {
    #[param(variants = "Sine, Square, Saw, Triangle", default = 0)]
    waveform: f32,
    // ...
}
```

**Dependencies:** Step 1.1.

---

### Phase 2: Oscillator Implementation

> **Goal:** Add `Waveform` enum and implement all 4 waveform generation algorithms.
> **Risk:** Low — self-contained in one file.

#### Step 2.1 — Define `Waveform` enum

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** Add the following above `OscillatorParams`:

```rust
/// Available oscillator waveform shapes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Waveform {
    #[default]
    Sine,
    Square,
    Saw,
    Triangle,
}

impl Waveform {
    /// Variant labels in declaration order (must match enum discriminant order).
    pub const VARIANTS: &'static [&'static str] = &["Sine", "Square", "Saw", "Triangle"];

    /// Convert a 0-based index to a `Waveform`.
    /// Out-of-range values default to `Sine`.
    pub fn from_index(index: f32) -> Self {
        match index.round() as u32 {
            0 => Self::Sine,
            1 => Self::Square,
            2 => Self::Saw,
            3 => Self::Triangle,
            _ => Self::Sine,
        }
    }
}
```

**Why:** Real Rust enum for type safety in DSP code. The `VARIANTS` const is referenced by `ParamRange::Enum { variants: Waveform::VARIANTS }`. The `from_index()` method converts the numeric wire value back to the enum.

**Dependencies:** None.

#### Step 2.2 — Add `waveform` field to `OscillatorParams`

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** Add `waveform: f32` to `OscillatorParams`. Since `ProcessorParams` is manually implemented here (not via derive macro), add the ParamSpec entry:

```rust
pub struct OscillatorParams {
    pub enabled: bool,
    pub waveform: f32,      // ← NEW: stored as f32 (index), converted to Waveform in process()
    pub frequency: f32,
    pub level: f32,
}
```

Add default:

```rust
impl Default for OscillatorParams {
    fn default() -> Self {
        Self {
            enabled: false,
            waveform: 0.0,  // Sine
            frequency: 440.0,
            level: 0.5,
        }
    }
}
```

Add the ParamSpec (insert as second entry in the SPECS array, between `enabled` and `frequency`):

```rust
ParamSpec {
    name: "Waveform",
    id_suffix: "waveform",
    range: ParamRange::Enum { variants: Waveform::VARIANTS },
    default: 0.0,
    unit: "",
    group: None,
},
```

Update param count from 3 to 4 in the static array.

**Why:** Placing `waveform` between `enabled` and `frequency` keeps a logical ordering (on/off → what shape → what frequency → how loud). The field is `f32` because the parameter system uses numeric values; conversion to `Waveform` enum happens at usage site.

**Dependencies:** Steps 1.1 and 2.1.

#### Step 2.3 — Implement waveform generation algorithms

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** In the `process()` method, replace the sine-only generation with waveform-aware logic:

```rust
fn process(
    &mut self,
    buffer: &mut [&mut [f32]],
    _transport: &Transport,
    params: &Self::Params,
) {
    if !params.enabled {
        for channel in buffer.iter_mut() {
            channel.fill(0.0);
        }
        return;
    }

    if self.sample_rate == 0.0 {
        return;
    }

    let waveform = Waveform::from_index(params.waveform);
    let phase_delta = params.frequency / self.sample_rate;
    let start_phase = self.phase;

    for channel in buffer.iter_mut() {
        self.phase = start_phase;
        for sample in channel.iter_mut() {
            *sample = generate_sample(waveform, self.phase) * params.level;

            self.phase += phase_delta;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}
```

Add a helper function (outside the impl block):

```rust
/// Generate a single sample for the given waveform at the given phase (0.0–1.0).
fn generate_sample(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (phase * std::f32::consts::TAU).sin(),
        Waveform::Square => {
            if phase < 0.5 { 1.0 } else { -1.0 }
        }
        Waveform::Saw => 2.0 * phase - 1.0,
        Waveform::Triangle => {
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                -4.0 * phase + 3.0
            }
        }
    }
}
```

**Waveform definitions** (phase 0.0 → 1.0):

| Waveform | Algorithm                      | Range   |
| -------- | ------------------------------ | ------- |
| Sine     | `sin(phase × 2π)`              | [-1, 1] |
| Square   | `< 0.5 → 1.0, else → -1.0`     | {-1, 1} |
| Saw      | `2 × phase - 1`                | [-1, 1) |
| Triangle | piecewise linear, peak at 0.25 | [-1, 1] |

**Why:** Extracting `generate_sample()` as a pure function makes it easy to test each waveform in isolation. All waveforms share the same phase accumulator.

**Dependencies:** Steps 2.1 and 2.2.

#### Step 2.4 — Update `from_param_defaults()`

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** The `from_param_defaults()` implementation currently returns `Self::default()`. Since we added the `waveform` field with a matching default (0.0), no change is needed — `Self::default()` already covers it. Verify this is correct.

**Dependencies:** Step 2.2.

---

### Phase 3: UI Layer

> **Goal:** Add the `variants` field to TS types, build a `ParameterSelect` component, wire it into OscillatorControl, and update ParameterGroup for auto-rendering.
> **Risk:** Low — additive UI changes.

#### Step 3.1 — Add `variants` to TypeScript `ParameterInfo`

**File:** `ui/packages/core/src/types/parameters.ts`

**Action:** Add the field to the `ParameterInfo` interface:

```typescript
export interface ParameterInfo {
  id: ParameterId;
  name: string;
  type: ParameterType;
  value: ParameterValue;
  default: ParameterValue;
  min: number;
  max: number;
  unit?: string;
  group?: string;
  /** Variant labels for enum parameters (e.g., ["Sine", "Square", "Saw", "Triangle"]). */
  variants?: string[];
}
```

**Why:** Mirrors the Rust `ParameterInfo.variants` field. Optional because only enum parameters carry it.

**Dependencies:** None.

#### Step 3.2 — Create `ParameterSelect` component

**File:** `ui/packages/components/src/ParameterSelect.tsx` (new file)

**Action:** Create a dropdown component for enum parameters:

```tsx
/**
 * ParameterSelect — dropdown selector for enum parameters.
 */

import React from 'react';
import { useParameter } from '@wavecraft/core';
import type { ParameterId } from '@wavecraft/core';

export interface ParameterSelectProps {
  /** Parameter ID to bind to */
  id: ParameterId;
}

export function ParameterSelect({
  id
}: Readonly<ParameterSelectProps>): React.JSX.Element {
  const { param, setValue, isLoading, error } = useParameter(id);

  if (isLoading || !param) {
    return <div className="h-8 animate-pulse rounded bg-plugin-dark" />;
  }

  if (error) {
    return <p className="text-sm text-red-400">Error: {error.message}</p>;
  }

  const variants = param.variants ?? [];
  const currentIndex = typeof param.value === 'number' ? param.value : 0;

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    setValue(Number(e.target.value)).catch(() => {
      // Error is surfaced via the hook's error state
    });
  };

  return (
    <div className="flex items-center justify-between gap-3 py-1">
      <label className="text-sm text-gray-300">{param.name}</label>
      <select
        className="rounded border border-plugin-border bg-plugin-dark px-2 py-1 text-sm text-gray-200 outline-none focus:border-accent"
        value={currentIndex}
        onChange={handleChange}
      >
        {variants.map((label, index) => (
          <option key={label} value={index}>
            {label}
          </option>
        ))}
      </select>
    </div>
  );
}
```

**Why:** Follows the same pattern as `ParameterSlider` and `ParameterToggle` — binds to a parameter ID via `useParameter()`, sends numeric values over IPC. The conversion index↔label happens entirely in this component.

**Dependencies:** Step 3.1.

#### Step 3.3 — Export `ParameterSelect` from components package

**File:** `ui/packages/components/src/index.ts`

**Action:** Add the export:

```typescript
export { ParameterSelect } from './ParameterSelect';
export type { ParameterSelectProps } from './ParameterSelect';
```

**Dependencies:** Step 3.2.

#### Step 3.4 — Update `ParameterGroup` to auto-render enum parameters

**File:** `ui/packages/components/src/ParameterGroup.tsx`

**Action:** Add import for `ParameterSelect` and add an `enum` branch in the rendering logic:

```tsx
import { ParameterSelect } from './ParameterSelect';

// In the map callback:
{
  group.parameters.map((param) =>
    param.type === 'bool' ? (
      <ParameterToggle key={param.id} id={param.id} />
    ) : param.type === 'enum' ? (
      <ParameterSelect key={param.id} id={param.id} />
    ) : (
      <ParameterSlider key={param.id} id={param.id} />
    )
  );
}
```

**Why:** ParameterGroup is the auto-rendering pipeline. With this change, any processor that declares an enum parameter will automatically get a dropdown in the UI without writing custom components.

**Dependencies:** Steps 3.2 and 3.3.

#### Step 3.5 — Wire waveform selector into `OscillatorControl`

**File:** `ui/packages/components/src/OscillatorControl.tsx`

**Action:** Add a constant for the waveform parameter ID and render the selector:

```tsx
import { ParameterSelect } from './ParameterSelect';

const OSCILLATOR_WAVEFORM_PARAM_ID = 'oscillator_waveform' as ParameterId;
```

Add the `ParameterSelect` in the controls section (after the toggle, before the sliders):

```tsx
<div className="mt-3">
  <ParameterSelect id={OSCILLATOR_WAVEFORM_PARAM_ID} />
  <ParameterSlider id={OSCILLATOR_FREQUENCY_PARAM_ID} />
  <ParameterSlider id={OSCILLATOR_LEVEL_PARAM_ID} />
</div>
```

**Why:** Places the waveform selector logically before frequency/level — you choose the shape first, then tune it.

**Dependencies:** Steps 3.2 and 3.3.

---

### Phase 4: Codegen Verification

> **Goal:** Confirm TypeScript codegen handles `ParamRange::Enum` correctly.
> **Risk:** Very low — existing code likely already works.

#### Step 4.1 — Verify TS codegen maps `Enum` → `number`

**File:** `cli/src/project/ts_codegen.rs`

**Action:** Review `ts_value_type_for_param()` (line ~66):

```rust
fn ts_value_type_for_param(param_type: ParameterType) -> &'static str {
    match param_type {
        ParameterType::Bool => "boolean",
        ParameterType::Float | ParameterType::Enum => "number",
    }
}
```

This already maps `Enum` → `"number"`. **No code change needed**, but verify by running `wavecraft start` after engine changes and checking the generated `parameters.ts` includes `oscillator_waveform: number`.

**Dependencies:** Phases 1 and 2 complete.

---

### Phase 5: Tests

> **Goal:** Comprehensive test coverage for new waveform generation and UI component.
> **Risk:** Low.

#### Step 5.1 — Unit tests for `Waveform::from_index()`

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** Add tests in the existing `mod tests`:

```rust
#[test]
fn waveform_from_index_maps_correctly() {
    assert_eq!(Waveform::from_index(0.0), Waveform::Sine);
    assert_eq!(Waveform::from_index(1.0), Waveform::Square);
    assert_eq!(Waveform::from_index(2.0), Waveform::Saw);
    assert_eq!(Waveform::from_index(3.0), Waveform::Triangle);
}

#[test]
fn waveform_from_index_out_of_range_defaults_to_sine() {
    assert_eq!(Waveform::from_index(-1.0), Waveform::Sine);
    assert_eq!(Waveform::from_index(4.0), Waveform::Sine);
    assert_eq!(Waveform::from_index(100.0), Waveform::Sine);
}

#[test]
fn waveform_from_index_rounds_floats() {
    assert_eq!(Waveform::from_index(0.4), Waveform::Sine);
    assert_eq!(Waveform::from_index(0.6), Waveform::Square);
    assert_eq!(Waveform::from_index(1.5), Waveform::Saw);
    assert_eq!(Waveform::from_index(2.7), Waveform::Triangle);
}
```

**Dependencies:** Step 2.1.

#### Step 5.2 — Unit tests for `generate_sample()`

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** Add tests for each waveform shape:

```rust
#[test]
fn sine_wave_zero_crossing_and_peak() {
    assert!((generate_sample(Waveform::Sine, 0.0)).abs() < 1e-5);      // zero at phase 0
    assert!((generate_sample(Waveform::Sine, 0.25) - 1.0).abs() < 1e-5); // peak at π/2
    assert!((generate_sample(Waveform::Sine, 0.5)).abs() < 1e-5);      // zero at π
    assert!((generate_sample(Waveform::Sine, 0.75) + 1.0).abs() < 1e-5); // trough at 3π/2
}

#[test]
fn square_wave_values() {
    assert_eq!(generate_sample(Waveform::Square, 0.0), 1.0);
    assert_eq!(generate_sample(Waveform::Square, 0.25), 1.0);
    assert_eq!(generate_sample(Waveform::Square, 0.5), -1.0);
    assert_eq!(generate_sample(Waveform::Square, 0.75), -1.0);
}

#[test]
fn saw_wave_values() {
    assert!((generate_sample(Waveform::Saw, 0.0) + 1.0).abs() < 1e-5);  // starts at -1
    assert!((generate_sample(Waveform::Saw, 0.5)).abs() < 1e-5);         // zero at midpoint
    assert!((generate_sample(Waveform::Saw, 1.0) - 1.0).abs() < 1e-5);  // ends at +1
}

#[test]
fn triangle_wave_values() {
    assert!((generate_sample(Waveform::Triangle, 0.0) + 1.0).abs() < 1e-5);  // starts at -1
    assert!((generate_sample(Waveform::Triangle, 0.25)).abs() < 1e-5);        // zero crossing
    assert!((generate_sample(Waveform::Triangle, 0.5) - 1.0).abs() < 1e-5);  // peak at +1
    assert!((generate_sample(Waveform::Triangle, 0.75)).abs() < 1e-5);        // zero crossing
}
```

**Dependencies:** Step 2.3.

#### Step 5.3 — Integration test: oscillator produces signal for each waveform

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** Update the existing `test_params()` helper and add a parameterized test:

```rust
fn test_params_with_waveform(enabled: bool, waveform: f32) -> OscillatorParams {
    OscillatorParams {
        enabled,
        waveform,
        frequency: 440.0,
        level: 0.5,
    }
}

#[test]
fn all_waveforms_produce_signal_when_enabled() {
    for waveform_index in 0..4 {
        let mut osc = Oscillator::default();
        osc.set_sample_rate(48_000.0);

        let mut left = [0.0_f32; 128];
        let mut right = [0.0_f32; 128];
        let mut buffer = [&mut left[..], &mut right[..]];

        osc.process(
            &mut buffer,
            &Transport::default(),
            &test_params_with_waveform(true, waveform_index as f32),
        );

        let peak = left.iter().fold(0.0_f32, |acc, s| acc.max(s.abs()));
        assert!(peak > 0.01, "waveform index {waveform_index} should produce signal");
    }
}
```

**Dependencies:** Steps 2.2 and 2.3.

#### Step 5.4 — Update existing oscillator tests

**File:** `engine/crates/wavecraft-processors/src/oscillator.rs`

**Action:** Update `test_params()` helper to include the new `waveform` field:

```rust
fn test_params(enabled: bool) -> OscillatorParams {
    OscillatorParams {
        enabled,
        waveform: 0.0,  // Sine (default)
        frequency: 440.0,
        level: 0.5,
    }
}
```

**Why:** Existing tests for silence-when-disabled and signal-when-enabled must continue to compile and pass.

**Dependencies:** Step 2.2.

#### Step 5.5 — UI component tests for `ParameterSelect`

**File:** `ui/packages/components/src/__tests__/ParameterSelect.test.tsx` (new file)

**Action:** Write Vitest + React Testing Library tests:

- Renders dropdown with variant labels
- Displays current value as selected option
- Calls `setValue()` with numeric index on change
- Shows loading skeleton when `isLoading` is true
- Shows error message when `error` is present

Follow the existing test patterns from `ParameterSlider.test.tsx` and `ParameterToggle.test.tsx`.

**Dependencies:** Steps 3.1 and 3.2.

#### Step 5.6 — Protocol serialization test for `variants`

**File:** `engine/crates/wavecraft-protocol/src/ipc.rs` (or a new test file in that crate)

**Action:** Add a round-trip serialization test:

```rust
#[test]
fn parameter_info_with_variants_serializes_correctly() {
    let info = ParameterInfo {
        id: "osc_waveform".to_string(),
        name: "Waveform".to_string(),
        param_type: ParameterType::Enum,
        value: 0.0,
        default: 0.0,
        min: 0.0,
        max: 3.0,
        unit: None,
        group: None,
        variants: Some(vec![
            "Sine".to_string(),
            "Square".to_string(),
            "Saw".to_string(),
            "Triangle".to_string(),
        ]),
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"variants\""));

    let deserialized: ParameterInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.variants.unwrap().len(), 4);
}

#[test]
fn parameter_info_without_variants_omits_field() {
    let info = ParameterInfo {
        id: "gain".to_string(),
        name: "Gain".to_string(),
        param_type: ParameterType::Float,
        value: 0.5,
        default: 0.5,
        min: 0.0,
        max: 1.0,
        unit: Some("dB".to_string()),
        group: None,
        variants: None,
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(!json.contains("\"variants\""));
}
```

**Why:** Ensures backward compatibility — existing ParameterInfo payloads (without variants) produce identical JSON.

**Dependencies:** Step 1.2.

---

## Testing Strategy

| Category           | What                                        | Where                          |
| ------------------ | ------------------------------------------- | ------------------------------ |
| Unit (Rust)        | `Waveform::from_index()` edge cases         | `oscillator.rs`                |
| Unit (Rust)        | `generate_sample()` per waveform            | `oscillator.rs`                |
| Unit (Rust)        | Protocol serialization round-trip           | `ipc.rs`                       |
| Integration (Rust) | All waveforms produce signal                | `oscillator.rs`                |
| Unit (TS)          | `ParameterSelect` component rendering       | `ParameterSelect.test.tsx`     |
| E2E                | Waveform selector visible, changes waveform | Visual test via Playwright MCP |
| Regression         | Existing oscillator tests still pass        | `oscillator.rs`                |
| CI                 | `cargo xtask ci-check` passes               | All                            |

## Risks & Mitigations

- **Risk:** Derive macro changes break existing `#[param(...)]` usage in sdk-template
  - **Mitigation:** `variants` is an additive key — existing attrs (using `range`) are unaffected. Run `cargo xtask ci-check --full` to validate template compilation.

- **Risk:** DAW host shows raw `0, 1, 2, 3` instead of labels
  - **Mitigation:** Expected for Phase 1. nih-plug `FloatParam` doesn't natively map to stepped labels. Future enhancement: use `.with_value_to_string()` on the `FloatParam` in the macro to show labels in DAW UI. Out of scope for this plan.

- **Risk:** Wire format change breaks existing UI clients
  - **Mitigation:** `variants` is `Option` + `skip_serializing_if = "Option::is_none"`. Existing parameters produce identical JSON. Backward compatible.

- **Risk:** `from_index()` on negative/NaN float values
  - **Mitigation:** `round() as u32` handles this — negative rounds to 0 or wraps, non-matching values default to Sine. Add explicit test for edge cases.

## Success Criteria

- [ ] `ParamRange::Enum { variants }` exists in DSP layer and compiles
- [ ] `ParameterInfo.variants` is populated for enum parameters in IPC responses
- [ ] Oscillator generates correct Sine, Square, Saw, and Triangle waveforms
- [ ] UI shows a dropdown for waveform selection with labeled options
- [ ] Changing the dropdown sends the correct numeric value over IPC
- [ ] `ParameterGroup` auto-renders enum parameters as dropdowns
- [ ] All existing tests pass (no regressions)
- [ ] `cargo xtask ci-check` passes
- [ ] Generated plugin (via `wavecraft create`) compiles with new enum support

## Estimated Effort

| Phase   | Scope                      | Complexity | Estimate                                          |
| ------- | -------------------------- | ---------- | ------------------------------------------------- |
| Phase 1 | DSP + Protocol type system | Medium     | Core of the work — 5 files, careful API threading |
| Phase 2 | Oscillator implementation  | Low        | 1 file, straightforward DSP                       |
| Phase 3 | UI layer                   | Low        | 1 new component + minor edits to 3 files          |
| Phase 4 | Codegen verification       | Trivial    | Verification only, no changes expected            |
| Phase 5 | Tests                      | Low        | ~8 test functions across Rust and TS              |

## Dependency Graph

```
Phase 1.1 (ParamRange::Enum)
├── Phase 1.2 (ParameterInfo.variants)
│   └── Phase 1.3 (param_spec_to_info bridge)
├── Phase 1.4 (plugin macro)
├── Phase 1.5 (derive macro)
└── Phase 2.1 (Waveform enum)
    └── Phase 2.2 (OscillatorParams)
        └── Phase 2.3 (waveform generation)
            └── Phase 5.1–5.4 (Rust tests)

Phase 3.1 (TS types)
└── Phase 3.2 (ParameterSelect)
    ├── Phase 3.3 (export)
    │   ├── Phase 3.4 (ParameterGroup)
    │   └── Phase 3.5 (OscillatorControl)
    └── Phase 5.5 (UI tests)

Phase 4.1 (codegen verification) — after Phases 1+2

Phase 5.6 (protocol tests) — after Phase 1.2
```
