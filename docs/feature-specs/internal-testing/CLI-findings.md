# Wavecraft CLI — Internal Testing Findings

Testing Date: 2026-02-06

## Summary

Findings from internal testing of the Wavecraft CLI user experience.

---

## Findings

### 1. Documentation: `sdk-getting-started.md` unclear opening

**Severity:** Low  
**Status:** ✅ Fixed

**Issue:** The getting started guide opened with package installation details instead of the Quick Start workflow. The title "Wavecraft SDK — Getting Started" appeared twice.

**Resolution:** Removed the duplicate intro and "Published Packages" section so users see Prerequisites → Quick Start immediately.

---

### 2. CLI: No help command

**Severity:** Medium  
**Status:** 🔴 Open

**Issue:** The CLI does not provide a `--help` flag or `help` subcommand to discover available commands and options.

**Expected:** Running `wavecraft --help` or `wavecraft help` should display:
- Available commands (e.g., `new`)
- Global options
- Brief usage examples

**Impact:** Users cannot discover CLI capabilities without reading documentation.

---

### 3. CLI: Asking for personal information feels invasive

**Severity:** High  
**Status:** 🔴 Open

**Issue:** The `wavecraft new` command prompts for vendor name, email, and website URL. This feels like the CLI is gathering personal data, which creates distrust.

**Current behavior:**
```bash
wavecraft new my-plugin
# Prompts for:
# - Vendor name
# - Email address
# - Website URL
```

**Recommendation:** Remove these prompts entirely. Use sensible defaults or placeholder values that users can optionally edit later in their project's `Cargo.toml` or plugin metadata.

**Impact:** 
- Creates distrust ("why does a scaffolding tool need my email?")
- Slows down the getting-started experience
- These fields are optional for plugin development anyway

---

## Test Environment

- macOS
- Rust 1.75+
- Wavecraft CLI (installed via `cargo install wavecraft`)
