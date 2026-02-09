# Coding Standards — CSS & Styling

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [TypeScript Standards](./coding-standards-typescript.md) — TypeScript and React conventions
- [High-Level Design](./high-level-design.md) — Architecture overview

---

## CSS / Styling (TailwindCSS)

### Utility-First Styling

**Rule:** Use TailwindCSS utility classes for all styling. Do not create separate CSS files for components.

The project uses TailwindCSS with a custom theme defined in `ui/tailwind.config.js`.

**Do:**
```tsx
// ✅ Utility classes in JSX
<div className="flex flex-col gap-2 rounded-lg border border-plugin-border bg-plugin-surface p-4">
  <h3 className="text-base font-semibold text-gray-200">Section Title</h3>
  <p className="text-sm text-gray-400">Description text</p>
</div>
```

**Don't:**
```tsx
// ❌ Separate CSS files
import './MyComponent.css';

<div className="my-component">
  <h3 className="my-component__title">Section Title</h3>
</div>
```

### Theme Tokens

**Rule:** Use semantic theme tokens instead of hardcoded colors.

| Token | Usage | Value |
|-------|-------|-------|
| `bg-plugin-dark` | Main background | `#1a1a1a` |
| `bg-plugin-surface` | Card/section backgrounds | `#2a2a2a` |
| `border-plugin-border` | Borders | `#444444` |
| `text-accent` | Primary accent color | `#4a9eff` |
| `text-accent-light` | Accent hover state | `#6bb0ff` |
| `bg-meter-safe` | Meter safe zone | `#4caf50` |
| `bg-meter-warning` | Meter warning zone | `#ffeb3b` |
| `bg-meter-clip` | Meter clipping | `#ff1744` |

**Do:**
```tsx
// ✅ Semantic theme tokens
<div className="bg-plugin-surface border-plugin-border text-accent">
```

**Don't:**
```tsx
// ❌ Hardcoded colors (except where no token exists)
<div className="bg-[#2a2a2a] border-[#444444] text-[#4a9eff]">
```

### Custom CSS (Exceptions)

**Rule:** Only use `@layer` directives for CSS that cannot be expressed as utility classes.

Valid exceptions:
- Vendor-prefixed pseudo-elements (`::-webkit-slider-thumb`)
- Complex animations requiring `@keyframes`
- Browser-specific hacks

**Example (slider thumb styling):**
```css
@layer components {
  .slider-thumb::-webkit-slider-thumb {
    @apply h-[18px] w-[18px] cursor-pointer rounded-full bg-accent;
  }
}
```

### Class Organization

**Rule:** Order Tailwind classes logically: layout → spacing → colors → typography → effects.

**Do:**
```tsx
// ✅ Logical grouping
<div className="flex flex-col gap-2 p-4 bg-plugin-surface text-gray-200 rounded-lg">
```

**Don't:**
```tsx
// ❌ Random ordering
<div className="text-gray-200 rounded-lg flex p-4 bg-plugin-surface flex-col gap-2">
```

**Note:** The Prettier Tailwind plugin automatically sorts classes. Run `npm run format` to apply.

### WebView Background Color

**Rule:** Both `body` and `#root` must have explicit background colors matching the theme.

The WebView shows a default white background when the user scrolls beyond the content boundaries (over-scroll/rubber-band scrolling). To prevent this visual inconsistency:

1. Apply `bg-plugin-dark` to both `body` and `#root` in `index.css`
2. Add an inline `style="background-color: #1a1a1a;"` to the `<html>` element in `index.html` as a pre-CSS fallback

**Why both?**
- `body` background covers the document area
- `#root` background covers the React app container
- `<html>` inline style prevents white flash before CSS loads

**Example (`index.css`):**
```css
@layer base {
  body {
    @apply m-0 bg-plugin-dark p-0;
  }

  #root {
    @apply h-screen w-full overflow-y-auto bg-plugin-dark;
  }
}
```

**Example (`index.html`):**
```html
<html lang="en" style="background-color: #1a1a1a;">
```

### File Structure

```
ui/
├── tailwind.config.js    # Theme configuration (colors, fonts, animations)
├── postcss.config.js     # PostCSS plugins
└── src/
    ├── index.css         # Tailwind directives + @layer overrides only
    └── components/
        └── *.tsx         # All styling via className utilities
```

No `.css` files should exist in `src/components/`.
