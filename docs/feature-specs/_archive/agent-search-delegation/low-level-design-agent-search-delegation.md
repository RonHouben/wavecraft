# Low-Level Design: Agent Delegation Patterns

## Overview

This design specifies two delegation patterns for specialized agents:

1. **Search Delegation** — How agents invoke the Search agent (272K context, GPT-5.2-Codex) for deep codebase analysis instead of doing fragmented multi-tool research themselves.
2. **Documentation Delegation** — How agents without `edit` tools invoke DocWriter to persist documentation artifacts they generate.

Both patterns are **instructions-only** — no code, no tooling changes, no modifications to the target agents (Search, DocWriter) themselves.

### Why Two Patterns in One Design

These patterns share a common shape: an agent recognizes its own tool limitations, generates content or a structured request, and delegates the mechanical action to a specialist. They also **compose** — a single workflow step may require both (e.g., Architect invokes Search for research, then invokes DocWriter to save the design). Designing them together ensures consistent invocation conventions and avoids conflicting instructions.

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

### AD-6: Documentation delegation uses content-first invocation, not filepath-first

**Decision:** When an agent invokes DocWriter, it passes the **complete markdown content first**, then the target filepath. DocWriter's job is to write the file, not to generate the content.

**Rationale:**
- The invoking agent (Architect, Planner, etc.) is the domain expert. It knows what the document should contain.
- DocWriter's value is in file creation mechanics (`edit` tool access) and enforcing formatting standards — not in generating domain-specific technical content.
- Content-first invocation prevents DocWriter from re-interpreting or diluting the agent's intent.
- This mirrors how humans work: "Here's my document — please save it to this path."

**Rejected alternative:** Pass a brief outline and let DocWriter flesh it out. This shifts domain authority away from the specialist agent, introduces interpretation errors, and wastes DocWriter's context on content it isn't qualified to create.

---

### AD-7: Documentation delegation section is placed after "Codebase Research"

**Decision:** The "Documentation Delegation" section follows immediately after "Codebase Research" in each agent's instructions.

**Rationale:**
- Natural workflow order: research first, then produce and save output.
- Groups all delegation patterns together — agents learn "when to call for help" in one place.
- Avoids scattering delegation instructions across workflow steps.

---

### AD-8: Only agents WITHOUT `edit` tools get documentation delegation instructions

**Decision:** Only Architect, Planner, Tester, and QA receive the "Documentation Delegation" section. Coder, PO, and DocWriter do not.

**Rationale:**
- **Coder** has `edit` tools — can write files directly.
- **PO** has `edit` tools — can write roadmap directly.
- **DocWriter** IS the delegation target — doesn't delegate to itself.
- Adding unnecessary instructions wastes context and creates confusion.

---

## Part 1: Search Delegation Pattern

### Section Structure

#### Template (agent-agnostic structure)

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

#### Constraints

| Property | Value |
|----------|-------|
| Max section length | 25-30 lines (excluding examples) |
| Tone | Direct, imperative. Match existing agent instruction style. |
| Examples per agent | 1-2 concrete examples using real Wavecraft patterns |
| Formatting | Same markdown conventions as rest of agent file |

---

### Delegation Decision Matrix

#### Decision Rule (universal)

```
IF the research requires reading >3 files to answer
   OR spans multiple layers (Engine + UI, Bridge + Protocol, etc.)
   OR asks "how is X done across the codebase?"
THEN → Invoke Search

ELSE → Use own tools (grep_search, read_file, semantic_search)
```

#### Agent-Specific Decision Table

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

### Search Invocation Format

#### Prompt Structure

When an agent invokes the Search subagent, the prompt should follow this format:

```
Search the codebase for [WHAT].
Focus on [WHERE].
Synthesize [WHAT TO RETURN].
```

#### Concrete Examples by Agent

##### Architect

```
Search the codebase for all parameter synchronization implementations.
Focus on engine/crates/wavecraft-bridge/, engine/crates/wavecraft-protocol/,
and ui/packages/core/src/.
Synthesize: what sync patterns exist, how they handle atomics vs message-passing,
and any inconsistencies between the Rust and TypeScript sides.
```

