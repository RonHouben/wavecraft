# User Stories: Agent Search Delegation

## Overview

The Search agent has a 272K context window enabling deep codebase analysis across 50-100 files simultaneously. However, specialized agents (Architect, Planner, Coder, Tester, QA, PO, DocWriter) currently lack instructions on **when and how** to delegate research tasks to Search.

This feature adds "Codebase Research" guidance to each agent's instructions, ensuring:
- Consistent research quality across the workflow
- Proper utilization of Search's specialized capabilities
- Clear delegation patterns that match our agent specialization philosophy
- Reduced redundant search operations by individual agents

**Problem:** Agents have the capability (`agents: [..., search]`) but lack the knowledge of when to use it.  
**Solution:** Explicit delegation instructions with concrete examples in each agent's mode instructions.

---

## User Story 1: Architect Needs Deep Research Capability

**As an** Architect agent  
**I want** clear instructions on when to delegate codebase research to Search  
**So that** I can base my designs on comprehensive analysis without manually searching dozens of files

### Acceptance Criteria

- [ ] Architect instructions include "Codebase Research" section
- [ ] Section explains Search's 272K context advantage for multi-file analysis
- [ ] Provides examples: "When designing parameter sync, invoke Search to find all existing sync implementations"
- [ ] Instructs to avoid deep grep_search/semantic_search chains — delegate instead
- [ ] Specifies handoff format: "Search for [pattern] across [scope], synthesize [specific findings]"

### Notes

- Architect often needs to understand existing patterns before designing new ones
- Current behavior: Architect uses own search tools, gets fragmented view
- Desired: Architect invokes Search, gets synthesized findings across entire codebase

---

## User Story 2: Planner Needs Context for Implementation Planning

**As a** Planner agent  
**I want** to invoke Search for comprehensive context gathering  
**So that** my implementation plans account for all relevant code locations and dependencies

### Acceptance Criteria

- [ ] Planner instructions include "Codebase Research" section
- [ ] Explains when to delegate: before breaking down complex features, identifying affected components
- [ ] Provides examples: "When planning IPC changes, invoke Search to map all IPC touchpoints"
- [ ] Instructs to use Search for dependency mapping, not just individual file reads
- [ ] Specifies Search can identify crosscutting concerns that affect multiple plan steps

### Notes

- Planner needs to see the "big picture" before creating detailed steps
- Missing dependencies in plan → Coder discovers surprises during implementation

---

## User Story 3: Coder Needs Fast Pattern Discovery

**As a** Coder agent  
**I want** instructions to invoke Search when I need to understand patterns or locate code  
**So that** I can implement consistently with existing codebase conventions

### Acceptance Criteria

- [ ] Coder instructions include "Codebase Research" section
- [ ] Explains when to delegate: before implementing new patterns, when unsure of conventions
- [ ] Provides examples: "Before adding error handling, invoke Search to find existing error patterns"
- [ ] Instructs to use Search for "How is X done here?" questions
- [ ] Clarifies when direct search is fine (quick lookups) vs when to delegate (pattern analysis)

### Notes

- Coder is execution-focused, should minimize research time
- Search delegation = faster implementation with better consistency

---

## User Story 4: Tester Needs Test Coverage Analysis

**As a** Tester agent  
**I want** to invoke Search to analyze existing test patterns and coverage  
**So that** I can write tests that match project conventions and fill gaps

### Acceptance Criteria

- [ ] Tester instructions include "Codebase Research" section
- [ ] Explains when to delegate: analyzing test coverage, finding test patterns
- [ ] Provides examples: "To verify parameter testing, invoke Search for all parameter-related test files"
- [ ] Instructs to use Search for gap analysis across test suites
- [ ] Specifies Search can identify untested code paths better than manual search

### Notes

- Tester needs to understand what's already tested vs what needs testing
- Search's 272K context ideal for across-the-board test analysis

---

## User Story 5: QA Needs Quality Pattern Analysis

**As a** QA agent  
**I want** to delegate pattern analysis to Search  
**So that** I can identify quality issues across the entire codebase, not just the changed files

