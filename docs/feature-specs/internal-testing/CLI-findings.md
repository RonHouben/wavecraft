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

### 4. CLI: `--sdk-version` flag is confusing for users

**Severity:** Medium  
**Status:** 🔴 Open

**Issue:** The `--sdk-version` option is exposed to end users, but from a user's perspective it's unclear why they would need to specify this. Users expect to always use the latest version.

**Current behavior:**
```bash
wavecraft new my-plugin --sdk-version "v0.7.0"
```

**Recommendation:** 
- The CLI should automatically use the latest SDK version (matching the installed CLI version)
- Hide `--sdk-version` from regular help output
- Only expose it as an advanced/hidden flag for SDK developers or specific debugging scenarios

**Impact:** Confuses new users and adds unnecessary complexity to the getting-started flow.

---

## Test Environment

- macOS
- Rust 1.75+
- Wavecraft CLI (installed via `cargo install wavecraft`)
