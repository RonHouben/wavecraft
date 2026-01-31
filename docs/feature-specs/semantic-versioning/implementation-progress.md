# Implementation Progress: Semantic Versioning

## Status: Not Started

**Started:** —  
**Last Updated:** —

---

## Task Checklist

### Phase 1: Build System — Version Extraction

- [ ] **Step 1.1:** Add `toml = "0.8"` dependency to `engine/xtask/Cargo.toml`
- [ ] **Step 1.2:** Create `read_workspace_version()` helper function in `bundle.rs`
- [ ] **Step 1.3:** Pass `VITE_APP_VERSION` env var to npm build command

### Phase 2: UI Build Configuration

- [ ] **Step 2.1:** Add `define` block to `ui/vite.config.ts`
- [ ] **Step 2.2:** Add `__APP_VERSION__` type declaration to `ui/src/vite-env.d.ts`

### Phase 3: UI Component

- [ ] **Step 3.1:** Create `ui/src/components/VersionBadge.tsx`
- [ ] **Step 3.2:** Create `ui/src/components/VersionBadge.test.tsx`
- [ ] **Step 3.3:** Update `ui/src/App.tsx` footer to include VersionBadge

### Phase 4: End-to-End Verification

- [ ] **Step 4.1:** Test development mode fallback (`npm run dev` shows `vdev`)
- [ ] **Step 4.2:** Test production build (`cargo xtask bundle` injects version)
- [ ] **Step 4.3:** Verify version matches plugin metadata in DAW

---

## Success Criteria

- [ ] Version visible in plugin UI footer
- [ ] Version follows SemVer format (e.g., `0.1.0`)
- [ ] Single source of truth in `engine/Cargo.toml`
- [ ] Plugin metadata matches UI version
- [ ] Dev mode shows `vdev`
- [ ] No manual synchronization required

---

## Notes

_Implementation notes will be added here as work progresses._