##### Planner

```
Search the codebase for all files that reference or handle IPC messages.
Focus on engine/crates/ and ui/packages/core/.
Synthesize: a complete map of IPC touchpoints (files and functions),
the message flow from UI to engine, and any shared type definitions.
```

##### Coder

```
Search the codebase for how error handling is implemented across IPC boundaries.
Focus on engine/crates/wavecraft-bridge/src/ and ui/packages/core/src/.
Synthesize: the error types used, how errors propagate from Rust to TypeScript,
and the established pattern I should follow for new error cases.
```

##### Tester

```
Search the codebase for all test files related to parameter handling.
Focus on engine/crates/*/tests/, ui/packages/core/src/**/*.test.*,
and ui/src/test/.
Synthesize: what parameter behaviors are tested, what patterns the tests follow,
and any gaps where parameter edge cases are not covered.
```

##### QA

```
Search the codebase for all uses of unwrap() and expect() in production code paths.
Focus on engine/crates/ (exclude test files).
Synthesize: which uses are in audio-thread code (real-time safety violation),
which are in initialization code (acceptable), and which should be replaced
with proper error handling.
```

##### PO

```
Search the codebase for the current state management architecture.
Focus on engine/crates/wavecraft-core/, engine/crates/wavecraft-bridge/,
and ui/packages/core/src/.
Synthesize: how plugin state is saved/restored, what infrastructure exists
for preset management, and how much work a preset system would require.
```

##### DocWriter

```
Search the codebase for all public APIs in the @wavecraft/core npm package.
Focus on ui/packages/core/src/ and ui/packages/core/package.json.
Synthesize: every exported function, class, hook, and type with a brief description
of what it does, so I can verify documentation completeness.
```

---

## Part 2: Documentation Delegation Pattern

### The Problem

Four agents are responsible for creating key documentation artifacts:

| Agent | Artifact | Target Path |
|-------|----------|-------------|
| **Architect** | Low-level design | `docs/feature-specs/{feature}/low-level-design-{feature}.md` |
| **Planner** | Implementation plan | `docs/feature-specs/{feature}/implementation-plan.md` |
| **Tester** | Test plan | `docs/feature-specs/{feature}/test-plan.md` |
| **QA** | QA report | `docs/feature-specs/{feature}/QA-report.md` |

None of these agents have the `edit` tool. They can read and search, but cannot create or modify files. Today, this means their documentation output exists only in conversation context — it is not persisted to disk unless a human copies it or the Coder is asked to save it.

### The Solution

Each of these four agents delegates file creation to **DocWriter**, which has `edit` tools and is already in their `agents:` invocation list. The delegating agent generates the **complete content**, then invokes DocWriter with a structured request containing the content and target filepath.

### Section Structure

#### Template (for agents without `edit` tools)

```markdown
## Documentation Delegation

You do NOT have `edit` tools. To persist documentation artifacts, invoke **DocWriter** as a subagent.

**Your responsibility:** Generate the complete document content. You are the domain expert — DocWriter writes files, it does not create content for you.

**When to invoke DocWriter:**
- After completing your analysis/design, when you have a full document ready to save
- When updating an existing document with new findings

**Invocation format:**
> Write the following content to `docs/feature-specs/{feature}/{filename}.md`:
>
> [complete markdown content here]

**What NOT to do:**
- Don't pass an outline and ask DocWriter to "flesh it out"
- Don't skip file creation — your work must be persisted, not left in conversation
- Don't ask Coder to save documentation (Coder's role is code, not docs)
```

#### Constraints

| Property | Value |
|----------|-------|
| Max section length | 15-20 lines |
| Tone | Direct, imperative. Match existing agent instruction style. |
| Examples per agent | 1 concrete example matching the agent's primary artifact |
| Placement | Immediately after "Codebase Research" section |

---

### Documentation Delegation Decision Matrix

#### Decision Rule

