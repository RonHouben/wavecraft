# Resize Handle Visibility Improvements

## Summary

Improves the visibility and usability of the resize handle in the plugin UI. This is a UX polish release (v0.2.1) addressing user feedback about the handle being hard to discover and use.

**Key improvements:**
- Increased contrast: 30% → 50% white opacity at rest
- Added accent colors: Blue on hover (#4a9eff), light blue when dragging (#6bb0ff)
- Larger target: 24×24px → 36×36px button, 16×16px → 20×20px icon
- Scrollbar clearance: 20px offset from right edge
- **Bonus:** Fixed WebView background color mismatch during over-scroll

## Changes

### UI Components
- **ResizeHandle.tsx**: Updated Tailwind classes for size, position, and colors
- **index.css**: Added `bg-plugin-dark` to body and #root for WebView background
- **index.html**: Added inline background color as pre-CSS fallback

### Engine (Rust)
- **Cargo.toml**: Version bump 0.2.0 → 0.2.1
- **editor/*.rs**: Added `#[allow(dead_code)]` to 7 items (pre-existing unused code)

### Documentation
- **coding-standards.md**: Added "WebView Background Color" section
- **Feature spec**: Complete documentation (user-stories, low-level-design, implementation-plan, test-plan, QA-report)

## Commits

- `aa982a3` feat: add user stories for resize handle visibility improvements
- `8d87a3c` feat: implement low-level design for resize handle visibility improvements
- `ab3becb` feat: add implementation plan and progress tracker
- `001dce1` feat: enhance user stories and documentation
- `710fe4d` feat: update version numbers and enhance ResizeHandle component styling
- `3be1d9c` feat: resolve Clippy dead code errors
- `7df1d6d` feat: add dead code cleanup task to roadmap
- `d6a6f0b` feat: update test plan and roadmap, add DAW installation instructions
- `5e242eb` feat: update test plan with DAW testing results
- `b60c23b` feat: add QA report and update index.css styling
- `43944d6` feat: add guidelines for WebView background color
- `0b3a388` Archive resize-handle-visibility feature spec and update roadmap

## Related Documentation

- [User Stories](docs/feature-specs/_archive/resize-handle-visibility/user-stories.md)
- [Low-Level Design](docs/feature-specs/_archive/resize-handle-visibility/low-level-design-resize-handle-visibility.md)
- [Implementation Plan](docs/feature-specs/_archive/resize-handle-visibility/implementation-plan.md)
- [Test Plan](docs/feature-specs/_archive/resize-handle-visibility/test-plan.md)
- [QA Report](docs/feature-specs/_archive/resize-handle-visibility/QA-report.md)

## Testing

### Automated Tests
- ✅ All 35 UI unit tests pass
- ✅ ESLint: 0 errors, 0 warnings
- ✅ Prettier: Formatted
- ✅ cargo fmt: Pass
- ✅ cargo clippy: Pass (with dead code suppressions)

### Manual Tests (13/13 PASS)
- ✅ Rest state visibility (50% white)
- ✅ Hover state (accent blue)
- ✅ Active/dragging state (accent light)
- ✅ Handle size (36×36px)
- ✅ Scrollbar clearance (20px offset)
- ✅ Rounded corners
- ✅ Drag to resize (DAW)
- ✅ Drag to shrink (DAW)
- ✅ Minimum size constraint (400×300)
- ✅ Contrast ratio verification
- ✅ Color transition smoothness
- ✅ Browser compatibility
- ✅ Plugin build verification (Ableton Live)

## Checklist

- [x] Code follows project coding standards
- [x] All automated tests pass (`cargo xtask test`)
- [x] All linting passes (`cargo xtask lint`)
- [x] No new warnings introduced
- [x] Documentation updated
- [x] Feature spec archived
- [x] Roadmap updated
- [x] QA approved
