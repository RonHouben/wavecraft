# Low-Level Design: Resize Handle Visibility

## 1. Overview

This design addresses visibility and usability issues with the resize handle component in the plugin UI. The changes are purely UI/CSS — no backend or IPC modifications required.

### Scope

| Area | In Scope | Out of Scope |
|------|----------|--------------|
| Visual styling | ✅ | |
| Size adjustments | ✅ | |
| Position offset | ✅ | |
| Resize behavior | | ❌ (unchanged) |
| New resize edges | | ❌ |

### Files Affected

| File | Change Type |
|------|-------------|
| `ui/src/components/ResizeHandle.tsx` | Modify (styling + size) |

---

## 2. Current State Analysis

### Current Implementation

```
┌─────────────────────────────────────────────────────────────┐
│                        Plugin Window                         │
│                                                             │
│                                                             │
│                                                     ████████│ ← Scrollbar (~15px)
│                                                     ████████│
│                                                     ████████│
│                                                        ┌───┐│
│                                                        │ ╲ ││ ← Handle overlaps
│                                                        └───┘│    scrollbar zone
└─────────────────────────────────────────────────────────────┘
```

**Current Specifications:**
- Button size: 24×24px (`h-6 w-6`)
- Icon size: 16×16px
- Position: `fixed bottom-0 right-0`
- Rest opacity: 30% white (`text-white/30`)
- Hover opacity: 60% white (`text-white/60`)
- Active opacity: 80% white (`text-white/80`)
- Background: Transparent, subtle white tint on hover

**Issues:**
1. Low contrast at rest (30% opacity barely visible on `#1a1a1a`)
2. Overlaps scrollbar (fixed to absolute corner)
3. Small target size (24px is minimum acceptable)
4. No brand color connection (uses neutral white)

---

## 3. Proposed Design

### 3.1 Visual Design

```
┌─────────────────────────────────────────────────────────────┐
│                        Plugin Window                         │
│                                                             │
│                                                             │
│                                                     ████████│ ← Scrollbar
│                                                     ████████│
│                                              ┌─────┐████████│
│                                              │  ╲  │        │
│                                              │   ╲ │        │ ← Handle clear
│                                              └─────┘        │    of scrollbar
└─────────────────────────────────────────────────────────────┘
                                               ↑
                                          20px from right edge
```

### 3.2 Size Specifications

| Property | Current | Proposed | Rationale |
|----------|---------|----------|-----------|
| Button size | 24×24px | 36×36px | Larger hit target, better discoverability |
| Icon size | 16×16px | 20×20px | Proportional to button, clearer visibility |
| Right offset | 0px | 20px | Clear scrollbar (~15-17px + margin) |
| Bottom offset | 0px | 4px | Slight lift for visual balance |

### 3.3 Color Specifications

| State | Current | Proposed | Rationale |
|-------|---------|----------|-----------|
| **Rest** | `text-white/30` | `text-white/50` | 67% increase in visibility |
| **Hover** | `text-white/60` | `text-accent` | Brand consistency, clear affordance |
| **Active** | `text-white/80` | `text-accent-light` | Distinct feedback during drag |
| **Background (hover)** | `bg-white/5` | `bg-white/10` | Subtle container highlight |
| **Background (active)** | `bg-white/10` | `bg-accent/20` | Accent tint during drag |

### 3.4 State Diagram

```
                    ┌──────────────────┐
                    │      REST        │
                    │                  │
                    │ Icon: white/50   │
                    │ BG: transparent  │
                    └────────┬─────────┘
                             │
                    mouse enter
                             │
                             ▼
                    ┌──────────────────┐
                    │     HOVER        │
                    │                  │
                    │ Icon: accent     │
                    │ BG: white/10     │
                    └────────┬─────────┘
                             │
                    mouse down
                             │
                             ▼
                    ┌──────────────────┐
                    │    DRAGGING      │
                    │                  │
                    │ Icon: accent-light│
                    │ BG: accent/20    │
                    └──────────────────┘
```

---

## 4. Implementation Details

### 4.1 Component Changes

**File:** `ui/src/components/ResizeHandle.tsx`

#### Position & Size Changes

```tsx
// BEFORE
className={`group fixed bottom-0 right-0 z-[9999] flex h-6 w-6 ...`}

// AFTER  
className={`group fixed bottom-1 right-5 z-[9999] flex h-9 w-9 ...`}
```

| Tailwind Class | Value | Purpose |
|----------------|-------|---------|
| `bottom-1` | 4px | Slight lift from bottom edge |
| `right-5` | 20px | Clear scrollbar width |
| `h-9 w-9` | 36×36px | Larger hit target |

#### Color Changes

```tsx
// BEFORE (button)
isDragging ? 'bg-white/10' : 'hover:bg-white/5'

// AFTER (button)
isDragging ? 'bg-accent/20' : 'hover:bg-white/10'

// BEFORE (SVG)
isDragging ? 'text-white/80' : 'text-white/30 group-hover:text-white/60'

// AFTER (SVG)
isDragging ? 'text-accent-light' : 'text-white/50 group-hover:text-accent'
```

