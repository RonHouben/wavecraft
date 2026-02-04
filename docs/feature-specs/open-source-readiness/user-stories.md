# User Stories: Open Source Readiness (Milestone 12)

## Overview

**Problem:** The Wavecraft SDK is feature-complete (Milestones 1-11) but cannot be used by external developers. The template project has hardcoded monorepo path dependencies, there's no CLI for project scaffolding, and documentation assumes internal usage.

**Solution:** Prepare the repository for open source release by:
1. Making the template truly independent (git dependencies or crates.io)
2. Creating a CLI tool for easy project setup
3. Fixing documentation for external users
4. Repository housekeeping for public visibility

**Impact:** This milestone is the gateway to external adoption. Without it, no one outside the core team can use Wavecraft.

---

## Version

**Target Version:** `0.7.0` (minor — significant public API changes, new CLI tool)

**Rationale:** 
- This is a significant milestone that changes how developers interact with Wavecraft
- CLI tool is new functionality
- Template restructuring may have breaking changes for existing internal users
- Minor version bump signals "significant changes" to early adopters

---

## User Story 1: Template Independence

**As a** plugin developer who discovered Wavecraft on GitHub  
**I want** to clone the template and build a plugin without any other setup  
**So that** I can evaluate Wavecraft within 10 minutes

### Acceptance Criteria
- [ ] `wavecraft-plugin-template` has NO path dependencies to `../../engine/crates/`
- [ ] Template uses git dependencies pointing to the public Wavecraft repo
- [ ] `git clone && cd ui && npm install && cd ../engine && cargo xtask bundle` works
- [ ] Build completes in under 5 minutes (first time, with downloads)
- [ ] No reference to monorepo structure in template files

### Technical Notes
- Replace `path = "../../engine/crates/wavecraft-*"` with `git = "https://github.com/RonHouben/wavecraft"`
- Consider branch vs tag strategy for dependency versions
- Template README must be updated with standalone instructions

---

## User Story 2: CLI Project Scaffolding

**As a** plugin developer starting a new project  
**I want** a single command to create a complete plugin project  
**So that** I don't have to manually copy and configure files

### Acceptance Criteria
- [ ] `cargo install wavecraft-cli` installs the CLI tool
- [ ] `wavecraft new my-plugin` creates a ready-to-build plugin project
- [ ] CLI prompts for plugin name, vendor, and other metadata
- [ ] Generated project compiles and bundles without errors
- [ ] Generated project has correct plugin name throughout (not "my-plugin" placeholders)
- [ ] CLI validates that the name is a valid Rust crate name

### Technical Notes
- CLI can be a new crate: `wavecraft-cli` or part of existing tooling
- Consider using `dialoguer` for interactive prompts
- Template files can use placeholders like `{{plugin_name}}` that CLI replaces
- Future: Could also support `wavecraft add processor` for adding components

---

## User Story 3: Version-Locked Dependencies

**As a** plugin developer with a working project  
**I want** my dependencies locked to a specific Wavecraft version  
**So that** updates don't break my project unexpectedly

### Acceptance Criteria
- [ ] Template Cargo.toml uses specific version tags, not `branch = "main"`
- [ ] Each Wavecraft release creates matching git tags
- [ ] Documentation explains how to update to newer versions
- [ ] CLI generates projects with version-locked dependencies

### Technical Notes
- Use `git = "...", tag = "v0.7.0"` instead of `branch = "main"`
- Need release workflow to create tags
- Consider semantic versioning strategy for SDK crates

---

## User Story 4: Documentation for External Developers

**As a** developer reading Wavecraft docs for the first time  
**I want** clear, accurate documentation that assumes no prior context  
**So that** I can understand and use Wavecraft without asking questions

### Acceptance Criteria
- [ ] SDK Getting Started guide works for external developers (not just monorepo users)
- [ ] All documentation links work (no broken 404s)
- [ ] README clearly explains what Wavecraft is and who it's for
- [ ] Prerequisites section lists everything needed (Rust, Node, macOS)
- [ ] No references to "internal testing" or monorepo-specific workflows
- [ ] Contact/community links provided (GitHub Discussions, Discord, etc.)

