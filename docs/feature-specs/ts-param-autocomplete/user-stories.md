# User Stories: TypeScript Parameter ID Autocompletion

## Overview

When an SDK developer writes `useParameter('|')` in their React code, VS Code shows "No suggestions." Parameter IDs are generated at Rust compile time by the `wavecraft_plugin!` macro (e.g., `inputgain_level`, `oscillator_frequency`), but the TypeScript `useParameter` hook accepts plain `string` — offering zero discoverability.

This feature introduces build-time codegen that generates a TypeScript union type of all parameter IDs during `wavecraft start`, enabling full IDE autocompletion and type-safe parameter references.

**Approved Approach:** Build-Time Codegen via `wavecraft start`

**Design Decision:** Backward compatibility is NOT required. The generated `ParameterId` type is used directly by `@wavecraft/core` hooks, `ParameterClient`, and `@wavecraft/components` — no generic type parameters needed. This results in a simpler, more ergonomic API: `useParameter('inputgain_level')` with direct autocompletion out of the box.

---

## User Story 1: Parameter ID Autocompletion in IDE

**As a** plugin developer using the Wavecraft SDK
**I want** VS Code to suggest all available parameter IDs when I type `useParameter('`
**So that** I can discover and reference parameters without memorizing or looking up their string IDs

### Acceptance Criteria

- [ ] Typing `useParameter('` in VS Code triggers IntelliSense with all parameter IDs
- [ ] Each suggestion matches the exact parameter ID as defined in the Rust DSL (e.g., `inputgain_level`, `oscillator_frequency`)
- [ ] Autocompletion works in `useParameter`, `ParameterClient.getParameter`, `ParameterSlider`, and any other API that accepts parameter IDs
- [ ] No additional VS Code extensions required — standard TypeScript language server is sufficient

### Notes

- The generated `ParameterId` type is used directly by `@wavecraft/core` and `@wavecraft/components` — no imports or generics needed by the developer
- Works with any editor that supports TypeScript language services (VS Code, WebStorm, Neovim LSP, etc.)

---

## User Story 2: Automatic Type Generation During Development

**As a** plugin developer
**I want** the parameter type file to be generated automatically when I run `wavecraft start`
**So that** I don't need to run any extra commands or manual steps to get type safety

### Acceptance Criteria

- [ ] Running `wavecraft start` generates `ui/src/generated/parameters.ts` containing a `ParameterId` union type
- [ ] The generated file includes a "do not edit" header comment
- [ ] The file is generated before Vite starts, so types are available immediately
- [ ] If the file already exists, it is overwritten with the latest parameter IDs
- [ ] The generated file is added to `.gitignore` (it's a build artifact)

### Notes

- `cargo xtask dev` delegates to `wavecraft start`, so both commands produce the generated file
- The generated file path should follow a conventional `generated/` directory pattern

---

## User Story 3: Type Updates on Rust Source Changes (Hot-Reload)

**As a** plugin developer iterating on my signal chain
**I want** the TypeScript parameter types to update automatically when I modify my Rust processors
**So that** my IDE immediately reflects new, renamed, or removed parameters without restarting the dev server

### Acceptance Criteria

- [ ] When a Rust source file changes and triggers a hot-reload rebuild, the `parameters.ts` file is regenerated
- [ ] Vite HMR detects the file change and updates the TypeScript types in the running dev session
- [ ] Adding a new processor to the signal chain results in its parameter IDs appearing in the union type
- [ ] Removing a processor results in its parameter IDs being removed from the union type
- [ ] TypeScript compilation errors surface immediately if code references a now-removed parameter ID

### Notes

- Depends on Milestone 18.9 (Rust Hot-Reload) for the file watcher and rebuild pipeline
- The TS file regeneration rides the existing rebuild pipeline — no separate watcher needed

---

## User Story 4: Type Safety Catches Typos at Compile Time

**As a** plugin developer
**I want** TypeScript to show an error when I reference a parameter ID that doesn't exist
**So that** I catch typos and stale references before runtime instead of debugging silent failures

### Acceptance Criteria

- [ ] `useParameter('nonexistent_param')` produces a TypeScript compile error
- [ ] The error message clearly indicates the string is not assignable to the `ParameterId` type
- [ ] Error appears in the IDE (red underline) and in `tsc --noEmit` output
- [ ] Renaming a parameter in Rust and regenerating types surfaces all stale references in TypeScript

### Notes

- This is the primary value proposition — shifting parameter ID errors from runtime to compile time
- Works with `tsc --noEmit` in CI to prevent merging code with invalid parameter references
- Type safety is the default — every `useParameter()` call is type-checked automatically

---

## User Story 5: Generated File Follows Project Conventions

**As a** plugin developer
**I want** the generated types file to follow standard project conventions (location, formatting, exports)
**So that** it integrates naturally with my project structure and import patterns

### Acceptance Criteria

- [ ] Generated file is at `ui/src/generated/parameters.ts` (conventional `generated/` directory)
- [ ] File uses standard TypeScript formatting (consistent with project's Prettier config)
- [ ] Type is exported as a named export: `export type ParameterId = ...`
- [ ] Union type uses `|` syntax with one ID per line for readable diffs
- [ ] File includes a timestamp or generation notice for debugging

### Notes

- The `generated/` directory pattern is well-known in codegen workflows (GraphQL, Prisma, etc.)
- SDK template should include this directory in `.gitignore` with a comment explaining why

---

## User Story 6: SDK Template Includes Type-Safe Example

**As a** new plugin developer starting with `wavecraft create`
**I want** the generated template project to demonstrate type-safe parameter usage
**So that** I learn the recommended pattern from day one

### Acceptance Criteria

- [ ] Template's `App.tsx` (or equivalent) uses `useParameter('inputgain_level')` with direct autocompletion
- [ ] Template README mentions the generated types and how autocompletion works
- [ ] Running `wavecraft start` on a fresh template project generates the types file and IDE autocompletion works immediately
- [ ] The example is educational — includes a comment explaining that parameter IDs are auto-generated from the Rust DSL

### Notes

- The template is the first-run experience for every SDK user — it should showcase best practices
- Keep the example simple and focused — no type imports or generics needed, just `useParameter('param_id')`

---

## Edge Cases & Constraints

### Empty Parameter List

- If a plugin has no parameters (e.g., passthrough), the generated type should be `export type ParameterId = never`
- The generated file should still be created (prevents import errors)

### Special Characters in Parameter IDs

- Parameter IDs are derived from Rust identifiers (snake_case) — no special characters expected
- If a parameter ID somehow contains characters invalid in a TypeScript string literal, the codegen should escape or skip it with a warning

### Multiple Signal Chain Configurations

- When developers switch between signal chain configurations (e.g., commenting/uncommenting processors in `lib.rs`), the types should reflect the active configuration after rebuild

---

## Priority Assessment

| Factor            | Assessment                                                         |
| ----------------- | ------------------------------------------------------------------ |
| **User Impact**   | High — every SDK developer benefits on every parameter reference   |
| **Strategic Fit** | High — aligns with "make building plugins simple" vision           |
| **Effort**        | Low — estimated 1-2 days, rides existing infrastructure            |
| **Risk**          | Low — no new dependencies, breaking change is acceptable (pre-1.0) |
| **Dependencies**  | Low — hot-reload integration optional (works without M18.9)        |

**Recommendation:** HIGH priority. This is a high-value, low-effort DX improvement that should ship before user testing (M19). It directly addresses SDK discoverability — a key factor in first impressions with beta testers.
