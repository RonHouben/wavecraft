---
applyTo: '**'
---

# MOST Important guidelines

The following are the MOST important guidelines to follow when editing files in this repository:

- do not edit the contents of files under the `/docs/feature-specs/_archive/` directory. They are kept for historical reference only. Moving completed feature spec folders into `_archive` is allowed, but once archived the files should not be modified.
- ONLY the Product Owner agent is allowed to edit the roadmap file located at `/docs/roadmap.md`! When any other agent needs changes to the roadmap, they must hand off to the Product Owner agent.
- Before making changes in the code, check the #file:../docs/architecture/coding-standards.md file for relevant coding standards and navigation to language-specific guides.
- For understanding the overall project architecture, refer to #file:../docs/architecture/high-level-design.md (overview with links to detailed topic docs).
- For understanding the audio input via WASM architecture, refer to #file:../docs/feature-specs/audio-input-via-wasm/high-level-design.md for the design overview, tiered backend system, and parameter ownership model.
- Always keep the #tool:todo list up to date with relevant tasks and their statuses.

---

# Agent Development Flow

This project uses specialized agents with distinct responsibilities that hand off to each other throughout the development lifecycle.

**📖 For the complete agent development flow, roles, handoffs, and diagrams, always refer to:**
**[docs/architecture/agent-development-flow.md](../docs/architecture/agent-development-flow.md)**

That document is the single source of truth for:

- Agent roles and responsibilities
- Standard feature development flow diagram
- Handoff triggers between agents
- Key documentation artifacts
- Agent constraints (code editing, roadmap access, etc.)
- When to invoke each agent

---

# Codebase Research Guidelines

These guidelines apply to all specialized agents (Architect, Planner, Coder, Tester, QA, DocWriter, PO) that need to research the codebase. The Orchestrator does not perform research itself but may delegate to Search.

## The Search Agent

The **Search agent** is a dedicated research specialist with a **272K context window** that can analyze 50-100 files simultaneously. It is designed for deep, exploratory codebase research.

### When to Delegate to Search (DEFAULT)

**Delegate to Search by default for any research task.** This preserves your context window for your specialized work (planning, coding, testing, etc.).

**Always delegate when:**

- You need to find, locate, or survey code/docs and don't already know the exact file path
- Exploratory search where you don't know which files contain the answer
- Understanding how a pattern is implemented across multiple files
- Finding all locations affected by a feature or refactoring
- Discovering established conventions or patterns to follow
- Identifying crosscutting concerns, dependencies, or architectural constraints
- Any research spanning 2+ crates or packages

**How to invoke Search effectively:**

Specify three things in your delegation:

1. **What** to find or map (pattern, implementation, dependencies, etc.)
2. **Where** to focus (which crates, packages, directories)
3. **What to synthesize** (summary, list of files with roles, pattern to follow, etc.)

**Example delegations:**

- **Architect:** "Search for all parameter validation patterns across engine/crates/wavecraft-protocol/ and engine/crates/wavecraft-bridge/. Synthesize: how parameters are validated and where validation should occur for a new parameter type."

- **Planner:** "Search for all files affected by IPC changes across engine/crates/wavecraft-bridge/, engine/crates/wavecraft-protocol/, and ui/packages/core/src/. Synthesize: list all affected files with their roles in the IPC flow."

- **Coder:** "Search for how existing IPC message types are defined and handled across engine/crates/wavecraft-protocol/src/, engine/crates/wavecraft-bridge/src/, and ui/packages/core/src/. Synthesize: the pattern for adding a new message type end-to-end (Rust struct, handler, TypeScript type, client method)."

- **Tester:** "Search for all metering-related test files and assertions across engine/crates/wavecraft-metering/tests/, ui/packages/core/src/\*_/_.test.\*, and ui/test/. Synthesize: what metering behaviors are tested, what patterns the tests use, and what edge cases are missing."

- **QA:** "Search for all error handling patterns across engine/crates/ (excluding test files). Synthesize: which crates use Result vs panic, where unwrap()/expect() appears in production paths, and any inconsistencies with the coding standards."

- **DocWriter:** "Search for all documentation files that reference the old parameter system in docs/ and README files. Synthesize: list of files that need updates with current content context."

- **PO:** "Search for all user-facing feature implementations in ui/packages/components/src/ and engine/crates/wavecraft-nih_plug/. Synthesize: what user-visible features exist and how they're exposed in the UI and plugin interface."

### When to Use Your Own Tools (EXCEPTION)

Only use your own `read` and `search` tools when you **already know the exact file path or symbol name**. Do NOT use your own tools for exploratory research — that is Search's job.

**Acceptable own-tool usage:**

- Reading a file you're about to edit (you know the path from the plan)
- Looking up a specific symbol definition (you know the name)
- Checking a specific file mentioned in a plan or error message
- Quick reads of 1-2 files when you have exact paths

### Composed Workflows

You may invoke Search for research AND DocWriter for file creation in the same workflow:

**Search → Analyze → DocWriter** — Research, synthesize findings, then persist documentation

**Example:** Planner creates implementation plan:

1. Invoke Search to map all affected files
2. Use findings to write detailed implementation plan
3. Invoke DocWriter to persist `implementation-plan.md`
