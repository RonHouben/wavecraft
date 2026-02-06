# User Stories: CLI UX Improvements

## Overview

Internal testing of the Wavecraft CLI revealed several friction points in the developer onboarding experience. These improvements focus on making the CLI more intuitive, trustworthy, and self-documenting.

**Source:** [CLI Testing Findings](../internal-testing/CLI-findings.md)

## Version

**Target Version:** `0.8.0` (minor bump from `0.7.x`)

**Rationale:** These changes modify CLI behavior and public interface (removing prompts, adding help). While not breaking existing workflows, they represent a meaningful UX improvement warranting a minor version bump.

---

## User Story 1: CLI Help Command

**As a** plugin developer  
**I want** to run `wavecraft --help` to see available commands and options  
**So that** I can discover CLI capabilities without reading external documentation

### Acceptance Criteria

- [ ] Running `wavecraft --help` displays usage information
- [ ] Running `wavecraft help` displays the same information
- [ ] Running `wavecraft` with no arguments shows brief usage and suggests `--help`
- [ ] Help output includes:
  - Available commands (e.g., `new`)
  - Global options
  - Brief usage example
- [ ] Each subcommand has its own help (e.g., `wavecraft new --help`)
- [ ] Documentation in `sdk-getting-started.md` references the help command

### Notes

- Use `clap`'s built-in help generation if available
- Keep help text concise — link to docs for details

---

## User Story 2: Remove Personal Information Prompts

**As a** plugin developer  
**I want** `wavecraft new` to create a project without asking for my email or personal details  
**So that** I feel confident the CLI isn't collecting my data and I can get started faster

### Acceptance Criteria

- [ ] `wavecraft new my-plugin` creates a project without any interactive prompts
- [ ] Vendor name defaults to a placeholder (e.g., `"Your Company"` or derived from plugin name)
- [ ] Email field uses a placeholder (e.g., `"you@example.com"`)
- [ ] URL field uses a placeholder (e.g., `"https://example.com"`)
- [ ] Users can still override via optional flags: `--vendor`, `--email`, `--url`
- [ ] Generated `Cargo.toml` includes comments indicating these are placeholder values to customize
- [ ] Documentation updated to show the simplified flow (no prompts)
- [ ] Documentation explains where to customize vendor/email/url after project creation

### Notes

- The goal is zero prompts for the happy path
- Experienced users can use flags if they want to set values upfront
- This builds trust — scaffolding tools shouldn't ask for personal data

---

## User Story 3: Clean CLI Interface (Remove/Hide Internal Flags)

**As a** plugin developer  
**I want** the CLI to show only relevant options  
**So that** I'm not confused by internal SDK development flags

### Acceptance Criteria

- [ ] `--sdk-version` flag is **removed entirely**
- [ ] CLI automatically determines SDK version from its own version (`env!("CARGO_PKG_VERSION")`)
- [ ] Generated projects use git tag matching CLI version (e.g., CLI v0.8.0 → `tag = "v0.8.0"`)
- [ ] `--local-dev` is **renamed to `--sdk-path`** for clarity
- [ ] `--sdk-path` is hidden from `wavecraft new --help` output
- [ ] `--sdk-path` still works when explicitly provided (for SDK developers/CI)
- [ ] Documentation removes references to `--sdk-version` from user-facing sections
- [ ] `--sdk-path` documented in contributor/SDK developer docs only

### Technical Notes

- `--sdk-version` removal: The CLI version *is* the SDK version. No user decision needed.
- `--sdk-path` naming: Clearer than `--local-dev` — it means "use SDK from this filesystem path"
- Implementation: Use `const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");` at compile time

### Notes

- Combines findings #4 and #5 from testing
- The principle: show users what they need, hide what they don't
- SDK version should "just work" — users expect the latest

---

## User Story 4: Installation PATH Guidance

**As a** plugin developer  
**I want** clear guidance on PATH setup after installing the CLI  
**So that** I can successfully run `wavecraft` without troubleshooting

### Acceptance Criteria

- [ ] `sdk-getting-started.md` includes a "Verify Installation" section after the install step
- [ ] Documentation covers PATH setup for common shells:
  - zsh (default on macOS)
  - bash
  - fish (optional)
- [ ] Documentation includes a "Quick workaround" for users who don't want to modify PATH:
  ```bash
  ~/.cargo/bin/wavecraft new my-plugin
  ```
- [ ] Consider: Provide alternative installation method that handles PATH automatically (discuss with architect)

### Notes

- This is a Cargo ecosystem limitation, not something we can fix in CLI code
- The primary solution is better documentation
- The architect should evaluate whether an install script (`curl | sh`) is worth the effort
- Keep the happy path simple — most Rust developers already have `~/.cargo/bin` in PATH

---

## Priority Recommendation

| Story | Impact | Effort | Priority |
|-------|--------|--------|----------|
| **Story 2: Remove prompts** | High (trust + speed) | Low | 🔴 Do First |
| **Story 1: Help command** | High (discoverability) | Low | 🔴 Do First |
| **Story 4: PATH guidance** | High (unblocks users) | Low (docs only) | 🟡 Do Second |
| **Story 3: Hide flags** | Medium (cleaner UX) | Low | 🟡 Do Second |

**Rationale:** Stories 1 and 2 have the highest impact on first impressions and trust. Story 4 is critical for unblocking users but is primarily a docs change. Story 3 is polish that improves the experience but isn't blocking anyone.

---

## Out of Scope

- Install script (`curl | sh`) — to be evaluated separately by architect
- Windows/Linux-specific PATH instructions — macOS is primary focus
- Additional CLI commands beyond `new` — future work
