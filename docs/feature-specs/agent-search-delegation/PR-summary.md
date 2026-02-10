# PR Summary: Add Search Delegation Instructions to Agents

## Summary

This PR adds "Codebase Research" sections to all specialized agent files, enabling them to invoke the Search agent (272K context, GPT-5.2-Codex) for deep codebase analysis instead of fragmented multi-tool research.

**Key changes:**
- Added standardized "Codebase Research" section to 7 agent files (Architect, Planner, Coder, Tester, QA, PO, DocWriter)
- Added "Search Delegation Pattern" section to `agent-development-flow.md`
- Each section includes:
  - When to delegate to Search vs. use own tools
  - How to structure Search requests (what + where + synthesize)
  - Agent-specific examples using real Wavecraft patterns

**Impact:**
- Reduces fragmented multi-tool searches by specialized agents
- Enables comprehensive cross-layer analysis (e.g., Engine → Bridge → UI flows)
- Improves design/planning quality through better codebase understanding
- Maintains clear delegation boundaries (single-file lookups = self, multi-file analysis = Search)

## Type of Change

- [x] Documentation update
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change

## Changes by Area

### Agent Instructions
- `.github/agents/architect.agent.md` — Added "Codebase Research" section (19 lines)
- `.github/agents/planner.agent.md` — Added "Codebase Research" section (21 lines)
- `.github/agents/coder.agent.md` — Added "Codebase Research" section (18 lines)
- `.github/agents/tester.agent.md` — Added "Codebase Research" section (20 lines)
- `.github/agents/QA.agent.md` — Added "Codebase Research" section (20 lines)
- `.github/agents/PO.agent.md` — Added "Codebase Research" section (18 lines)
- `.github/agents/docwriter.agent.md` — Added "Codebase Research" section (18 lines)

### Architecture Documentation
- `docs/architecture/agent-development-flow.md` — Added "Search Delegation Pattern" subsection (14 lines)

### Feature Specifications
- `docs/feature-specs/agent-search-delegation/implementation-progress.md` — Progress tracking
- `docs/feature-specs/agent-search-delegation/low-level-design-agent-search-delegation.md` — Design specification
- `docs/feature-specs/agent-search-delegation/user-stories.md` — User requirements

## Commits

- `adcfac0` docs: add search delegation instructions to agents

## Related Documentation

- [Low-Level Design](../docs/feature-specs/agent-search-delegation/low-level-design-agent-search-delegation.md) — Technical design and section content
- [User Stories](../docs/feature-specs/agent-search-delegation/user-stories.md) — Requirements and acceptance criteria
- [Implementation Progress](../docs/feature-specs/agent-search-delegation/implementation-progress.md) — Task tracking
- [Agent Development Flow](../docs/architecture/agent-development-flow.md) — Updated with Search delegation pattern
- [High-Level Design](../docs/architecture/high-level-design.md) — System architecture context

## Testing

- [x] Manual validation: All markdown files render correctly
- [x] Section insertion points verified (after "Project Context", "Your Role", etc.)
- [x] Content matches low-level design specification
- [x] Examples reference real Wavecraft patterns (engine/crates/, ui/packages/)
- [x] No syntax errors or broken section flow

## Migration Notes

This is a documentation-only change with no code impact. The Search agent behavior is unchanged. The new instructions will be available to agents immediately upon merge.

**Validation approach:**
- Observe agent behavior over next 3-5 feature development cycles
- Track Search invocation frequency (target: 3-5 per feature, up from ~0-1)
- Monitor whether agents use what/where/synthesize prompt structure
- Refine examples if needed based on agent underuse/overuse

## Checklist

- [x] Code follows project conventions
- [x] Documentation updated
- [x] No breaking changes
- [x] Single atomic commit
- [x] All files properly attributed to feature
- [x] No unrelated changes included
