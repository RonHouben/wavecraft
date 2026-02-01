# Implementation Plan: Resize Handle Visibility

## Overview

Improve the visibility and usability of the resize handle component by increasing contrast, adding accent colors, enlarging the visual indicator, and offsetting from the scrollbar.

**Complexity:** Low  
**Estimated Time:** 30-45 minutes  
**Files Changed:** 1 (`ui/src/components/ResizeHandle.tsx`)

---

## Requirements Summary

From user stories:
1. ✅ Improved contrast for accessibility (50% white at rest)
2. ✅ Enhanced hover state with accent color
3. ✅ Scrollbar clearance (20px offset)
4. ✅ Larger visual indicator (36×36px button, 20×20px icon)

---

## Architecture Changes

| File | Change Type | Description |
|------|-------------|-------------|
| `ui/src/components/ResizeHandle.tsx` | Modify | Update Tailwind classes for size, position, and colors |

No new files. No backend changes. No new dependencies.

---

## Implementation Steps

### Phase 1: Position & Size Updates

#### Step 1.1: Update Button Position Classes

**File:** `ui/src/components/ResizeHandle.tsx`

**Action:** Change position from absolute corner to offset position

**Before:**
```tsx
className={`group fixed bottom-0 right-0 z-[9999] flex h-6 w-6 ...`}
```

**After:**
```tsx
className={`group fixed bottom-1 right-5 z-[9999] flex h-9 w-9 ...`}
```

| Class Change | Effect |
|--------------|--------|
| `bottom-0` → `bottom-1` | 4px from bottom (subtle lift) |
| `right-0` → `right-5` | 20px from right (clears scrollbar) |
| `h-6 w-6` → `h-9 w-9` | 24px → 36px (50% larger) |

**Dependencies:** None  
**Risk:** Low

---

#### Step 1.2: Add Rounded Corners

**File:** `ui/src/components/ResizeHandle.tsx`

**Action:** Add `rounded` class for subtle corner rounding

**Why:** Larger button benefits from slight rounding to soften appearance

**Dependencies:** Step 1.1  
**Risk:** Low

---

### Phase 2: Color Updates

#### Step 2.1: Update Button Background Colors

**File:** `ui/src/components/ResizeHandle.tsx`

**Action:** Change hover and active background colors

**Before:**
```tsx
isDragging ? 'bg-white/10' : 'hover:bg-white/5'
```

**After:**
```tsx
isDragging ? 'bg-accent/20' : 'hover:bg-white/10'
```

| State | Before | After |
|-------|--------|-------|
| Hover | 5% white | 10% white |
| Active | 10% white | 20% accent |

**Dependencies:** None  
**Risk:** Low

---

#### Step 2.2: Update SVG Icon Colors

**File:** `ui/src/components/ResizeHandle.tsx`

**Action:** Change icon colors to use accent on hover/active

**Before:**
```tsx
className={`transition-colors duration-150 ${
  isDragging ? 'text-white/80' : 'text-white/30 group-hover:text-white/60'
}`}
```

**After:**
```tsx
className={`transition-colors duration-150 ${
  isDragging ? 'text-accent-light' : 'text-white/50 group-hover:text-accent'
}`}
```

| State | Before | After |
|-------|--------|-------|
| Rest | 30% white | 50% white |
| Hover | 60% white | Accent (`#4a9eff`) |
| Active | 80% white | Accent-light (`#6bb0ff`) |

**Dependencies:** None  
**Risk:** Low

---

### Phase 3: Icon Size Update

#### Step 3.1: Increase SVG Dimensions

**File:** `ui/src/components/ResizeHandle.tsx`

**Action:** Scale SVG from 16×16 to 20×20

**Before:**
```tsx
<svg
  width="16"
  height="16"
  viewBox="0 0 16 16"
```

**After:**
```tsx
<svg
  width="20"
  height="20"
  viewBox="0 0 16 16"
```

**Note:** `viewBox` remains `0 0 16 16` — the SVG scales proportionally. Grip lines maintain their relative positions.

