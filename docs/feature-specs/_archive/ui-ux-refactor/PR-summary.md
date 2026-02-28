## Summary

This PR adds and validates knob double-click reset behavior while landing the broader `ui-ux-refactor` branch work detected in the merge-base analysis. The diff shows substantial UI/component modernization, processor component updates, type-safety improvements, and supporting docs/test updates across the monorepo.

## Changes

- **UI**: Added/updated `Knob` behavior and tests (`ui/packages/components/src/Knob.tsx`, `ui/packages/components/src/Knob.test.tsx`), expanded processor/component set (e.g. `GainProcessor`, `PassthroughProcessor`, `ToneFilterProcessor`, `ProcessorCard`), and refreshed `sdk-template/ui/src/App.tsx` composition and generated processor typings.
- **Build/Config**: Updated TypeScript/Vite/Tailwind and package config across UI workspaces (`ui/tsconfig.json`, `ui/packages/*/tsconfig.json`, `ui/package.json`, `sdk-template/ui/vite.config.ts`, `sdk-template/ui/tailwind.config.js`) and refreshed embedded UI dist assets.
- **Documentation**: Updated architecture/guides/feature-spec documentation, including `docs/feature-specs/ui-ux-refactor/*`, archived artifacts under `docs/feature-specs/_archive/ui-ux-refactor/`, and testing/workflow guidance.

## Commits

