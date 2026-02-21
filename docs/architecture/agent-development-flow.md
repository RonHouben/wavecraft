# Agent Development Flow

This project uses specialized agents with distinct responsibilities that hand off to each other throughout the development lifecycle.

## Agent Roles

| Agent                  | Role                                                                                  | Key Outputs                                           |
| ---------------------- | ------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| **Orchestrator**       | Workflow coordinator, routes work between agents                                      | Phase tracking, handoff decisions                     |
| **PO** (Product Owner) | Owns product vision, roadmap, feature prioritization                                  | User stories, `docs/roadmap.md`                       |
| **Architect**          | Designs system architecture, enforces technical constraints                           | Low-level designs in `docs/feature-specs/{feature}/`  |
| **Planner**            | Creates detailed implementation plans                                                 | `docs/feature-specs/{feature}/implementation-plan.md` |
| **Coder**              | Implements features, writes production code                                           | Code changes, PRs                                     |
| **Tester**             | Runs local CI pipeline, executes manual tests                                         | `docs/feature-specs/{feature}/test-plan.md`           |
| **QA**                 | Static analysis, code quality verification                                            | QA reports                                            |
| **DocWriter**          | Creates and updates all documentation                                                 | All markdown files in `docs/`                         |
| **Search**             | Deep codebase research and analysis                                                   | Search results, code explanations                     |
| **Cleaner**            | Investigates and cleans up code/doc noise, dead code, duplicates, missed abstractions | Clean code changes, cleanup reports                   |
| **UX Designer**        | Designs and implements UI/UX changes; owns the frontend design system and React components | Code changes in `ui/`, `sdk-template/ui/`        |

## Standard Feature Development Flow

**Note:** The Orchestrator agent serves as the central coordinator for this workflow, routing work between specialized agents. Users can start with either the Orchestrator (recommended for full features) or go directly to a specific agent for targeted work.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FEATURE DEVELOPMENT FLOW                         │
│                   (Orchestrator coordinates all phases)                 │
└─────────────────────────────────────────────────────────────────────────┘

  ┌──────┐
  │  PO  │  Feature Request / User Story
  └──┬───┘
     │
     │  "Create low level design"
     ▼
┌──────────┐
│ Architect│  Low-Level Design
└────┬─────┘  └─► docs/feature-specs/{feature}/low-level-design-{feature}.md
     │
     │  "Create implementation plan"
     ▼
┌─────────┐
│ Planner │  Implementation Plan
└────┬────┘  └─► docs/feature-specs/{feature}/implementation-plan.md
     │
     │  "Start Implementation"
     ▼
┌────────┐
│ Coder  │  Implementation + Create PR
└────┬───┘  └─► docs/feature-specs/{feature}/implementation-progress.md
     │
     │  "Test Implementation"
     ▼
┌────────┐◄─────────────────────────────────────────────────────┐
│ Tester │  Manual Testing                                      │
└────┬───┘  └─► docs/feature-specs/{feature}/test-plan.md       │
     │                                                          │
     │ ─────────────────────────────────────────────┐           │
     │  Test Issues Found?                          │ No Issues │
     ▼                                              ▼           │
┌────────┐                                    ┌──────┐          │
│ Coder  │  Fix Test Issues ──► Re-test       │  QA  │  Static  │
└────────┘                                    └──┬───┘  Analysis│
                                                 │     └─► QA-report.md
                                   ┌─────────────┴─────────────┐
                                   │  QA Issues Found?         │ No Issues
                                   ▼                           ▼
                             ┌────────┐                  ┌──────────┐
                             │ Coder  │  Fix Findings    │ Architect│
                             └────┬───┘                  └────┬─────┘
                                  │                           │ Update
                                  └──────► Re-test ───────────┤ Arch Docs
                                                              ▼
                                                        ┌──────┐
                                                        │  PO  │  Update
                                                        └──┬───┘  Roadmap
                                                           │      Archive
                                                           ▼
                                                    ✅ Feature Complete
                                                       (Manual PR Merge)
