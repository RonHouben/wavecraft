# Resize Ownership Inventory — ui-ux-refactor (Phase 5.1)

## Scan scope

- `ui/packages/components/src/**`
- `sdk-template/ui/src/**`

## Findings

- No `new ResizeObserver(...)` construction exists in current UI sources.
- Existing resize flow uses explicit resize requests via `useRequestResize`.

## Ownership outcome

- Resize request ownership is now in `sdk-template/ui` smart layer (`App.tsx`).
- `@wavecraft/components` provides presentational resize controls (`ResizeHandle`, `ResizeControls`) that receive an `onRequestResize` callback and contain no transport/runtime logic.

## Notes

- Because no ResizeObserver instances were present in this feature scope, Phase 5 migration focused on explicit ownership boundaries rather than observer relocation.