### Acceptance Criteria

- [ ] QA instructions include "Codebase Research" section
- [ ] Explains when to delegate: looking for anti-patterns, consistency checks
- [ ] Provides examples: "To check error handling consistency, invoke Search across all error sites"
- [ ] Instructs to use Search for codebase-wide quality metrics
- [ ] Specifies Search can find outliers and inconsistencies that grep misses

### Notes

- QA needs to ensure changes fit broader codebase quality standards
- Manual search = limited view; Search delegation = comprehensive analysis

---

## User Story 6: PO Needs Feature Impact Analysis

**As a** Product Owner agent  
**I want** to invoke Search when assessing feature feasibility  
**So that** I can make prioritization decisions based on complete technical context

### Acceptance Criteria

- [ ] PO instructions include "Codebase Research" section
- [ ] Explains when to delegate: assessing feature complexity, identifying conflicts
- [ ] Provides examples: "To evaluate 'add preset system' effort, invoke Search for existing state management patterns"
- [ ] Instructs to use Search before writing user stories for complex features
- [ ] Specifies Search findings inform acceptance criteria and risk assessment

### Notes

- PO needs technical context without diving into code myself
- Search acts as my "technical analyst" for feasibility assessment

---

## User Story 7: DocWriter Needs Documentation Scope Discovery

**As a** DocWriter agent  
**I want** to invoke Search to find all code relevant to documentation updates  
**So that** my documentation is complete and doesn't miss related components

### Acceptance Criteria

- [ ] DocWriter instructions include "Codebase Research" section
- [ ] Explains when to delegate: documenting features, finding related code
- [ ] Provides examples: "To document IPC system, invoke Search for all IPC-related code"
- [ ] Instructs to use Search to ensure documentation covers all touchpoints
- [ ] Specifies Search can identify undocumented code that needs docs

### Notes

- DocWriter needs to know "what exists" before deciding "what to document"
- Search's broad analysis prevents documentation gaps

---

## User Story 8: Workflow Efficiency Improvement

**As a** system architect maintaining the agent workflow  
**I want** all agents to follow consistent research delegation patterns  
**So that** the Search agent's specialized capabilities are properly utilized

### Acceptance Criteria

- [ ] All 7 agent instruction files updated with "Codebase Research" section
- [ ] Each section follows consistent format (when/how/examples)
- [ ] Documentation in `agent-development-flow.md` updated to explain delegation pattern
- [ ] Examples in agent instructions reference real codebase patterns
- [ ] Agent invocation table in `agent-development-flow.md` includes Search delegation note

### Notes

- This is a systemic improvement across the entire agent workflow
- Success = agents naturally delegate research, Search agent usage increases
- Measure: Search agent invocation count should increase after this change

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|---------|
| Search agent invocations per feature | ~0-1 | ~3-5 |
| Agents using own search tools for deep analysis | High | Low (quick lookups only) |
| Research quality consistency | Variable | Consistent (all use Search) |
| Time spent on research per agent | High | Low (delegate to Search) |

---

## Dependencies

- No code changes required (instructions-only update)
- Affects all 7 specialized agents
- Requires coordination with DocWriter for instruction updates
- Should update `agent-development-flow.md` to document pattern

---

## Risks

| Risk | Mitigation |
|------|------------|
| Agents over-delegate (invoke Search for trivial lookups) | Provide "when to delegate" vs "when to search directly" guidance |
| Search agent becomes bottleneck | Search is read-only, no bottleneck risk (parallel invocations OK) |
| Instructions too long/complex | Keep concise: 5-7 bullet points per agent, concrete examples |
| Agents ignore new instructions | Add examples that match real tasks they perform |

---

## Next Steps

1. **Architect:** Design the "Codebase Research" section structure
   - Define consistent format across all agents
   - Create agent-specific examples
   - Specify handoff protocol to Search agent

2. **Planner:** Create implementation plan for updating all agent instructions

3. **Coder:** Implement instruction updates + `agent-development-flow.md` changes

4. **Tester:** Validate agents correctly delegate research tasks

5. **QA:** Review instruction clarity and consistency
