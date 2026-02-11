# Implementation Plan: Agent Delegation Patterns

## 1. Overview

This plan details the implementation of two key agent delegation patterns: **Search Delegation** and **Documentation Delegation**. The goal is to standardize how specialized agents perform deep codebase research (by delegating to the Search agent) and create documentation artifacts (by delegating to the DocWriter agent). This involves updating the instruction files for 7 agents plus 1 architecture document (8 files total) to include these standardized delegation patterns.

## 2. Requirements

- **R1. Search Delegation:** Update 7 agent instruction files to include a "Codebase Research" section defining when and how to delegate research tasks to the Search agent.
- **R2. Documentation Delegation:** Update 4 agent instruction files (Architect, Planner, Tester, QA) to include a "Documentation Delegation" section defining how to request document creation from the DocWriter agent.
- **R3. Architecture Documentation:** Update `docs/architecture/agent-development-flow.md` to document both delegation patterns.
- **R4. Accurate Content:** Ensure added sections match the low-level design (~35 lines for agents with both sections, ~20 lines for agents with Research only).
- **R5. Single PR:** All file modifications delivered in a single, cohesive Pull Request.

## 3. Affected Files (8 total)

### Agent Instruction Files (7)

1. `.github/agents/architect.agent.md` — Add both sections (~35 lines)
2. `.github/agents/planner.agent.md` — Add both sections (~35 lines)
3. `.github/agents/tester.agent.md` — Add both sections (~35 lines)
4. `.github/agents/QA.agent.md` — Add both sections (~35 lines)
5. `.github/agents/coder.agent.md` — Add Research section only (~20 lines)
6. `.github/agents/PO.agent.md` — Add Research section only (~20 lines)
7. `.github/agents/docwriter.agent.md` — Add Research section only (~20 lines)

### Architecture Documentation (1)

8. `docs/architecture/agent-development-flow.md` — Add "Documentation Delegation Pattern" subsection (~25 lines)

**Note:** `orchestrator.agent.md` is NOT modified. Per AD-5 in the low-level design, Orchestrator is a pure routing agent and doesn't need delegation instructions.

## 4. Implementation Steps

All steps can be executed in parallel as changes are isolated to individual files.

### Phase 1: Update Agent Instructions (Steps 1-7)

#### Step 1: Update Architect Instructions
- **File:** `.github/agents/architect.agent.md`
- **Action:** 
  - Insert "Codebase Research" section (~20 lines) after "Project Context"
  - Insert "Documentation Delegation" section (~15 lines) after "Codebase Research"
- **Content source:** Low-level design, "Agent-Specific Section Content → Architect"
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None

#### Step 2: Update Planner Instructions
- **File:** `.github/agents/planner.agent.md`
- **Action:**
  - Insert "Codebase Research" section (~20 lines) after "Your Role"
  - Insert "Documentation Delegation" section (~15 lines) after "Codebase Research"
- **Content source:** Low-level design, "Agent-Specific Section Content → Planner"
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None

#### Step 3: Update Coder Instructions
- **File:** `.github/agents/coder.agent.md`
- **Action:**
  - Insert "Codebase Research" section (~20 lines) after "Project Context", before "Coding Principles"
- **Content source:** Low-level design, "Agent-Specific Section Content → Coder"
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None
- **Note:** Coder has `edit` tools, so NO Documentation Delegation section

#### Step 4: Update Tester Instructions
- **File:** `.github/agents/tester.agent.md`
- **Action:**
  - Insert "Codebase Research" section (~20 lines) after "Project Context", before "Workflow"
  - Insert "Documentation Delegation" section (~15 lines) after "Codebase Research"
- **Content source:** Low-level design, "Agent-Specific Section Content → Tester"
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None

#### Step 5: Update QA Instructions
- **File:** `.github/agents/QA.agent.md`
- **Action:**
  - Insert "Codebase Research" section (~20 lines) after "Project Context", before "Automated Checks Workflow"
  - Insert "Documentation Delegation" section (~15 lines) after "Codebase Research"
- **Content source:** Low-level design, "Agent-Specific Section Content → QA"
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None

#### Step 6: Update PO Instructions
- **File:** `.github/agents/PO.agent.md`
- **Action:**
  - Insert "Codebase Research" section (~20 lines) after "Product Context", before "Your Guiding Principles"
- **Content source:** Low-level design, "Agent-Specific Section Content → PO"
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None
- **Note:** PO has `edit` tools, so NO Documentation Delegation section

#### Step 7: Update DocWriter Instructions
- **File:** `.github/agents/docwriter.agent.md`
- **Action:**
  - Insert "Codebase Research" section (~20 lines) after "Project Documentation Structure", before "Documentation Standards"
- **Content source:** Low-level design, "Agent-Specific Section Content → DocWriter"
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None
- **Note:** DocWriter IS the delegation target, so NO Documentation Delegation section

### Phase 2: Update Architecture Documentation (Step 8)

#### Step 8: Update Agent Development Flow
- **File:** `docs/architecture/agent-development-flow.md`
- **Action:**
  - Add new "Documentation Delegation Pattern" subsection after "Search Delegation Pattern" subsection