```
IF you have generated a complete document that should be saved to docs/
   AND you do NOT have `edit` tools
THEN → Invoke DocWriter with complete content + target filepath

IF you need to UPDATE an existing document
THEN → Invoke DocWriter with the updated content + target filepath

IF you only need to OUTPUT content in conversation (no file needed)
THEN → No delegation needed
```

#### Agent-Specific Documentation Table

| Agent | Primary Artifact | When to Invoke DocWriter | What to Pass |
|-------|-----------------|--------------------------|--------------|
| **Architect** | Low-level design | After design decisions are finalized | Complete LLD markdown + `docs/feature-specs/{feature}/low-level-design-{feature}.md` |
| **Planner** | Implementation plan | After plan steps are defined and ordered | Complete plan markdown + `docs/feature-specs/{feature}/implementation-plan.md` |
| **Tester** | Test plan | After test cases are written and structured | Complete test plan markdown + `docs/feature-specs/{feature}/test-plan.md` |
| **QA** | QA report | After findings are analyzed and categorized | Complete QA report markdown + `docs/feature-specs/{feature}/QA-report.md` |

---

### DocWriter Invocation Format

#### Prompt Structure (for documentation delegation)

When an agent invokes DocWriter to persist a document:

```
Write the following content to `[TARGET_FILEPATH]`:

[COMPLETE MARKDOWN CONTENT]
```

The invocation must include:
1. **Target filepath** — exact path in `docs/` where the file should be created or updated
2. **Complete content** — the full markdown document, ready to write as-is

DocWriter should NOT rewrite or restructure the content. It may apply minor formatting fixes (e.g., trailing newline, consistent header levels) per its documentation standards.

#### Concrete Examples by Agent

##### Architect

```
Write the following content to
`docs/feature-specs/parameter-validation/low-level-design-parameter-validation.md`:

# Low-Level Design: Parameter Validation

## Overview
[...full design content generated by Architect...]

## Architectural Decisions
[...AD-1, AD-2, etc...]
```

##### Planner

```
Write the following content to
`docs/feature-specs/parameter-validation/implementation-plan.md`:

# Implementation Plan: Parameter Validation

## Steps
[...full plan content generated by Planner...]

## Estimates
[...per-step estimates...]
```

##### Tester

```
Write the following content to
`docs/feature-specs/parameter-validation/test-plan.md`:

# Test Plan: Parameter Validation

## Test Cases
[...full test cases generated by Tester...]

## Coverage Matrix
[...matrix content...]
```

##### QA

```
Write the following content to
`docs/feature-specs/parameter-validation/QA-report.md`:

# QA Report: Parameter Validation

## Findings
[...categorized findings generated by QA...]

## Recommendations
[...recommendations...]
```

---

### Composing Both Patterns

An agent may need **both** Search delegation and Documentation delegation in a single workflow. This is expected and natural.

#### Typical Composed Workflow

```
┌─────────────┐     invoke      ┌──────────┐
│   Agent      │ ──────────────►│  Search   │
│ (Architect)  │◄────────────── │           │
│              │   findings     └──────────┘
│              │
│  [uses findings to produce design]
│              │
│              │     invoke      ┌──────────┐
│              │ ──────────────►│ DocWriter │
│              │                │           │
└─────────────┘                └──────────┘
                               writes file to disk
```

#### Sequencing Rules

1. **Search first, DocWriter second.** Research informs content. Never invoke DocWriter before the content is finalized.
2. **Independent invocations.** Search and DocWriter are separate subagent calls. Do not combine them into one invocation.
3. **Agent retains authority.** The delegating agent reviews Search findings and decides what goes into the document. DocWriter receives finished content, not raw search output.

#### Example: Architect Composing Both

```
Step 1 — Research:
  Invoke Search:
    "Search for all parameter validation code across engine/crates/ and
    ui/packages/core/. Synthesize: where validation happens, patterns used,
    and gaps."

Step 2 — Design (agent's own reasoning):
  Use Search findings to write the low-level design document content.

Step 3 — Persist:
  Invoke DocWriter:
    "Write the following content to
    docs/feature-specs/parameter-validation/low-level-design-parameter-validation.md:
    [complete LLD content]"
```

