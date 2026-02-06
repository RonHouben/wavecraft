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

## Test Environment

- macOS
- Rust 1.75+
- Wavecraft CLI (installed via `cargo install wavecraft`)