```

## Lightweight Workflow for Bug Fixes

For **bug fixes** and **minor improvements** that don't require architectural changes, use a reduced documentation set:

**Required documents:**

1. `test-plan.md` — Test cases, steps to reproduce, verification
2. `PR-summary.md` — Changes made, files affected, testing notes (auto-generated by create-pull-request skill)

**Omitted documents:**

- `user-stories.md` — Not needed for bug fixes
- `low-level-design-{feature}.md` — No architectural changes
- `implementation-plan.md` — Bug fixes are typically small enough to not require detailed planning

**Workflow:**

1. **Coder** identifies bug, creates `test-plan.md` with reproduction steps
2. **Coder** implements fix
3. **Coder** runs `cargo xtask ci-check` (pre-handoff checks)
4. **Coder** creates PR using `create-pull-request` skill (auto-generates `PR-summary.md`)
5. **Tester** executes test-plan.md, verifies fix
6. **QA** reviews code quality (optional for trivial fixes)
7. **PO** archives feature folder after merge

**When to use lightweight workflow:**

- Bug fixes
- Performance optimizations
- Refactoring without behavior changes
- Documentation updates
- Dependency updates

**When to use full workflow:**

- New features
- API changes
- Architectural modifications
- Breaking changes

---

## UI/UX Change Workflow

For **UI/UX changes** — React component work, design system adjustments, accessibility improvements, and visual redesigns — route to the **UX Designer** agent.

**Workflow path:**

1. **Orchestrator** routes the UI/UX task to UX Designer
2. **UX Designer** researches existing patterns → proposes UX approach → implements (following required skill order)
3. **UX Designer** hands off to **Tester** for visual and functional validation
4. **Tester** returns results; if issues are found, UX Designer iterates
5. **UX Designer** hands back to **Orchestrator** on completion

**Scope:** `ui/` and `sdk-template/ui/` only. Rust engine code is out of scope for this workflow.

**When to use:**

- New React components or changes to existing components
- TailwindCSS / design system token updates
- Accessibility improvements
- Layout and interaction redesigns
- Visual consistency fixes

---

## Handoff Summary

**Note:** The Orchestrator agent coordinates these handoffs. Agents can hand off directly (when context is clear) or route through Orchestrator (recommended for phase transitions).

| From                     | To                        | Trigger                   | What Gets Passed |
| ------------------------ | ------------------------- | ------------------------- | ---------------- |
| Orchestrator → PO        | "Define requirements"     | User's feature request    |
| PO → Orchestrator        | "Requirements complete"   | User stories document     |
| Orchestrator → Architect | "Create design"           | User stories              |
| Architect → Orchestrator | "Design complete"         | Low-level design document |
| Orchestrator → Planner   | "Create plan"             | Low-level design          |
| Planner → Orchestrator   | "Plan complete"           | Implementation plan       |
| Orchestrator → Coder     | "Start implementation"    | Implementation plan       |
| Coder → Orchestrator     | "Implementation complete" | Code + PR + progress doc  |
| Orchestrator → Tester    | "Test implementation"     | Completed implementation  |
| Tester → Orchestrator    | "Tests complete/failed"   | Test results + test plan  |
| Orchestrator → Coder     | "Fix issues"              | Test failures             |
| Orchestrator → QA        | "Quality review"          | All tests passing         |
| QA → Orchestrator        | "QA complete/issues"      | QA report                 |
| Orchestrator → Coder     | "Fix findings"            | QA issues                 |
| Orchestrator → Architect | "Update docs"             | Implementation review     |
| Orchestrator → PO        | "Archive feature"         | Complete feature          |
| Orchestrator → UX Designer | "Implement UI/UX change" | UI/UX task description    |
| UX Designer → Tester     | "Test UI changes"         | Implemented UI changes    |
| UX Designer → Orchestrator | "UI/UX implementation complete" | Code changes + verification notes |

**Direct handoffs** (bypass Orchestrator when appropriate):

- Coder ↔ Tester: Rapid fix/retest cycles
- Tester → QA: Direct handoff when all tests pass
- QA → Coder: Direct handoff for minor fixes

## Key Documentation Artifacts

All feature documentation lives in `docs/feature-specs/{feature}/`:

```
docs/feature-specs/{feature}/
├── user-stories.md              # PO: User requirements
├── low-level-design-{feature}.md # Architect: Technical design
├── implementation-plan.md       # Planner: Step-by-step plan
├── implementation-progress.md   # Coder: Progress tracking
├── test-plan.md                 # Tester: Test cases & results
└── QA-report.md                 # QA: Quality review & findings
```

On completion, PO archives the entire feature folder to `docs/feature-specs/_archive/{feature}/`.

## PR Creation & Merge Policy

### PR Creation

The **Coder** agent is responsible for creating Pull Requests using the `create-pull-request` skill. This happens after implementation is complete and pre-handoff checks pass.

### PR Merge Policy

**CRITICAL: Complete these steps BEFORE merging the PR:**

1. ✅ QA approval received
2. ✅ Feature spec archived to `docs/feature-specs/_archive/{feature}/`
3. ✅ Roadmap updated (task marked complete, changelog entry added)

**Only after all three steps are complete:** PR can be merged manually by the repository maintainer.

**No agent has automated merge capabilities.**

**Rationale:** The feature spec documents the implementation. **Archiving must happen before merge** to ensure the documentation matches the merged code. **Roadmap updates must happen before merge** to maintain accurate project tracking. These are pre-merge requirements, not post-merge cleanup tasks.

## Agent Constraints

### Editing Permissions

| Agent | Can Edit Code? | Can Edit Docs? | Can Edit Roadmap? | Can Edit Archived Specs? |
|-------|----------------|----------------|-------------------|--------------------------|
| Orchestrator | ❌ | ❌ | ❌ | ❌ |
| PO | ❌ | ❌ | ✅ (exclusive) | ❌ |
| Architect | ❌ | ❌ | ❌ | ❌ |
| Planner | ❌ | ❌ | ❌ | ❌ |
| Coder | ✅ | ✅ | ❌ | ❌ |
| Tester | ❌ | ❌ | ❌ | ❌ |
| QA | ❌ | ❌ | ❌ | ❌ |
| DocWriter | ❌ | ✅ (only `.md` in `docs/`) | ❌ | ❌ |
| Search | ❌ | ❌ | ❌ | ❌ |
| Cleaner | ✅ | ✅ | ❌ | ❌ |
| UX Designer | ✅ (UI only) | ✅ | ❌ | ❌ |

### Models & Tools

| Agent            | Model (prioritized fallback chain)                  | Tools                                                            | Can Execute? |
| ---------------- | --------------------------------------------------- | ---------------------------------------------------------------- | ------------ |
| **Orchestrator** | Claude Opus 4.6 → Claude Sonnet 4.5 → GPT-5.3-Codex | read, search, agent, web                                         | ❌           |
| **PO**           | Claude Opus 4.6 → Claude Sonnet 4.5 → GPT-5.2       | edit, read, search, web, agent                                   | ❌           |
| **Architect**    | Claude Opus 4.6 → GPT-5.3-Codex → Gemini 2.5 Pro    | search, read, web, agent                                         | ❌           |
| **Planner**      | Claude Opus 4.6 → Claude Sonnet 4.5 → GPT-5.3-Codex | read, search, web, agent                                         | ❌           |
| **Coder**        | GPT-5.3-Codex → GPT-5.2-Codex → Claude Opus 4.6     | vscode, execute, read, edit, search, web, agent, github/\*, todo | ✅           |
| **Tester**       | GPT-5.3-Codex → Claude Opus 4.6 → GPT-5.2           | read, search, execute, agent, playwright/_, github/_, web        | ✅           |
| **QA**           | GPT-5.3-Codex → Claude Opus 4.6 → Gemini 2.5 Pro    | agent, search, read, web                                         | ❌           |
| **DocWriter**    | Claude Opus 4.6 → Claude Sonnet 4.5 → GPT-5.2       | read, search, edit, web, agent                                   | ❌           |
| **Search**       | GPT-5.3-Codex → GPT-5.2-Codex → Claude Opus 4.6     | read, search, web                                                | ❌           |
| **Cleaner**      | GPT-5.3-Codex → Claude Opus 4.6 → Gemini 2.5 Pro    | read, search, edit, execute, agent, todo, web, memory            | ✅           |
| **UX Designer**  | GPT-5.3-Codex → GPT-5.2-Codex → Claude Opus 4.6     | read, search, edit, execute, todo, agent                         | ✅           |

### Subagent Invocation

Each agent can only invoke specific subagents:

| Agent            | Can Invoke                                                            |
| ---------------- | --------------------------------------------------------------------- |
| **Orchestrator** | PO, Architect, Planner, Coder, Tester, QA, DocWriter, Search, Cleaner, UX Designer |
| **PO**           | Orchestrator, Architect, DocWriter, Search                            |
| **Architect**    | Orchestrator, Planner, PO, DocWriter, Search                          |
| **Planner**      | Orchestrator, DocWriter, Search                                       |
| **Coder**        | Orchestrator, Tester, DocWriter, Search                               |
| **Tester**       | Orchestrator, Coder, QA, DocWriter, Search                            |
| **QA**           | Orchestrator, Coder, Architect, DocWriter, Search                     |
| **DocWriter**    | Orchestrator, Search                                                  |
| **Search**       | — (none)                                                              |
| **Cleaner**      | Orchestrator, Search, Architect, Tester, DocWriter                    |
| **UX Designer**  | Orchestrator, Search, Tester, DocWriter                               |

**Notes:**

- Orchestrator can invoke all agents and serves as the central workflow coordinator. All agents can hand back to Orchestrator for routing to the next phase.
- DocWriter can edit markdown documentation in `docs/` but not code files. It is invoked as a subagent by other agents.
- Search is read-only for codebase research. Its 272K context window enables analysis across many files simultaneously.
- Only Coder and Tester have terminal execution access.
- PO can only edit `docs/roadmap.md` and `docs/backlog.md`.
- Cleaner can edit code and docs for cleanup/refactoring only. It cannot add features or change behavior. It has terminal execution access for running `cargo xtask ci-check`.
- UX Designer owns the frontend design system and React components in `ui/` and `sdk-template/ui/`. It follows a required skill invocation order for all UI tasks (see [UX Designer Skill Invocation Order](#ux-designer-skill-invocation-order)).

### Search Delegation Pattern

All specialized agents (except Orchestrator) can invoke the Search agent for deep codebase research. Each agent's instructions include a "Codebase Research" section that specifies:

- **When to delegate** vs. use own search tools
- **How to structure** Search requests (what + where + synthesize)
- **Agent-specific examples** matching their typical research needs

**Rule of thumb:** If the research requires reading >3 files or spans multiple layers, delegate to Search. For quick single-file lookups, use your own tools.

**Search is read-only.** It returns findings and analysis. The invoking agent decides what to do with the results.

### Documentation Delegation Pattern

Four agents (Architect, Planner, Tester, QA) don't have `edit` tools but are responsible for creating documentation artifacts. Each agent's instructions include a "Documentation Delegation" section that specifies:

- **When to delegate** — after generating complete document content
- **Who to delegate to** — DocWriter (already in each agent's `agents:` list)
- **What to pass** — complete markdown content + target filepath

**Rule:** The delegating agent generates ALL content. DocWriter writes the file — it does not author technical documents.

**Composition:** An agent may invoke Search for research AND DocWriter for persistence in the same workflow. Always: Search → generate content → DocWriter.

### UX Designer Skill Invocation Order

For every UI task, the UX Designer must invoke skills in this exact order:

1. **`ui-ux-change-workflow`** — defines and drives the end-to-end UI change path
2. **`design-token-compliance`** — normalizes spacing, color, typography, and interaction states to existing design tokens
3. **`ui-accessibility-review`** — pre-handoff accessibility gate (WCAG-minded keyboard, ARIA, and contrast checks)

> **Exception (narrow):** For tiny copy-only text edits with no layout, style, interaction, or semantic impact, steps 2–3 may be minimized. Step 1 still applies.

---

## When to Invoke Each Agent

- **Start with Orchestrator** when: Beginning a new feature, coordinating multi-phase work, unsure which specialist to use
- **Start with PO** when: Quick roadmap questions, backlog prioritization (no full feature workflow needed)
- **Use Architect** when: Design decisions needed, architectural review, defining boundaries
- **Use Planner** when: Complex feature needs breakdown, multi-step implementation
- **Use Coder** when: Ready to implement, bug fixes, code changes, creating PRs
- **Use Tester** when: Feature ready for testing, runs `cargo xtask ci-check` first, then manual testing
- **Use QA** when: Code review needed, static analysis, quality verification
- **Use DocWriter** when: Documentation needs creating or updating (invoked as subagent by other agents)
- **Use Search** when: Deep codebase research needed, finding patterns across files (invoked as subagent)
- **Use Cleaner** when: Codebase needs cleanup — dead code removal, AI slop cleaning, duplicate consolidation, missed abstraction usage, documentation noise reduction
- **Use UX Designer** when: UI/UX changes are needed — React component work, design system updates, accessibility improvements, layout or visual consistency fixes in `ui/` or `sdk-template/ui/`

## Testing Workflow

The Tester agent uses the following workflow for feature validation:

### Primary Testing Method: `cargo xtask ci-check`

```bash
# Run standard checks (docs, UI build, lint+typecheck, tests — ~1 minute)
cargo xtask ci-check

