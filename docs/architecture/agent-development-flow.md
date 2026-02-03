# Agent Development Flow

This project uses specialized agents with distinct responsibilities that hand off to each other throughout the development lifecycle.

## Agent Roles

| Agent | Role | Key Outputs |
|-------|------|-------------|
| **PO** (Product Owner) | Owns product vision, roadmap, feature prioritization, and **version decisions** | User stories (incl. target version), `docs/roadmap.md` |
| **Architect** | Designs system architecture, enforces technical constraints | Low-level designs in `docs/feature-specs/{feature}/` |
| **Planner** | Creates detailed implementation plans | `docs/feature-specs/{feature}/implementation-plan.md` |
| **Coder** | Implements features, writes production code, **bumps version per user stories** | Code changes, PRs |
| **Tester** | Runs local CI pipeline, executes manual tests, **verifies version display** | `docs/feature-specs/{feature}/test-plan.md` |
| **QA** | Static analysis, code quality verification | QA reports |

## Standard Feature Development Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FEATURE DEVELOPMENT FLOW                         │
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
│ Coder  │  Implementation + PR                                            
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
```

## Handoff Summary

| From | To | Trigger | What Gets Passed |
|------|----|---------|------------------|
| PO → Architect | "Create low level design" | Feature requirements, user stories (incl. target version) |
| Architect → Planner | "Create implementation plan" | Low-level design document |
| Planner → Coder | "Start Implementation" | Implementation plan |
| Coder → Tester | "Test Implementation" | Completed implementation |
| Tester → Coder | "Fix Issues" | Test failures documented in test-plan.md |
| Tester → QA | "Run QA" | All tests passing |
| QA → Coder | "Fix findings" | QA report with severity/location |
| Coder → Tester | "Re-test" | QA findings fixed |
| QA → Architect | "Update architectural Docs" | No QA issues, implementation review |
| Architect → PO | "Update roadmap" | Architecture docs updated |
| **PO** | — | **Archive & Merge** | Archive feature spec, update roadmap, **then** merge PR |

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

## PR Merge Policy

**CRITICAL: PRs must NOT be merged until the PO has completed the following:**

1. ✅ QA approval received
2. ✅ Feature spec archived to `docs/feature-specs/_archive/{feature}/`
3. ✅ Roadmap updated (task marked complete, changelog entry added)
4. ✅ Then and only then: PR can be merged

**Rationale:** The feature spec documents the implementation. Archiving before merge ensures the documentation matches the merged code. Updating the roadmap before merge ensures accurate project tracking.

## Agent Constraints

| Agent | Can Edit Code? | Can Edit Roadmap? | Can Edit Archived Specs? |
|-------|----------------|-------------------|--------------------------|
| PO | ❌ | ✅ (exclusive) | ❌ |
| Architect | ❌ | ❌ | ❌ |
| Planner | ❌ | ❌ | ❌ |
| Coder | ✅ | ❌ | ❌ |
| Tester | ❌ | ❌ | ❌ |
| QA | ❌ | ❌ | ❌ |

## When to Invoke Each Agent

- **Start with PO** when: New feature request, prioritization question, roadmap update needed
- **Use Architect** when: Design decisions needed, architectural review, defining boundaries
- **Use Planner** when: Complex feature needs breakdown, multi-step implementation
- **Use Coder** when: Ready to implement, bug fixes, code changes
- **Use Tester** when: Feature ready for testing, runs `cargo xtask check` first, then manual testing
- **Use QA** when: Code review needed, static analysis, quality verification

## Testing Workflow

The Tester agent uses the following workflow for feature validation:

### Primary Testing Method: `cargo xtask check`

```bash
# Run all checks locally (fast, ~1 minute)
cargo xtask check

# Run with auto-fix for linting issues
cargo xtask check --fix

# Skip certain phases
cargo xtask check --skip-lint
cargo xtask check --skip-tests
```

This command runs:
1. **Linting** (with optional --fix): ESLint, Prettier, cargo fmt, clippy
2. **Automated Tests**: Engine (Rust) and UI (Vitest) tests

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

### Why Not Docker/act?

The `cargo xtask check` approach is **~26x faster** than running the full CI pipeline via Docker:

| Method | Time | Use Case |
|--------|------|----------|
| `cargo xtask check` | ~52s | Daily testing, pre-push validation |
| `act` (Docker CI) | ~9-12 min | CI performance comparison, debugging GitHub Actions |

Docker-based testing is only needed for:
- Validating CI workflow YAML changes
- Performance benchmarking CI pipeline itself
- Debugging container-specific issues
