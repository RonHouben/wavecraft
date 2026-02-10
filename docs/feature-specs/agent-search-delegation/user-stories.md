# User Stories: Agent Delegation Patterns

## Overview

This feature establishes clear delegation patterns for two critical agent capabilities:

### 1. Search Delegation (Research)

The Search agent has a 272K context window enabling deep codebase analysis across 50-100 files simultaneously. However, specialized agents (Architect, Planner, Coder, Tester, QA, PO, DocWriter) currently lack instructions on **when and how** to delegate research tasks to Search.

**Problem:** Agents have the capability (`agents: [..., search]`) but lack the knowledge of when to use it.  
**Solution:** Explicit "Codebase Research" sections in agent instructions with concrete examples.

### 2. Documentation Delegation (File Creation)

Four agents (Architect, Planner, Tester, QA) are instructed to CREATE documentation files but lack the `edit` tool. They can't actually write `.md` files.

**Problem:** Agents try to create documents they can't edit, leading to workflow failures.  
**Solution:** These agents generate document content/structure and invoke DocWriter to actually write the file.

---

## Benefits

- **Consistent research quality** — All agents use Search for deep analysis
- **Proper tool usage** — Agents delegate to specialists with the right capabilities
- **Clear workflows** — No confusion about who creates what
- **Reduced failures** — Agents don't attempt operations they can't perform

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

## User Story 9: Architect Document Delegation

**As an** Architect agent  
**I want** clear instructions to delegate low-level-design.md creation to DocWriter  
**So that** I don't try to create files I can't edit

### Acceptance Criteria

- [ ] Architect instructions clarify: "You generate design content, DocWriter writes the file"
- [ ] Provides handoff format: "Generate complete markdown content → invoke DocWriter with filepath and content"
- [ ] Includes example: "After completing design, invoke DocWriter to create `docs/feature-specs/{feature}/low-level-design-{feature}.md`"
- [ ] Specifies Architect responsibility: structure, technical decisions, diagrams (as markdown)
- [ ] Specifies DocWriter responsibility: file creation, formatting consistency

### Notes

- **Current problem:** Architect lacks `edit` tool but instructions say "create low-level-design.md"
- **Solution:** Architect generates content, hands off to DocWriter for file creation
- Maintains separation: technical design (Architect) vs documentation mechanics (DocWriter)

---

## User Story 10: Planner Document Delegation

**As a** Planner agent  
**I want** clear instructions to delegate implementation-plan.md creation to DocWriter  
**So that** I can focus on planning logic without worrying about file operations

### Acceptance Criteria

- [ ] Planner instructions clarify delegation pattern for implementation-plan.md
- [ ] Provides handoff format: "Generate step-by-step plan → invoke DocWriter"
- [ ] Includes example: "After creating implementation steps, invoke DocWriter to create `docs/feature-specs/{feature}/implementation-plan.md`"
- [ ] Specifies Planner responsibility: task breakdown, ordering, dependencies
- [ ] Specifies DocWriter responsibility: file creation, markdown structure

### Notes

- **Current problem:** Planner lacks `edit` tool, can't create implementation-plan.md
- **Solution:** Planner focuses on planning, delegates file writing
- Cleaner separation of concerns: planning (Planner) vs documentation (DocWriter)

---

## User Story 11: Tester Document Delegation

**As a** Tester agent  
**I want** clear instructions to delegate test-plan.md creation to DocWriter  
**So that** I can focus on test strategy without file creation concerns

### Acceptance Criteria

- [ ] Tester instructions clarify delegation pattern for test-plan.md
- [ ] Provides handoff format: "Generate test cases → invoke DocWriter"
- [ ] Includes example: "After defining test strategy, invoke DocWriter to create `docs/feature-specs/{feature}/test-plan.md`"
- [ ] Specifies Tester responsibility: test cases, coverage analysis, execution results
- [ ] Specifies DocWriter responsibility: file creation, consistent formatting

