# Low-Level Design: Agent Search Delegation

## Overview

This design specifies how to add "Codebase Research" delegation instructions to each specialized agent, enabling them to invoke the Search agent (272K context, GPT-5.2-Codex) for deep codebase analysis instead of doing fragmented multi-tool research themselves.

The change is **instructions-only** — no code, no tooling changes, no modifications to the Search agent itself.

---

## Architectural Decisions

### AD-1: Add a compact section to each agent, not a shared reference

**Decision:** Each agent gets its own inline "Codebase Research" section with role-specific examples. No shared external document is referenced.

**Rationale:**
- Agents are stateless. They can't "look up" a shared reference mid-reasoning without consuming tool calls and context.
- Inline instructions are always in the agent's prompt — zero cost to access, guaranteed to influence behavior.
- Agent-specific examples are more effective than generic ones. An Architect example ("find all IPC implementations") means nothing to a Tester.
- The section is small enough (~20-30 lines) that duplication is cheap.

**Rejected alternative:** Central `docs/architecture/search-delegation.md` referenced by all agents. This requires each agent to read a file before acting — adds latency, consumes context, and may be skipped.

---

### AD-2: Binary decision model (self vs. delegate), not a spectrum

**Decision:** Two clear categories: "use your own tools" or "invoke Search." No intermediate tiers.

**Rationale:**
- Agents make fast binary decisions well. Nuanced scoring rubrics get ignored under context pressure.
- The threshold is clear: **single-location lookups = self, multi-file analysis = delegate.**
- Adding a third category (e.g., "medium research") creates ambiguity and decision paralysis.

---

### AD-3: Search invocations use structured prompts with three required elements

**Decision:** Every Search invocation prompt must include: (1) what to find, (2) where to look, (3) what to synthesize.

**Rationale:**
- Search's effectiveness scales with prompt specificity. Vague prompts ("find parameter code") produce vague results.
- The three elements map to Search's methodology: discover → analyze → synthesize.
- Structured prompts also make Search's output predictable and reusable by the invoking agent.

---

### AD-4: Place section after "Project Context," before task-specific instructions

**Decision:** The "Codebase Research" section is placed immediately after "Project Context" (or equivalent introductory section) and before any task-specific workflow or checklist.

**Rationale:**
- After context: The agent needs to understand the project before understanding when to delegate.
- Before workflow: Research delegation is a pre-task activity. Placing it before "Implementation Workflow" (Coder) or "Analysis Checklists" (QA) ensures agents think about delegation before starting their main work.
- This mirrors how a human would think: "Before I start designing/coding/testing, do I need to understand something first?"

---

### AD-5: Orchestrator is excluded from this change

**Decision:** The Orchestrator agent does NOT get a "Codebase Research" section.

**Rationale:**
- Orchestrator is a pure routing agent. It doesn't perform analysis, research, or reasoning about the codebase.
- It already lists Search in its routing map for direct user requests.
- Adding research instructions would violate the Orchestrator's constraint: "You do NOT make technical judgments."

---

## Section Structure

### Template (agent-agnostic structure)

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Delegate to Search when you need:**
- Multi-file pattern analysis (e.g., "How is X done across the codebase?")
- Cross-layer tracing (e.g., UI → IPC → Engine data flow)
- Dependency mapping (e.g., "What depends on module Y?")
- Convention discovery (e.g., "What's the established pattern for Z?")

**Use your own search tools for:**
- Looking up a specific file or function you already know
- Reading a single document (e.g., a spec or design doc)
- Quick grep for a known string or symbol name

**When invoking Search, always specify:**
1. **What** to find — the pattern, implementation, or concept
2. **Where** to look — specific crates, layers, or file patterns
3. **What to synthesize** — the specific findings you need back

