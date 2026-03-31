# Test Plan — Settings Modal Opener

## Scope

Verify that the SDK-managed `useSettingsModal` hook can open `SettingsModal` from the sidebar button without external modal state management in `sdk-template/ui/src/App.tsx`.

## Automated Validation

### Focused UI checks

- `npm run typecheck` in `ui/` ✅
- `npm test -- --run packages/core/src/context/WavecraftProvider.test.tsx packages/components/src/TemplateApp.test.tsx packages/components/src/settings/SettingsModal.test.tsx` ✅
- `npm test -- --run packages/components/src/Modal.test.tsx packages/components/src/settings/SettingsModal.test.tsx packages/components/src/TemplateApp.test.tsx packages/core/src/context/WavecraftProvider.test.tsx` ✅

### Covered behaviors

- Shared settings modal state is available to multiple consumers under one `WavecraftProvider`
- `useSettingsModal` fails fast outside `WavecraftProvider`
- Sidebar Settings button calls the opener hook
- `SettingsModal` renders as an accessible dialog and closes via backdrop, close button, and `Escape`
- Stacked `Sidebar` + `Modal` flow keeps the sidebar open when `Escape` closes the topmost modal

## Manual UI Validation

### Environment

- URL: `http://localhost:5173/`
- Browser surface: VS Code integrated browser session
- Date: 2026-03-28

### Checks

1. Load plugin UI and confirm main app renders ✅
2. Open sidebar via the menu button in the header ✅
3. Click **Settings** in the sidebar and confirm `Plugin settings` modal appears ✅
4. Confirm settings form content renders in the modal ✅
5. Press `Escape` and confirm the modal closes while the sidebar remains open ✅

## Evidence

- In-session screenshot: main plugin UI rendered after provider/hook fix
- In-session screenshot: sidebar + `Plugin settings` modal open
- In-session screenshot: sidebar remains open after `Escape` closes the modal

## Notes

A broader `cargo xtask ci-check` run in this repository still reports unrelated pre-existing failures outside this change (existing UI lint issues and unrelated test failures in other areas). The settings modal implementation itself passed focused lint, typecheck, test, and manual UI verification.
