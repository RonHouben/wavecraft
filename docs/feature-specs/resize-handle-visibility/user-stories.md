# User Stories: Resize Handle Visibility

## Overview

The resize handle in the bottom-right corner of the plugin UI needs improved visibility and usability. Current issues include low contrast, scrollbar overlap, and a small visual indicator that's hard to discover.

---

## Version

**Target Version:** `0.2.1` (patch bump from `0.2.0`)

**Rationale:** This is a UX polish improvement to an existing feature. No new functionality is being added — we're making the resize handle more visible and usable.

---

## User Story 1: Improved Contrast for Accessibility

**As a** music producer using VstKit plugins  
**I want** the resize handle to be clearly visible at all times  
**So that** I can easily find and use the resize functionality without straining my eyes

### Acceptance Criteria

- [ ] Resize handle has a minimum contrast ratio that's easily visible against the dark plugin background
- [ ] Handle is visible without requiring hover/interaction
- [ ] Maintains professional audio plugin aesthetic (not garish or distracting)

### Notes

- Current implementation uses 30% white opacity which is too subtle
- Should increase to at least 50-60% opacity at rest

---

## User Story 2: Enhanced Hover State

**As a** user interacting with the resize handle  
**I want** clear visual feedback when I hover over the handle  
**So that** I know I'm in the right spot to start dragging

### Acceptance Criteria

- [ ] Handle becomes noticeably lighter/brighter on hover
- [ ] Uses the accent color (`#4a9eff`) to match the rest of the UI
- [ ] Smooth transition between normal and hover states
- [ ] Active/dragging state is clearly distinguishable from hover

### Notes

- Current hover is barely distinguishable from rest state
- Accent color creates visual consistency with other interactive elements

---

## User Story 3: Scrollbar Clearance

**As a** user scrolling through plugin content  
**I want** the resize handle to not overlap with the scrollbar  
**So that** I can use both controls without confusion or misclicks

### Acceptance Criteria

- [ ] Resize handle is positioned to avoid overlap with browser scrollbar
- [ ] Handle remains accessible when content is scrollable
- [ ] Clear visual separation between resize handle and scrollbar area

### Notes

- Scrollbars typically take ~15-17px on macOS
- Handle should be offset to the left of the scrollbar when present

---

## User Story 4: Larger Visual Indicator

**As a** user looking for the resize functionality  
**I want** the resize handle to be larger and more prominent  
**So that** I can quickly locate it and have a comfortable drag target

### Acceptance Criteria

- [ ] Visual indicator (grip icon) is larger than current 16x16px
- [ ] Hit area is proportionally larger for easier targeting
- [ ] Size feels appropriate for the plugin UI scale (not oversized)

### Notes

- Current size: 24x24px button with 16x16px icon
- Recommended: 32x32px or larger button with proportional icon
- Larger targets improve usability, especially when dragging

---

## Out of Scope

- Changing resize behavior or constraints (min/max sizes)
- Adding resize from other edges/corners
- Mobile/touch-specific considerations

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Visual changes reviewed in actual plugin (not just browser)
- [ ] Works correctly in both light scrollbar (rare) and dark scrollbar scenarios
- [ ] No regression in existing resize functionality
- [ ] Unit tests pass
- [ ] QA approval