**Example:**
> [Agent-specific example here]
```

### Constraints

| Property | Value |
|----------|-------|
| Max section length | 25-30 lines (excluding examples) |
| Tone | Direct, imperative. Match existing agent instruction style. |
| Examples per agent | 1-2 concrete examples using real Wavecraft patterns |
| Formatting | Same markdown conventions as rest of agent file |

---

## Delegation Decision Matrix

### Decision Rule (universal)

```
IF the research requires reading >3 files to answer
   OR spans multiple layers (Engine + UI, Bridge + Protocol, etc.)
   OR asks "how is X done across the codebase?"
THEN → Invoke Search

ELSE → Use own tools (grep_search, read_file, semantic_search)
```

### Agent-Specific Decision Table

| Agent | Self-Search (own tools) | Delegate to Search |
|-------|------------------------|--------------------|
| **Architect** | Read a specific design doc; check one crate's structure | Find all implementations of a pattern; trace data flows across layers; survey existing abstractions before designing new ones |
| **Planner** | Read a spec or implementation plan; check a file's structure | Map all affected files for a feature; find reusable patterns across crates; identify crosscutting dependencies |
| **Coder** | Look up a function signature; read a specific file; grep for a symbol | Understand a convention used across multiple files; find all places that need updating for a change; discover how similar features are implemented |
| **Tester** | Read a specific test file; check a test plan | Analyze test coverage across crates; find all test patterns for a feature area; identify untested code paths |
| **QA** | Read a specific file for review; check a coding standard | Audit an anti-pattern across the codebase; verify naming consistency; find all error-handling sites for review |
| **PO** | Read roadmap or a spec; check a feature's current state | Assess feature complexity by surveying related code; find all components a proposed feature would affect |
| **DocWriter** | Read a single doc to update; check cross-reference links | Find all code related to a topic (e.g., "all IPC code") for documentation scope; discover undocumented public APIs |

---

## Search Invocation Format

### Prompt Structure

When an agent invokes the Search subagent, the prompt should follow this format:

```
Search the codebase for [WHAT].
Focus on [WHERE].
Synthesize [WHAT TO RETURN].
```

### Concrete Examples by Agent

#### Architect

```
Search the codebase for all parameter synchronization implementations.
Focus on engine/crates/wavecraft-bridge/, engine/crates/wavecraft-protocol/,
and ui/packages/core/src/.
Synthesize: what sync patterns exist, how they handle atomics vs message-passing,
and any inconsistencies between the Rust and TypeScript sides.
```

#### Planner

```
Search the codebase for all files that reference or handle IPC messages.
Focus on engine/crates/ and ui/packages/core/.
Synthesize: a complete map of IPC touchpoints (files and functions),
the message flow from UI to engine, and any shared type definitions.
```

#### Coder

```
Search the codebase for how error handling is implemented across IPC boundaries.
Focus on engine/crates/wavecraft-bridge/src/ and ui/packages/core/src/.
Synthesize: the error types used, how errors propagate from Rust to TypeScript,
and the established pattern I should follow for new error cases.
```

#### Tester

```
Search the codebase for all test files related to parameter handling.
Focus on engine/crates/*/tests/, ui/packages/core/src/**/*.test.*,
and ui/src/test/.
Synthesize: what parameter behaviors are tested, what patterns the tests follow,
and any gaps where parameter edge cases are not covered.
```

#### QA

```
Search the codebase for all uses of unwrap() and expect() in production code paths.
Focus on engine/crates/ (exclude test files).
Synthesize: which uses are in audio-thread code (real-time safety violation),
which are in initialization code (acceptable), and which should be replaced
with proper error handling.
```

#### PO

```
Search the codebase for the current state management architecture.
Focus on engine/crates/wavecraft-core/, engine/crates/wavecraft-bridge/,
and ui/packages/core/src/.
Synthesize: how plugin state is saved/restored, what infrastructure exists
for preset management, and how much work a preset system would require.
```

#### DocWriter

```
Search the codebase for all public APIs in the @wavecraft/core npm package.
Focus on ui/packages/core/src/ and ui/packages/core/package.json.
Synthesize: every exported function, class, hook, and type with a brief description
of what it does, so I can verify documentation completeness.
```

---

## Agent-Specific Section Content

Below is the exact "Codebase Research" section for each agent. These are designed to be inserted directly into the respective `.agent.md` files.

### Architect (`architect.agent.md`)

Insert after the "Project Context" section:

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only research specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Invoke Search before designing** when you need to:
- Survey existing patterns before proposing new abstractions
- Trace data flows across layers (Engine → Bridge → UI)
- Understand all implementations of a concept you're about to redesign
- Map module boundaries and dependency relationships

**Use your own search tools** for quick lookups: reading a single doc, checking a crate's structure, or finding one specific definition.

**When invoking Search, specify:** (1) what pattern/concept to find, (2) which crates or layers to focus on, (3) what synthesis you need (e.g., "list all sync patterns and their tradeoffs").

**Example:** Before designing a new parameter validation layer, invoke Search:
> "Search for all parameter validation and range-checking code across engine/crates/ and ui/packages/core/. Synthesize: where validation currently happens, what patterns are used, and any gaps where invalid values could propagate."
```