# Run with auto-fix for linting issues
cargo xtask ci-check --fix

# Full validation (adds template validation + CD dry-run)
cargo xtask ci-check --full   # or -F

# Skip individual phases
cargo xtask ci-check --skip-docs
cargo xtask ci-check --skip-lint
cargo xtask ci-check --skip-tests
cargo xtask ci-check -F --skip-template
cargo xtask ci-check -F --skip-cd
```

This command runs 6 phases:

0. **Documentation**: Link validation (`scripts/check-links.sh`)
1. **UI Dist Build**: Rebuilds `ui/dist` (two-stage: packages in `ui/`, app in `sdk-template/ui/`)
2. **Linting + Type-Checking** (with optional --fix): ESLint, Prettier, `tsc --noEmit`, cargo fmt, clippy
3. **Automated Tests**: Engine (Rust) and UI (Vitest) tests
4. **Template Validation** (`--full` only): Validates CLI-generated projects compile
5. **CD Dry-Run** (`--full` only): Git-based change detection matching CD workflow path filters

### Visual Testing with Playwright MCP

Visual testing is done separately using the **playwright-mcp-ui-testing** skill:

```bash
# 1. Start the dev servers
cargo xtask dev

# 2. Tester agent uses Playwright MCP tools to:
#    - Navigate to http://localhost:5173
#    - Take screenshots
#    - Validate UI appearance
#    - Compare against baselines