---

## Agent-Specific Section Content

Below is the exact content for each agent. These are designed to be inserted directly into the respective `.agent.md` files.

### Architect (`architect.agent.md`)

#### Codebase Research (insert after "Project Context")

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

#### Documentation Delegation (insert after "Codebase Research")

```markdown
## Documentation Delegation

You do NOT have `edit` tools. To save your low-level design documents, invoke **DocWriter** as a subagent.

**Your responsibility:** Generate the complete design document content. You are the architecture authority — DocWriter writes files, it does not create designs for you.

**When to invoke DocWriter:**
- After finalizing a low-level design, invoke DocWriter to write it to disk
- After updating architectural decisions that require document changes

**Invocation format:**
> Write the following content to `docs/feature-specs/{feature}/low-level-design-{feature}.md`:
>
> [complete low-level design markdown]

**Composed workflow:** If you invoked Search for research, use those findings to write your design, THEN invoke DocWriter to persist it. Search → Design → DocWriter.
```

### Planner (`planner.agent.md`)

#### Codebase Research (insert after "Your Role")

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

#### Documentation Delegation (insert after "Codebase Research")

```markdown
## Documentation Delegation

You do NOT have `edit` tools. To save your implementation plans, invoke **DocWriter** as a subagent.

**Your responsibility:** Generate the complete implementation plan content. You are the planning authority — DocWriter writes files, it does not create plans for you.

**When to invoke DocWriter:**
- After finalizing the implementation plan with all steps, estimates, and dependencies
- After revising a plan based on new information

**Invocation format:**
> Write the following content to `docs/feature-specs/{feature}/implementation-plan.md`:
>
> [complete implementation plan markdown]

**Composed workflow:** If you invoked Search for scope analysis, use those findings to write your plan, THEN invoke DocWriter to persist it. Search → Plan → DocWriter.
```

### Coder (`coder.agent.md`)

#### Codebase Research (insert after "Project Context", before "Coding Principles")

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

*Note: Coder has `edit` tools — no Documentation Delegation section needed.*

### Tester (`tester.agent.md`)

#### Codebase Research (insert after "Project Context", before "Workflow")

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

#### Documentation Delegation (insert after "Codebase Research")

```markdown
## Documentation Delegation

You do NOT have `edit` tools. To save your test plans, invoke **DocWriter** as a subagent.

**Your responsibility:** Generate the complete test plan content. You are the testing authority — DocWriter writes files, it does not create test plans for you.

**When to invoke DocWriter:**
- After writing all test cases, coverage matrices, and test results
- After updating a test plan with new findings or retest results

**Invocation format:**
> Write the following content to `docs/feature-specs/{feature}/test-plan.md`:
>
> [complete test plan markdown]

**Composed workflow:** If you invoked Search for coverage analysis, use those findings to write your test plan, THEN invoke DocWriter to persist it. Search → Test Plan → DocWriter.
```

### QA (`QA.agent.md`)

#### Codebase Research (insert after "Project Context", before "Automated Checks Workflow")

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

#### Documentation Delegation (insert after "Codebase Research")

```markdown
## Documentation Delegation

You do NOT have `edit` tools. To save your QA reports, invoke **DocWriter** as a subagent.

**Your responsibility:** Generate the complete QA report content. You are the quality authority — DocWriter writes files, it does not create QA reports for you.

**When to invoke DocWriter:**
- After completing your analysis and categorizing all findings
- After updating a report with fixes verified or new issues found

**Invocation format:**
> Write the following content to `docs/feature-specs/{feature}/QA-report.md`:
>
> [complete QA report markdown]

**Composed workflow:** If you invoked Search for codebase-wide auditing, use those findings to write your QA report, THEN invoke DocWriter to persist it. Search → QA Report → DocWriter.
```

### PO (`PO.agent.md`)

#### Codebase Research (insert after "Product Context", before "Your Guiding Principles")

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

*Note: PO has `edit` tools — no Documentation Delegation section needed.*

### DocWriter (`docwriter.agent.md`)