- **Content source:** Low-level design, "`agent-development-flow.md` Updates → Add new Documentation Delegation Pattern subsection"
- **Content:** ~25 lines explaining the pattern, when to use it, composition with Search
- **Risk:** Low (markdown content update, no code changes)
- **Dependencies:** None

## 5. Branch & Commit Strategy

- **Branch Name:** `feature/agent-delegation-patterns` (already exists)
- **Commits:** Single atomic commit containing all 8 file changes
- **Commit Message:**
  ```
  feat(agents): Add Search and Documentation delegation patterns
  
  - Add "Codebase Research" section to 7 agents (Architect, Planner, Coder, Tester, QA, PO, DocWriter)
  - Add "Documentation Delegation" section to 4 agents (Architect, Planner, Tester, QA)
  - Update agent-development-flow.md with Documentation Delegation Pattern
  - Standardize when/how agents delegate to Search (research) and DocWriter (persistence)
  
  Closes #<issue-number>
  ```

## 6. Testing Strategy

### 6.1 Pre-Commit Validation

- [ ] **Content Accuracy:** Verify each section matches the low-level design exactly
- [ ] **Line Count:** Confirm ~35 lines for 4 agents (both sections), ~20 lines for 3 agents (Research only), ~25 lines for architecture doc
- [ ] **Insertion Points:** Verify sections are inserted at correct locations per integration table in low-level design
- [ ] **Markdown Syntax:** No broken links, proper code fences, consistent formatting

### 6.2 Post-Commit Validation (Observational)

Monitor next 3-5 feature development cycles for:

| Metric | How to Measure | Target |
|--------|----------------|--------|
| Search invocations per feature | Count Search subagent calls during features | 3-5 per feature (up from ~0-1) |
| DocWriter invocations per feature | Count DocWriter calls by non-edit agents | 1-2 per feature (up from 0) |
| Self-search for deep analysis | Observe multi-tool grep chains | Rare (quick lookups only) |
| Doc persistence rate | Check if agent docs saved to disk vs conversation-only | 100% (all artifacts persisted) |
| Content-first compliance | Check if DocWriter receives complete content vs outlines | High adherence |
| Prompt format compliance | Check what/where/synthesize format (Search) | High adherence |

### 6.3 Anti-Patterns to Watch

| Anti-Pattern | Symptom | Fix |
|--------------|---------|-----|
| Over-delegation to Search | Search invoked for single-file reads | Add more "use own tools" examples |
| Under-delegation to Search | Agent does 10+ grep searches | Add more explicit trigger scenarios |
| Vague Search prompts | "Find parameter code" (no what/where/synthesize) | Strengthen prompt structure examples |
| Skipped DocWriter | Agent generates content but doesn't persist it | Strengthen "must persist" language |
| Outline delegation | Agent passes outline instead of complete content to DocWriter | Add explicit anti-examples |
| Delegating to Coder | Agent asks Coder to save docs instead of DocWriter | Add explicit "Don't ask Coder" instruction |

## 7. Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agents ignore new instructions | Medium | Low | Examples are concrete and match real tasks; iterate if needed |
| Instructions inflate context budget | Low | Low | ~25 lines Research + ~15 lines Documentation per agent; negligible vs. total instruction size |
| Over-delegation for trivial queries | Medium | Low | Clear "use own tools" criteria; can tighten in Phase 2 |
| Agents skip DocWriter persistence | Medium | Medium | Explicit "must persist" language; Orchestrator can verify file existence before handoff |
| DocWriter rewrites agent content | Low | Medium | Clear instruction: "write as-is, minor formatting only" |

## 8. Summary of Changes

| File | Change | Lines Added |
|------|--------|-------------|
| `.github/agents/architect.agent.md` | Add both sections | ~35 |
| `.github/agents/planner.agent.md` | Add both sections | ~35 |
| `.github/agents/coder.agent.md` | Add Research section | ~20 |
| `.github/agents/tester.agent.md` | Add both sections | ~35 |
| `.github/agents/QA.agent.md` | Add both sections | ~35 |
| `.github/agents/PO.agent.md` | Add Research section | ~20 |
| `.github/agents/docwriter.agent.md` | Add Research section | ~20 |
| `docs/architecture/agent-development-flow.md` | Add Documentation Delegation Pattern | ~25 |
| **Total** | **8 files modified** | **~225 lines** |

## 9. Success Criteria

- [ ] All 7 agent instruction files updated with correct sections
- [ ] All 4 applicable agents have Documentation Delegation sections
- [ ] Architecture documentation updated
- [ ] Single PR created and merged
- [ ] No syntax errors in any modified files
- [ ] Agents begin using Search delegation in subsequent features
- [ ] Agents persist documentation artifacts via DocWriter instead of leaving content in conversation

## 10. Related Documents

- [User Stories](./user-stories.md) — Added Stories 9-12 for Documentation Delegation
- [Low-Level Design](./low-level-design-agent-search-delegation.md) — Complete design with both patterns
- [Agent Development Flow](../../architecture/agent-development-flow.md) — Will be updated with new subsection