#### SVG Size Changes

```tsx
// BEFORE
<svg width="16" height="16" viewBox="0 0 16 16" ...>

// AFTER
<svg width="20" height="20" viewBox="0 0 16 16" ...>
```

Note: `viewBox` stays at `0 0 16 16` — the SVG scales up. The grip lines maintain their proportions.

### 4.2 Full Diff Preview

```diff
  return (
    <button
-     className={`group fixed bottom-0 right-0 z-[9999] flex h-6 w-6 cursor-nwse-resize select-none items-center justify-center border-none bg-transparent p-0 transition-colors duration-150 ${
-       isDragging ? 'bg-white/10' : 'hover:bg-white/5'
+     className={`group fixed bottom-1 right-5 z-[9999] flex h-9 w-9 cursor-nwse-resize select-none items-center justify-center rounded border-none bg-transparent p-0 transition-colors duration-150 ${
+       isDragging ? 'bg-accent/20' : 'hover:bg-white/10'
      }`}
      onMouseDown={handleMouseDown}
      aria-label="Resize window"
      type="button"
    >
      <svg
-       width="16"
-       height="16"
+       width="20"
+       height="20"
        viewBox="0 0 16 16"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className={`transition-colors duration-150 ${
-         isDragging ? 'text-white/80' : 'text-white/30 group-hover:text-white/60'
+         isDragging ? 'text-accent-light' : 'text-white/50 group-hover:text-accent'
        }`}
      >
```

---

## 5. Accessibility Considerations

### Contrast Ratios

| State | Foreground | Background | Approx. Ratio | WCAG |
|-------|------------|------------|---------------|------|
| Rest | `white/50` (#808080) | `#1a1a1a` | ~4.5:1 | AA ✅ |
| Hover | `#4a9eff` | `#1a1a1a` | ~5.5:1 | AA ✅ |
| Active | `#6bb0ff` | `#1a1a1a` | ~6.5:1 | AAA ✅ |

### Keyboard Accessibility

The current implementation uses a `<button>` element with proper `aria-label`. No changes needed — resize via keyboard is not a standard pattern for plugin UIs.

---

## 6. Testing Strategy

### Visual Verification

| Test Case | Expected Result |
|-----------|-----------------|
| Rest state visibility | Handle clearly visible at 50% white |
| Hover state | Icon turns accent blue, background lightens |
| Active/drag state | Icon turns light accent, background has blue tint |
| Scrollbar clearance | Handle does not overlap scrollbar |
| Size increase | Handle noticeably larger, easier to target |

### Functional Verification

| Test Case | Expected Result |
|-----------|-----------------|
| Drag to resize | Works identically to before |
| Min size constraint | Still enforced (400×300) |
| Release outside window | Drag ends cleanly |

### Browser Compatibility

| Browser | Scrollbar Width | Right Offset | Clearance |
|---------|-----------------|--------------|-----------|
| Safari/WKWebView | ~15px | 20px | ✅ 5px margin |
| Chrome | ~17px | 20px | ✅ 3px margin |
| Firefox | ~15px | 20px | ✅ 5px margin |

---

## 7. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Accent color clashes with future themes | Low | Low | Accent is already used throughout UI; this adds consistency |
| 20px offset too large for some configs | Low | Low | Scrollbars rarely exceed 17px; 20px is safe |
| Larger handle feels out of place | Low | Medium | 36px is still modest; review in actual plugin context |

---

## 8. Future Considerations

**Not in scope, but noted for future:**

1. **Dynamic scrollbar detection**: Could detect if content is scrollable and conditionally offset. Adds complexity for minimal gain.

2. **Theme support**: If dark/light themes are added, handle colors should adapt. Current design uses semantic tokens (`accent`, `accent-light`) which would inherit theme changes.

3. **Touch targets**: If mobile/tablet support is added, 36px is still below the recommended 44px. Would need larger targets.

---

## 9. Implementation Checklist

- [ ] Update button position classes (`bottom-1 right-5`)
- [ ] Update button size classes (`h-9 w-9`)
- [ ] Add `rounded` class for subtle corner rounding
- [ ] Update button background colors (hover/active)
- [ ] Update SVG dimensions (`width="20" height="20"`)
- [ ] Update SVG color classes (rest/hover/active)
- [ ] Visual testing in plugin context
- [ ] Verify existing resize functionality unchanged
- [ ] Run unit tests

---

## 10. Appendix: Color Reference

```
Plugin Dark Background: #1a1a1a

Rest State:
  Icon: rgba(255, 255, 255, 0.5) = #808080 effective
  
Hover State:
  Icon: #4a9eff (accent)
  Background: rgba(255, 255, 255, 0.1)
  
Active State:
  Icon: #6bb0ff (accent-light)
  Background: rgba(74, 158, 255, 0.2)
```