#### Codebase Research (insert after "Project Documentation Structure", before "Documentation Standards")

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

*Note: DocWriter IS the delegation target for documentation — no Documentation Delegation section needed.*

---

## Integration with Existing Instructions

### Insertion Points (per file)

| Agent File | Codebase Research: Insert After | Codebase Research: Insert Before | Doc Delegation: Insert After | Doc Delegation: Insert Before |
|------------|--------------------------------|----------------------------------|------------------------------|-------------------------------|
| `architect.agent.md` | "Project Context" | "Architectural Principles You Must Enforce" | "Codebase Research" | "Architectural Principles You Must Enforce" |
| `planner.agent.md` | "Your Role" | "Planning Process" | "Codebase Research" | "Planning Process" |
| `coder.agent.md` | "Project Context" | "Coding Principles You Must Follow" | — (has `edit` tools) | — |
| `tester.agent.md` | "Project Context" | "Workflow" | "Codebase Research" | "Workflow" |
| `QA.agent.md` | "Project Context" | "Automated Checks Workflow" | "Codebase Research" | "Automated Checks Workflow" |
| `PO.agent.md` | "Product Context" | "Your Guiding Principles" | — (has `edit` tools) | — |
| `docwriter.agent.md` | "Project Documentation Structure" | "Documentation Standards" | — (is the target) | — |

### Files NOT Modified

| File | Reason |
|------|--------|
| `orchestrator.agent.md` | Pure router, doesn't do research or create documentation |
| `search.agent.md` | Search delegation target, doesn't need delegation instructions |

### `agent-development-flow.md` Updates

#### 1. Update "Search Delegation Pattern" subsection (already exists)

No changes needed — the existing subsection is correct.

#### 2. Add new "Documentation Delegation Pattern" subsection after "Search Delegation Pattern"

```markdown
### Documentation Delegation Pattern

Four agents (Architect, Planner, Tester, QA) don't have `edit` tools but are responsible for creating documentation artifacts. Each agent's instructions include a "Documentation Delegation" section that specifies:

- **When to delegate** — after generating complete document content
- **Who to delegate to** — DocWriter (already in each agent's `agents:` list)
- **What to pass** — complete markdown content + target filepath

**Rule:** The delegating agent generates ALL content. DocWriter writes the file — it does not author technical documents.

**Composition:** An agent may invoke Search for research AND DocWriter for persistence in the same workflow. Always: Search → generate content → DocWriter.
```

---

## Migration Strategy

### Phase 1: Update Agent Instructions (single PR)

1. Add "Codebase Research" section to all 7 agent files
2. Add "Documentation Delegation" section to 4 agent files (Architect, Planner, Tester, QA)
3. Update `agent-development-flow.md` with both delegation pattern subsections
4. All changes in one PR — this is a documentation-only change with no risk of breaking running agents

### Phase 2: Validation (next 3-5 features)

1. During the next 3-5 feature development cycles, observe whether agents:
   - Invoke Search appropriately (not over/under-delegating)
   - Invoke DocWriter to persist their documentation artifacts
   - Follow the content-first invocation format
2. If agents skip DocWriter and leave content in conversation, strengthen the "must persist" language
3. If agents pass outlines instead of complete content, add explicit examples of correct vs. incorrect invocations

### Rollback

If the instructions cause problems, the sections can be removed in a single commit with no side effects. The two patterns are independent — one can be rolled back without affecting the other.

---

## Validation Approach

### Immediate (PR review)

- [ ] Each agent's "Codebase Research" section follows the template structure
- [ ] Each applicable agent's "Documentation Delegation" section follows the template structure
- [ ] Examples reference real Wavecraft codebase patterns (not generic placeholders)
- [ ] Section length is ≤30 lines for Research, ≤20 lines for Documentation per agent
- [ ] Insertion points don't break existing section flow
- [ ] `agent-development-flow.md` is updated with both delegation patterns

### Ongoing (observational, across features)