### Planner (`planner.agent.md`)

Insert after the "Your Role" section:

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only research specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Invoke Search before planning** when you need to:
- Map all files affected by a feature (dependency graph)
- Find reusable patterns that the implementation plan should follow
- Identify crosscutting concerns that affect multiple plan steps
- Understand the full scope of a refactoring

**Use your own search tools** for quick lookups: reading a spec, checking a file structure, or finding a specific function.

**When invoking Search, specify:** (1) what to map or find, (2) which crates or packages to focus on, (3) what to synthesize (e.g., "list all affected files with their roles").

**Example:** Before planning IPC changes, invoke Search:
> "Search for all files that send or receive IPC messages across engine/crates/wavecraft-bridge/, engine/crates/wavecraft-protocol/, and ui/packages/core/src/. Synthesize: a complete map of IPC touchpoints, message types, and the handler chain from UI to engine."
```

### Coder (`coder.agent.md`)

Insert after the "Project Context" section (before "Coding Principles"):

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only research specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Invoke Search before implementing** when you need to:
- Understand how a pattern is implemented across multiple files before following it
- Find all locations that need updating for a cross-cutting change
- Discover conventions for something you haven't implemented before in this codebase

**Use your own search tools** for quick lookups: finding a function signature, reading a specific file, or grepping for a known symbol.

**When invoking Search, specify:** (1) what pattern or implementation to find, (2) which crates or packages to focus on, (3) what to synthesize (e.g., "the established pattern I should follow").

**Example:** Before adding a new IPC message type, invoke Search:
> "Search for how existing IPC message types are defined and handled across engine/crates/wavecraft-protocol/src/, engine/crates/wavecraft-bridge/src/, and ui/packages/core/src/. Synthesize: the pattern for adding a new message type end-to-end (Rust struct, handler, TypeScript type, client method)."
```

### Tester (`tester.agent.md`)

Insert after the "Project Context" section (before "Workflow"):

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only research specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Invoke Search when testing** to:
- Analyze test coverage across crates or packages for a feature area
- Find all test patterns and utilities to follow established conventions
- Identify untested code paths or missing edge case coverage

**Use your own search tools** for quick lookups: reading a specific test file, checking a test plan, or finding one test function.

**When invoking Search, specify:** (1) what test area to analyze, (2) which test directories to focus on, (3) what to synthesize (e.g., "coverage gaps and untested paths").