# 3. Stop servers when done
pkill -f "cargo xtask dev"
```

**When to use visual testing:**

- UI component changes
- Styling updates
- Layout modifications
- New visual features

### Testing CLI-Generated Plugins

When SDK changes affect generated plugins (templates, engine crates, CLI), you must validate `wavecraft create` produces working projects.

**Standard workflow using `--output` flag:**

```bash
# Generate test plugin into SDK's build directory (gitignored)
# Note: --local-sdk is NOT needed when running via `cargo run` — the CLI
# auto-detects SDK development mode and uses path dependencies automatically.
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin \
  --output target/tmp/test-plugin

# Test the generated plugin
cd target/tmp/test-plugin
cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start --install
```

**Why `--output` is the standard approach:**

| Approach                     | Pros                               | Cons                                  |
| ---------------------------- | ---------------------------------- | ------------------------------------- |
| `--output target/tmp/...`    | Isolated, gitignored, easy cleanup | Slightly longer command               |
| Create in separate directory | Clean separation                   | Must remember to delete after         |
| Create in current directory  | Quick                              | Pollutes SDK repo with test artifacts |

**Test checklist for CLI/template changes:**

1. `wavecraft create` completes without errors
2. `wavecraft start` builds without compile errors (no `include_dir` panics, etc.)
3. **`cargo clippy` passes on generated project** — catch unused imports, dead code warnings
4. `cargo xtask bundle` produces valid plugin bundles
5. Plugin loads in a DAW

**Complete test workflow:**

```bash
# Step 1: Generate test plugin
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin \
  --output target/tmp/test-plugin

# Step 2: Run clippy on generated code (catches template issues)
cd target/tmp/test-plugin/engine
cargo clippy --all-targets -- -D warnings

# Step 3: Verify compile and bundle
cd ..
cargo xtask bundle

# Step 4: Cleanup
cd ../../..
rm -rf target/tmp/test-plugin
```

**Why clippy on generated code is critical:**

The CI pipeline runs `template-validation.yml` which executes clippy on generated templates. If the Tester doesn't run clippy locally, template issues (unused imports, dead code) will only be caught in CI, causing pipeline failures after merge.

### Why Not Docker/act?

The `cargo xtask ci-check` approach is **~26x faster** than running the full CI pipeline via Docker:

| Method                 | Time      | Use Case                                            |
| ---------------------- | --------- | --------------------------------------------------- |
| `cargo xtask ci-check` | ~52s      | Daily testing, pre-push validation                  |
| `act` (Docker CI)      | ~9-12 min | CI performance comparison, debugging GitHub Actions |

Docker-based testing is only needed for:

- Validating CI workflow YAML changes
- Performance benchmarking CI pipeline itself
- Debugging container-specific issues
