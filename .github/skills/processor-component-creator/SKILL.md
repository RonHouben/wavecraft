---
name: processor-component-creator
description: Create new processor UI components in @wavecraft/components with default scope (component, unit test, typecheck, index exports), processorId-derived IDs, and targeted validation.
---

# Processor Component Creator

Use this skill to add a new processor UI component in `@wavecraft/components` with a predictable, minimal scope.

## When to use

- Add a new processor component under `ui/packages/components`
- Add/update processor-specific unit tests and typecheck coverage
- Wire exports for component consumption
- Follow ID derivation from `processorId` without template app wiring

## Guardrails

- Never edit `docs/feature-specs/_archive/**`.
- Do not edit `docs/roadmap.md` (PO-owned).
- Always use `ask_questions` to clarify requirements with the user **before** starting implementation and **during** implementation whenever uncertainty appears.
- Default scope is only:
  1. Component file
  2. Unit test file
  3. Typecheck file
  4. Index exports
- Do **not** add default template app wiring unless explicitly requested.
- Follow:
  - `docs/architecture/coding-standards.md`
  - `docs/architecture/coding-standards-typescript.md`
  - `docs/architecture/coding-standards-testing.md`

## Clarification Protocol

- Before implementation: ask concise clarifying questions with `ask_questions` to confirm scope, naming, and expected behavior.
- During implementation: if any ambiguity or conflict appears, pause and ask follow-up clarifying questions with `ask_questions` before proceeding.

## Workflow

### 1) Discover processor IDs and params

- Identify canonical `processorId` and expected parameter/bypass IDs from generated metadata and existing patterns.
- Confirm suffixes used by the processor are valid for its `ProcessorId` type.
- If IDs look stale or missing, regenerate/build before coding to avoid chasing outdated generated IDs.

### 2) Implement component with `ProcessorCard`

- Create the processor component in `ui/packages/components/src/processors/` (or existing processor location).
- Compose UI with `ProcessorCard` and existing component primitives.
- Bind derived IDs through `useParameter` in the component (not ad-hoc state), e.g.:
  - `const { param, setValue } = useParameter<number>(`${processorId}\_level`)`
- Keep props and behavior minimal; avoid cross-package side effects.

### 3) Derive IDs from `processorId` (default strategy)

- Build IDs from the selected `processorId`, for example:
  - bypass: `${processorId}_bypass`
  - level: `${processorId}_level`
- Use typed `useParameter` calls for those derived IDs where values are read/written, e.g.:
  - level (`number`): `const { param, setValue } = useParameter<number>(`${processorId}\_level`)`
  - bypass (`boolean`): `const { param: bypass, setValue: setBypass } = useParameter<boolean>(`${processorId}\_bypass`)`
- Prefer filtered/narrowed `ProcessorId` types (or helper types) before suffixing, so invalid combinations are rejected at compile time.
- If a processor uses different canonical suffixes, codify that explicitly in typed mappings.

### 4) Update index exports

- Export the new component from the relevant local `index.ts`.
- Bubble export to package-level index only where required by existing structure.
- Keep export changes minimal and ordered consistently with nearby entries.

### 5) Add tests and typechecks

- Add/extend unit tests for rendering, interactions, callback behavior, and ID wiring.
- Add/extend typecheck assertions to ensure derived IDs and props remain type-safe.
- Cover failure-prone cases (invalid suffix/type narrowing regressions).

### 6) Validate (targeted)

- [ ] Run targeted TypeScript diagnostics on touched files.
- [ ] Run processor component tests (targeted Vitest scope).
- [ ] Run targeted lint + typecheck for affected package(s).
- [ ] Perform optional visual check in browser/dev UI for layout/state sanity.

## Done criteria

- [ ] Component implemented with `ProcessorCard`
- [ ] IDs derived from `processorId` by default strategy
- [ ] Unit tests added/updated and passing
- [ ] Typecheck file added/updated and passing
- [ ] Index exports updated and verified
- [ ] Targeted diagnostics/lint/typecheck/tests completed
- [ ] No template app wiring added by default

## Common pitfalls

- `never` type collapse after over-narrowing unions or incompatible suffix concatenation
- Invalid suffixes for a given `ProcessorId` (string interpolation compiles too loosely)
- Outdated generated IDs/metadata causing false failures
- Export added in one index but not package-level re-export path
- Tests pass while typecheck file misses unsafe ID derivation paths