- `00f71f7 refactor: improve layout and styling in PassthroughProcessor and ProcessorCard components`
- `bf27a52 refactor: enhance PassthroughProcessor and useMeterSignalActivity for improved signal handling and intensity calculations`
- `2fe2051 refactor: enhance type safety for processor IDs and update useMeterSignalActivity return type`
- `907d254 refactor: rename processors.d.ts to processors.types.d.ts and update related imports; enhance signal activity handling in PassthroughProcessor`
- `76c2dda feat: add enabled parameter to TestToneProcessor for toggling tone generation`
- `d39f460 Refactor TestToneProcessor: Remove enabled parameter, update tests, and add PassthroughProcessor`
- `a40740f refactor: remove unused ProcessorId import and adjust import paths for TestToneProcessor`
- `acf6e6b refactor: add ToneFilterProcessor and associated type checks; update processor parameter IDs and enhance documentation`
- `c979421 refactor: update processor component creator documentation to require processor-parameter ID types and clarify index export process`
- `bdc80c9 refactor: enhance processor component creator documentation for clarity on parameter sourcing and user interaction`
- `11af882 refactor: add processor-component-creator skill documentation for UI component creation`
- `7645b21 refactor: update research rules across agent documentation for clarity and consistency`
- `6ecf480 refactor: update ProcessorParameter type imports and enhance type definitions; adjust tsconfig and vite config to exclude typecheck files`
- `1f25c27 refactor: enhance try-sdk command to include UI package preflight checks and update project creation steps`
- `29a0be3 refactor: add try-sdk command to generate and open a local test plugin project; update main command enum to include try-sdk`
- `726db14 refactor: introduce GainProcessor component; update App layout to include GainProcessor; enhance type safety and parameter handling in processor-related files`
- `df73daa refactor: remove unused SmartProcessor and ExampleProcessor components; clean up App layout by eliminating redundant processor instances`
- `c2c917a refactor: enhance layout and styling in App component; update SmartProcessor, TestToneProcessor, and ProcessorCard for improved class management and styling flexibility`
- `a3b32be refactor: update RadioGroup tests for improved option handling; enhance ProcessorCard styling with mergeClassNames utility`
- `a73e3d7 refactor: enhance layout and styling in App component; update SmartProcessor to improve radio group options handling; modify Fader and Meter components for better class management; add className prop to various components for improved styling flexibility`
- `811f051 refactor: enhance App layout with Row and Col components; update SmartProcessor to support radio group options; improve RadioGroup styling and functionality; adjust ProcessorCard for better content layout; enable CSS sourcemaps in Vite config`
- `b1eda34 refactor: enhance SmartProcessor and RadioGroup for better enum handling; update ProcessorCard and TestToneProcessor to improve structure and functionality`
- `e2d9fa2 refactor: enhance SmartProcessor with ErrorMessage component for error handling; add tests for ErrorMessage; update index exports`
- `702b9bc refactor: integrate SmartProcessor into App; update ExampleProcessor to use processorId; create ProcessorCard for better parameter management; enhance OscilloscopeProcessor with ProcessorCard; remove deprecated Processor component and tests`
- `a81491b refactor: update TypeScript code generation to use global augmentation for WavecraftParameterIdMap; enhance OscilloscopeProcessor with bypass functionality; implement Select component with tests; adjust tsconfig for parameter generation`
- `7099b6c refactor: consolidate processor components; enhance OscilloscopeProcessor and TestToneProcessor functionality; update tests for improved coverage`
- `3cb413a refactor: rename oscillator processor to test tone processor and update related tests`
- `4e5b7b4 refactor: enhance OscillatorProcessor layout and accessibility; update Knob component for dynamic text size; integrate clsx and tailwind-merge for class management`
- `d9c8faa refactor: update waveform icon tests to use role-based queries; enhance SVG paths for better rendering`
- `769e21d refactor: update waveform selection handling in OscillatorProcessor; enforce size type for waveform options`
- `34afa95 refactor: implement RadioGroup component; enhance accessibility and keyboard handling; update related documentation and tests`
- `28c9deb refactor: introduce Icon and IconButton components; enhance OscillatorProcessor with waveform icons; update tests for waveform icon rendering`
- `1ca8d2a refactor: enhance Knob component to manage precision cue state during drag events; improve event listener handling`
- `153cdd3 refactor: update layout of OscillatorProcessor; enhance Col and Row components for consistent sizing`
- `b270fac refactor: enhance OscillatorProcessor and SmartProcessor components; introduce Col and Row components for layout consistency; add Switch component with tests for improved functionality`
- `4f1091d refactor: enhance OscillatorProcessor and SmartProcessor components for improved layout and functionality; update parameter handling in hooks for better type safety`
- `6a131cc refactor: enhance OscillatorProcessor component with bypass state handling and update related tests for visual feedback`
- `212417c refactor: streamline Fader component for improved layout and interaction; enhance test coverage for horizontal and vertical orientations`
- `9d45bda refactor: enhance Fader and Knob components with gradient backgrounds and improved shadow effects`
- `fd64360 refactor: enhance troubleshooting section in SKILL.md for clarity and consistency`
- `3f1f6cb refactor: update Playwright MCP UI testing documentation for dev server pre-check and workflow; enhance Knob component with reserved value width handling and related tests`
- `6aeef9f refactor: replace Toggle components with Button for Bypass and Enabled parameters in OscillatorProcessor; update related tests`
- `460f19e refactor: enhance Fader component with horizontal footprint support and update related tests for orientation handling`
- `ce94f8f refactor: enhance Fader and OscillatorProcessor components with vertical orientation support and precision dragging; update related tests`
- `ae13579 refactor: add Playwright MCP UI testing skill documentation for automated visual testing`
- `5c5ee77 refactor: implement Shift precision hints in Fader, Knob, OscillatorProcessor, and SmartProcessor components; update related tests`
- `8a3fc45 refactor: update visual testing references from Playwright to VS Code Simple Browser`
- `2daf482 refactor: enhance Fader and Knob components for Shift precision dragging; implement state management for drag events`
- `81048a6 refactor: add OscillatorProcessor component and integrate into App; enhance Fader and Knob components with Shift precision hints`
- `4fedd76 refactor: enhance layout structure by adding data-testid to processor grid for improved testing`
- `35fed94 refactor: restrict getModifierState parameter to 'Shift' for improved type safety`
- `9f27e4a refactor: reorganize processor parameters to prioritize bypass parameters and enhance Knob component keyboard controls`
- `fae2b34 refactor: update Button component active state styling for improved visual clarity`
- `9decaf8 refactor: enhance Button component active state styling and remove visual indicator`
- `6d5f676 refactor: update Button component to use 'active' prop for state management and enhance accessibility attributes`
- `9d18d5c refactor: update UI/UX documentation and testing guidelines for Playwright integration`
- `94fc2d0 refactor: improve SmartProcessor parameter rendering with enhanced styles and layout adjustments`
- `8cf4cc8 refactor: enhance SmartProcessor UI with improved styles, parameter formatting, and loading/error states`
- `37a374f feat: implement SmartProcessor component with primitive controls and loading/error states`
- `e51aac5 Add VST Component Spec Sheet for React + Tailwind implementation`
- `c0c9399 feat(ui-ux-refactor): complete documentation and implementation of UI/UX improvements`
- `5116f1f refactor: enhance control visual states and add tests for disabled and error semantics in UI components`
- `3158273 refactor: enhance UI components with new styles and states, add Button, Toggle, Knob, and Fader components`
- `707ecc3 refactor: improve table formatting and alignment in VST component spec sheet`
- `0ed1a55 refactor: remove compatibility shims and legacy processor files`
- `b42e27b refactor: remove compatibility shims and update processor imports`
- `4fc9e46 refactor: remove deprecated UI compatibility shims and clean up backlog items`
- `bcb3cf3 Merge branch 'main' into feature/ui-ux-refactor`
- `80fdfeb refactor: add release hygiene section to backlog for removing deprecated UI compatibility shims`
- `4906a14 refactor: update document references to final versions in UI/UX refactor specs`
- `62fc508 refactor: replace legacy processor components with compat shim re-exports`
- `e1a3c73 feat: add output marker check for server readiness in startup smoke validation`
- `0488756 refactor: consolidate processor components into compat module and update tests`
- `79f20a8 feat: Implement final UI/UX refactor plan focusing on code minimization`
- `84cac82 feat(ui-ux-refactor): add low-level design document for UI/UX refactor focusing on code minimization`
- `fb8c136 fix: resolve optimistic rollback race condition in WavecraftProvider`
- `ecc1ab6 feat: add QA report for WavecraftProvider parameter state with findings and recommendations`
- `d5ff153 test: add comprehensive test plan for WavecraftProvider parameter state verification`
- `9cd40d1 refactor: rename useAllParametersFor to useParametersForProcessor and update related types`
- `920cb54 feat(ui-ux-refactor): add implementation and low-level design documents for WavecraftProvider parameter state`
- `190b43f docs: add PR summary for ui-ux-refactor`
- `7aaa90f chore(ui-ux-refactor): harden raw IPC literal lint guards`
- `08c860b fix(ui): close remaining ui-ux-refactor hygiene and guardrail gaps`
- `6ad9414 feat(ui-ux-refactor): Add comprehensive test plan and visual baseline documentation`
- `accadb4 fix(ui-ux-refactor): resolve QA blockers for tokens, a11y, and IPC constants`
- `761ffc8 docs(ui-ux-refactor): add phase inventories and progress snapshot`
- `c61e08f refactor(ui): complete phases 3-5 smart/presentational split`
- `3021fe9 docs(feature): add ui-ux-refactor planning artifacts`
- `be0c77b test(ui): add ui-ux-refactor phase 0.1 visual baseline`
- `47b317d feat(ui): start ui-ux-refactor phase 0/1`
- `11a0829 feat(ui): start ui-ux-refactor phase 0/1`

## Related Documentation

- [Test Plan](./test-plan.md)

## Testing

- [x] Knob unit tests pass: 26/26 in `Knob.test.tsx`
- [x] Focused processor tests pass (GainProcessor, TestToneProcessor, ToneFilterProcessor)
- [x] CI lint/tests: pre-existing branch failures confirmed unrelated to this change
- [ ] Manual visual verification via Simple Browser (in-app)

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated for new behavior
- [x] QA review: no blockers found (2 minor findings resolved)
- [x] No new lint errors introduced by this change