### Technical Notes
- Fix the 217 broken documentation links (Issue #5 from M12 testing)
- Review all docs from "new developer" perspective
- Consider adding a troubleshooting section

---

## User Story 5: CLI Help and Documentation

**As a** developer using the Wavecraft CLI  
**I want** built-in help and clear error messages  
**So that** I can learn the tool without external documentation

### Acceptance Criteria
- [ ] `wavecraft --help` shows available commands with descriptions
- [ ] `wavecraft new --help` shows all options and examples
- [ ] Error messages are helpful (e.g., "Invalid crate name: use lowercase and underscores")
- [ ] CLI outputs progress during project creation
- [ ] Success message includes "next steps" instructions

### Technical Notes
- Use `clap` for argument parsing (consistent with xtask)
- Color output using `anstyle` or similar
- Consider `--quiet` and `--verbose` flags

---

## User Story 6: Repository Public Readiness

**As a** project maintainer  
**I want** the repository clean and ready for public eyes  
**So that** the project makes a good first impression

### Acceptance Criteria
- [ ] No secrets, API keys, or sensitive data in repository
- [ ] No TODO comments with names or internal references
- [ ] All test files pass without monorepo-specific setup
- [ ] CI/CD works for forks (no secrets required for basic checks)
- [ ] GitHub repository settings ready (description, topics, about)
- [ ] .gitignore covers all build artifacts

### Technical Notes
- Run `git log` review for sensitive commits
- Search for email addresses, internal URLs, etc.
- Review CI workflow for fork-friendliness

---

## User Story 7: Dependency Source Strategy

**As a** project maintainer  
**I want** a clear strategy for SDK dependency distribution  
**So that** we can evolve from git to crates.io when ready

### Acceptance Criteria
- [ ] Decision documented: git dependencies now, crates.io later
- [ ] Template supports easy migration to crates.io
- [ ] Release process documented for creating version tags
- [ ] Future crates.io publishing workflow planned (not implemented)

### Technical Notes
- For now: git dependencies with version tags
- Document in roadmap when crates.io publishing will happen
- Template should use version specifiers that work for both git and crates.io

---

## User Story 8: CI for Template Validation

**As a** project maintainer  
**I want** CI to automatically test that the template builds  
**So that** changes don't accidentally break the external developer experience

### Acceptance Criteria
- [ ] CI workflow tests template in isolation (not using monorepo path deps)
- [ ] Template build tested on each push to main
- [ ] CI uses same commands external developers would use
- [ ] Failure clearly indicates "template is broken for external users"

### Technical Notes
- New CI job: checkout template, use git deps, build
- May need to temporarily publish or use git submodule approach
- Consider matrix testing (multiple Rust versions)

---

## Success Criteria

1. **External developer can build a plugin from scratch**
   - Clone → Install CLI → Create project → Build → Test in DAW
   - Total time: < 15 minutes (excluding DAW testing)

2. **Documentation is accurate and complete**
   - Zero broken links
   - All commands work as documented
   - Prerequisites clearly stated

3. **Template is truly independent**
   - No path dependencies
   - Builds without monorepo present
   - Version-locked for stability

4. **CLI provides excellent DX**
   - Single command project creation
   - Helpful error messages
   - Clear next-step guidance

---

## Out of Scope (Future Milestones)

- Publishing to crates.io (M14 or later)
- Windows/Linux testing (M14 or later)
- `wavecraft add` subcommands for adding processors
- Plugin marketplace or registry
- Remote/cloud builds

---

## Dependencies

- **Requires:** M11 (Code Quality & OSS Prep) ✅ complete
- **Blocks:** M13 (Internal Testing) — testing should validate the external experience
- **Blocks:** M14 (User Testing) — external users need working template

---

## Estimated Effort

| Area | Effort |
|------|--------|
| Template independence | 1-2 days |
| CLI tool development | 3-5 days |
| Documentation fixes | 1-2 days |
| CI template validation | 1 day |
| Repository cleanup | 0.5 days |
| Testing & QA | 1-2 days |
| **Total** | **1-2 weeks** |