| Metric | How to Measure | Target |
|--------|----------------|--------|
| Search invocations per feature | Count Search subagent calls during feature dev cycles | 3-5 per feature (up from ~0-1) |
| DocWriter invocations per feature | Count DocWriter subagent calls by non-edit agents | 1-2 per feature (up from 0) |
| Self-search for deep analysis | Observe if agents still do long grep chains for multi-file questions | Rare (quick lookups only) |
| Doc persistence rate | Check if agent-generated docs are saved to disk vs left in conversation | 100% (all artifacts persisted) |
| Research quality | Review whether agent outputs cite comprehensive codebase evidence | Consistent cross-crate awareness |
| Content-first compliance | Check if DocWriter receives complete content vs outlines | High adherence |
| Instruction compliance | Check if agents follow the what/where/synthesize prompt format (Search) | High adherence |

### Anti-Patterns to Watch For

| Anti-Pattern | Symptom | Affected Pattern | Fix |
|--------------|---------|------------------|-----|
| Over-delegation to Search | Agent invokes Search for a single-file read | Search | Add more explicit "use own tools" examples |
| Under-delegation to Search | Agent does 10+ grep searches instead of one Search call | Search | Add more explicit trigger scenarios |
| Vague Search prompts | Agent invokes Search with "find parameter code" | Search | Add prompt structure enforcement to examples |
| Ignored Search results | Agent invokes Search but doesn't use the findings | Search | Investigate if Search output format needs tuning |
| Skipped DocWriter | Agent generates content but doesn't invoke DocWriter to save it | Documentation | Strengthen "must persist" language; add reminder at end of workflow |
| Outline delegation | Agent passes a brief outline and tells DocWriter to write the doc | Documentation | Add explicit anti-example showing incorrect vs correct invocation |
| Delegating to Coder | Agent asks Coder to save documentation instead of DocWriter | Documentation | Add explicit "Don't ask Coder to save docs" instruction |
| Double-writing | Agent generates content AND tries to use non-existent edit tools | Documentation | Ensure agent instructions clearly state tool limitations |

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agents ignore new instructions | Medium | Low | Examples are concrete and match real tasks; iterate if needed |
| Instructions inflate context budget | Low | Low | ~25 lines Research + ~15 lines Documentation per agent; negligible vs. total instruction size |
| Search returns low-quality results | Low | Medium | Search agent is already well-specified; prompt structure improves input quality |
| Over-delegation for trivial queries | Medium | Low | Clear "use own tools" criteria; can tighten in Phase 2 |
| Instructions conflict with existing workflow | Very Low | Medium | Sections are additive; don't modify existing instructions |
| Agents skip DocWriter persistence | Medium | Medium | Explicit "must persist" language; Orchestrator can verify file existence before handoff |
| DocWriter rewrites agent content | Low | Medium | Clear instruction: "write as-is, minor formatting only" |
| Large documents exceed DocWriter context | Low | Low | Documents are typically <500 lines; well within context limits |

---

## Summary of Changes

| File | Change | Lines Added |
|------|--------|-------------|
| `.github/agents/architect.agent.md` | Add "Codebase Research" + "Documentation Delegation" sections | ~35 |
| `.github/agents/planner.agent.md` | Add "Codebase Research" + "Documentation Delegation" sections | ~35 |
| `.github/agents/coder.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/tester.agent.md` | Add "Codebase Research" + "Documentation Delegation" sections | ~35 |
| `.github/agents/QA.agent.md` | Add "Codebase Research" + "Documentation Delegation" sections | ~35 |
| `.github/agents/PO.agent.md` | Add "Codebase Research" section | ~20 |
| `.github/agents/docwriter.agent.md` | Add "Codebase Research" section | ~20 |
| `docs/architecture/agent-development-flow.md` | Add "Search Delegation Pattern" + "Documentation Delegation Pattern" subsections | ~25 |
| **Total** | **8 files modified** | **~225 lines** |

---

## Related Documents

- [User Stories](./user-stories.md) — PO requirements for this feature
- [High-Level Design](../../architecture/high-level-design.md) — System architecture
- [Agent Development Flow](../../architecture/agent-development-flow.md) — Agent roles and handoffs
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