**Example:** Before writing tests for metering, invoke Search:
> "Search for all metering-related test files and assertions across engine/crates/wavecraft-metering/tests/, ui/packages/core/src/**/*.test.*, and ui/src/test/. Synthesize: what metering behaviors are tested, what patterns the tests use, and what edge cases are missing."
```

### QA (`QA.agent.md`)

Insert after the "Project Context" section (before "Automated Checks Workflow"):

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only research specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Invoke Search during review** to:
- Audit a pattern or anti-pattern across the entire codebase (not just changed files)
- Verify naming, error handling, or architectural consistency at scale
- Find all instances of a violation category for comprehensive reporting

**Use your own search tools** for quick lookups: reading a specific file for review, checking a coding standard, or finding one definition.

**When invoking Search, specify:** (1) what pattern or anti-pattern to audit, (2) which crates or packages to analyze, (3) what to synthesize (e.g., "all violations with file locations and severity").

**Example:** When reviewing error handling consistency, invoke Search:
> "Search for all error handling patterns across engine/crates/ (excluding test files). Synthesize: which crates use Result vs panic, where unwrap()/expect() appears in production paths, and any inconsistencies with the coding standards."
```

### PO (`PO.agent.md`)

Insert after the "Product Context" section (before "Your Guiding Principles"):

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only research specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Invoke Search when assessing features** to:
- Evaluate feature complexity before prioritization (how much code is involved?)
- Identify what existing infrastructure supports a proposed feature
- Understand technical scope to write informed acceptance criteria

**Use your own search tools** for quick lookups: reading the roadmap, checking a spec, or reviewing a single feature folder.

**When invoking Search, specify:** (1) what capability or infrastructure to assess, (2) which areas of the codebase to survey, (3) what to synthesize (e.g., "existing infrastructure and estimated effort").

**Example:** Before prioritizing a preset management feature, invoke Search:
> "Search for all state save/restore and serialization code across engine/crates/ and ui/packages/core/. Synthesize: what state management infrastructure exists today, how plugin state is persisted, and how much of a preset system is already in place vs needs building."
```

### DocWriter (`docwriter.agent.md`)

Insert after the "Project Documentation Structure" section (before "Documentation Standards"):

```markdown
## Codebase Research

You have access to the **Search agent** — a read-only research specialist with a 272K context window that can analyze 50-100 files simultaneously.

**Invoke Search before writing docs** to:
- Discover all code related to a topic you're documenting (ensure completeness)
- Find undocumented public APIs that need documentation
- Map the scope of a feature across crates and packages

**Use your own search tools** for quick lookups: reading a single doc, checking cross-references, or finding a specific export.

**When invoking Search, specify:** (1) what topic or API surface to discover, (2) which packages or crates to scan, (3) what to synthesize (e.g., "all public exports with descriptions").

**Example:** Before documenting the IPC system, invoke Search:
> "Search for all IPC-related public APIs, types, and message handlers across engine/crates/wavecraft-bridge/, engine/crates/wavecraft-protocol/, and ui/packages/core/src/. Synthesize: every public function, type, and message format with a brief description, so I can ensure the documentation covers all touchpoints."
```

---

## Integration with Existing Instructions

### Insertion Points (per file)

| Agent File | Insert After Section | Insert Before Section |
|------------|---------------------|-----------------------|
| `architect.agent.md` | "Project Context" | "Architectural Principles You Must Enforce" |
| `planner.agent.md` | "Your Role" | "Planning Process" |
| `coder.agent.md` | "Project Context" | "Coding Principles You Must Follow" |
| `tester.agent.md` | "Project Context" | "Workflow" |
| `QA.agent.md` | "Project Context" | "Automated Checks Workflow" |
| `PO.agent.md` | "Product Context" | "Your Guiding Principles" |
| `docwriter.agent.md` | "Project Documentation Structure" | "Documentation Standards" |

### Files NOT Modified

| File | Reason |
|------|--------|
| `orchestrator.agent.md` | Pure router, doesn't do research |
| `search.agent.md` | Already correct; the delegation target doesn't need delegation instructions |

### `agent-development-flow.md` Update

Add a new section after "Subagent Invocation" titled "Search Delegation Pattern":

```markdown
### Search Delegation Pattern