**Dependencies:** None  
**Risk:** Low

---

### Phase 4: Verification

#### Step 4.1: Run Unit Tests

**Command:**
```bash
cargo xtask test --ui
```

**Expected:** All tests pass (no behavioral changes)

**Dependencies:** Steps 1-3 complete  
**Risk:** Low

---

#### Step 4.2: Visual Verification in Browser

**Command:**
```bash
cd ui && npm run dev
```

**Verify:**
- [ ] Handle visible at rest (50% white)
- [ ] Handle turns accent blue on hover
- [ ] Handle turns light blue when dragging
- [ ] Handle positioned 20px from right edge
- [ ] Handle sized 36×36px
- [ ] Smooth color transitions

**Dependencies:** Step 4.1  
**Risk:** Low

---

#### Step 4.3: Visual Verification in Plugin

**Command:**
```bash
cargo xtask bundle && cargo xtask sign
```

Then load in Ableton Live.

**Verify:**
- [ ] Handle does not overlap scrollbar
- [ ] Resize functionality unchanged
- [ ] Colors match browser preview

**Dependencies:** Step 4.2  
**Risk:** Low

---

## Testing Strategy

### Unit Tests

No new unit tests needed — this is a styling-only change. Existing tests verify:
- Component renders
- Mouse events work

### Manual Tests

| # | Test Case | Steps | Expected Result |
|---|-----------|-------|-----------------|
| 1 | Rest state visibility | Load plugin UI | Handle clearly visible at ~50% white |
| 2 | Hover state | Move mouse over handle | Icon turns accent blue, background lightens |
| 3 | Active state | Click and hold handle | Icon turns light accent, background has blue tint |
| 4 | Drag functionality | Drag handle to resize | Window resizes as before |
| 5 | Scrollbar clearance | Scroll content | Handle does not overlap scrollbar |
| 6 | Size verification | Inspect element | Button 36×36px, icon 20×20px |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Accent colors don't render in WKWebView | Very Low | Medium | Accent colors already used in UI; same CSS mechanism |
| Offset causes handle to be off-screen on small windows | Very Low | Low | Min window size is 400×300; 36px handle + 20px offset fits |
| Tests fail due to snapshot/class changes | Low | Low | No snapshot tests; class changes don't affect test assertions |

---

## Success Criteria

- [ ] Handle visible at rest without squinting
- [ ] Hover state clearly indicates interactivity with accent color
- [ ] Active/drag state distinct from hover
- [ ] Handle does not overlap scrollbar when content is scrollable
- [ ] Handle feels appropriately sized (not too small, not oversized)
- [ ] All unit tests pass
- [ ] Resize functionality unchanged

---

## Rollback Plan

If issues are discovered:

1. Revert single file: `git checkout HEAD -- ui/src/components/ResizeHandle.tsx`
2. Rebuild: `cargo xtask bundle`

No database migrations, no API changes, no coordination required.

---

## Implementation Order Summary

```
┌─────────────────────────────────────────────────────────────┐
│  Phase 1: Position & Size                                   │
│  ├─ 1.1 Update position classes (bottom-1 right-5 h-9 w-9) │
│  └─ 1.2 Add rounded corners                                │
├─────────────────────────────────────────────────────────────┤
│  Phase 2: Colors                                            │
│  ├─ 2.1 Update button background (hover/active)            │
│  └─ 2.2 Update SVG icon colors (rest/hover/active)         │
├─────────────────────────────────────────────────────────────┤
│  Phase 3: Icon Size                                         │
│  └─ 3.1 Increase SVG dimensions (16→20)                    │
├─────────────────────────────────────────────────────────────┤
│  Phase 4: Verification                                      │
│  ├─ 4.1 Run unit tests                                     │
│  ├─ 4.2 Visual verification in browser                     │
│  └─ 4.3 Visual verification in plugin                      │
└─────────────────────────────────────────────────────────────┘
```

All steps in Phases 1-3 can be done in a single edit since they're all in the same file with no dependencies on each other.