### Notes

- **Current problem:** Tester has `execute` tool but not `edit`, can't create test-plan.md
- **Solution:** Tester designs tests, delegates documentation
- Tester can still update test results by invoking DocWriter with updated content

---

## User Story 12: QA Document Delegation

**As a** QA agent  
**I want** clear instructions to delegate QA-report.md creation to DocWriter  
**So that** I don't attempt file operations I can't perform

### Acceptance Criteria

- [ ] QA instructions clarify delegation pattern for QA-report.md
- [ ] Provides handoff format: "Generate quality findings → invoke DocWriter"
- [ ] Includes example: "After analysis, invoke DocWriter to create `docs/feature-specs/{feature}/QA-report.md`"
- [ ] Specifies QA responsibility: quality analysis, issue identification, recommendations
- [ ] Specifies DocWriter responsibility: file creation, report formatting

### Notes

- **Current problem:** QA agent lacks `edit` tool, can't create QA-report.md
- **Solution:** QA analyzes quality, delegates report writing
- Maintains focus: quality analysis (QA) vs documentation mechanics (DocWriter)

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|---------|
| **Search Delegation** | | |
| Search agent invocations per feature | ~0-1 | ~3-5 |
| Agents using own search tools for deep analysis | High | Low (quick lookups only) |
| Research quality consistency | Variable | Consistent (all use Search) |
| Time spent on research per agent | High | Low (delegate to Search) |
| **Documentation Delegation** | | |
| Documentation creation failures | High (agents lack edit tool) | Zero |
| DocWriter invocations per feature | ~0 | ~4 (Architect, Planner, Tester, QA) |
| Time spent on file creation mechanics | High (agents retry/fail) | Low (DocWriter handles it) |

---

## Dependencies

- No code changes required (instructions-only update)
- **Search delegation:** Affects all 7 specialized agents
- **Documentation delegation:** Affects 4 agents (Architect, Planner, Tester, QA) + DocWriter
- Requires coordination with DocWriter for updated instructions
- Should update `agent-development-flow.md` to document both patterns

---

## Risks

| Risk | Mitigation |
|------|------------|
| **Search Delegation** | |
| Agents over-delegate (invoke Search for trivial lookups) | Provide "when to delegate" vs "when to search directly" guidance |
| Search agent becomes bottleneck | Search is read-only, no bottleneck risk (parallel invocations OK) |
| **Documentation Delegation** | |
| Agents forget to delegate, try to create files anyway | Clear error messages + examples in instructions |
| DocWriter becomes bottleneck | DocWriter is lightweight (file creation only), no bottleneck risk |
| Content/formatting mismatches | Specify clear handoff format (filepath + complete markdown content) |
| **General** | |
| Instructions too long/complex | Keep concise: 5-7 bullet points per agent, concrete examples |
| Agents ignore new instructions | Add examples that match real tasks they perform |

---

## Next Steps

1. **Architect:** Design both delegation patterns
   - "Codebase Research" section structure (Search delegation)
   - "Document Creation" section structure (DocWriter delegation)
   - Define consistent formats across all agents
   - Create agent-specific examples for both patterns
   - Specify handoff protocols

2. **Planner:** Create implementation plan for updating agent instructions
   - 7 agents need Search delegation guidance
   - 4 agents need DocWriter delegation guidance (+ DocWriter itself)
   - Update `agent-development-flow.md` with both patterns

3. **Coder:** Implement instruction updates
   - Add "Codebase Research" sections to all agents
   - Add "Document Creation" guidance to Architect, Planner, Tester, QA
   - Update DocWriter to expect delegation requests
   - Update `agent-development-flow.md`

4. **Tester:** Validate both delegation patterns
   - Test Search delegation (agents correctly invoke Search for research)
   - Test DocWriter delegation (agents successfully create documents via DocWriter)

5. **QA:** Review instruction clarity and consistency across both patterns