All specialized agents (except Orchestrator) can invoke the Search agent for deep codebase research. Each agent's instructions include a "Codebase Research" section that specifies:

- **When to delegate** vs. use own search tools
- **How to structure** Search requests (what + where + synthesize)
- **Agent-specific examples** matching their typical research needs

**Rule of thumb:** If the research requires reading >3 files or spans multiple layers, delegate to Search. For quick single-file lookups, use your own tools.

**Search is read-only.** It returns findings and analysis. The invoking agent decides what to do with the results.
```

---

## Migration Strategy

### Phase 1: Update Agent Instructions (single PR)

1. Add "Codebase Research" section to all 7 agent files
2. Update `agent-development-flow.md` with Search delegation pattern
3. All changes in one PR — this is a documentation-only change with no risk of breaking running agents

### Phase 2: Validation (next 3-5 features)

1. During the next 3-5 feature development cycles, Orchestrator (or human) observes whether agents invoke Search
2. If agents underuse Search, refine trigger examples to be more specific
3. If agents overuse Search for trivial lookups, tighten the self-search criteria

### Rollback

If the instructions cause problems (e.g., agents waste time on unnecessary Search invocations), the sections can be removed in a single commit with no side effects.

---

## Validation Approach

### Immediate (PR review)

- [ ] Each agent's "Codebase Research" section follows the template structure
- [ ] Examples reference real Wavecraft codebase patterns (not generic placeholders)
- [ ] Section length is ≤30 lines per agent
- [ ] Insertion point doesn't break existing section flow
- [ ] `agent-development-flow.md` is updated

### Ongoing (observational, across features)

| Metric | How to Measure | Target |
|--------|----------------|--------|
| Search invocations per feature | Count Search subagent calls during feature dev cycles | 3-5 per feature (up from ~0-1) |
| Self-search for deep analysis | Observe if agents still do long grep chains for multi-file questions | Rare (quick lookups only) |
| Research quality | Review whether agent outputs (designs, plans, reviews) cite comprehensive codebase evidence | Consistent cross-crate awareness |
| Instruction compliance | Check if agents follow the what/where/synthesize prompt format | High adherence |

### Anti-Patterns to Watch For

| Anti-Pattern | Symptom | Fix |
|--------------|---------|-----|
| Over-delegation | Agent invokes Search for a single-file read | Add more explicit "use own tools" examples |
| Under-delegation | Agent does 10+ grep searches instead of one Search call | Add more explicit trigger scenarios |
| Vague prompts | Agent invokes Search with "find parameter code" | Add prompt structure enforcement to examples |
| Ignored results | Agent invokes Search but doesn't use the findings | Investigate if Search output format needs tuning |

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agents ignore new instructions | Medium | Low | Examples are concrete and match real tasks; iterate if needed |
| Instructions inflate context budget | Low | Low | ~25 lines per agent; negligible vs. total instruction size |
| Search returns low-quality results | Low | Medium | Search agent is already well-specified; prompt structure improves input quality |
| Over-delegation for trivial queries | Medium | Low | Clear "use own tools" criteria; can tighten in Phase 2 |
| Instructions conflict with existing workflow | Very Low | Medium | Section is additive; doesn't modify existing instructions |

---

## Summary of Changes

| File | Change | Lines Added |
|------|--------|-------------|
| `.github/agents/architect.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/planner.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/coder.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/tester.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/QA.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/PO.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/docwriter.agent.md` | Add "Codebase Research" section | ~20 |
| `docs/architecture/agent-development-flow.md` | Add "Search Delegation Pattern" subsection | ~15 |
| **Total** | 8 files modified | ~175 lines |

---

## Related Documents

- [User Stories](./user-stories.md) — PO requirements for this feature
- [High-Level Design](../../architecture/high-level-design.md) — System architecture
- [Agent Development Flow](../../architecture/agent-development-flow.md) — Agent roles and handoffs
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
