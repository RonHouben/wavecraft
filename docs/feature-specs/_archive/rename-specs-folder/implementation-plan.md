# Implementation Plan: Rename docs/specs to docs/feature-specs

## Overview

Rename the `docs/specs` folder to `docs/feature-specs` to better communicate that this directory contains specifications for features under active development. Update all references throughout the codebase to point to the new location.

## Requirements

- Rename `docs/specs` directory to `docs/feature-specs`
- Update all references to `docs/specs` in agent configuration files
- Update all references in copilot-instructions.md
- Update references in the roadmap
- Preserve the `_archive` subdirectory structure and naming convention
- Maintain backward compatibility with archived documentation (references within archived files do not need updating as they are historical records)

## Architecture Changes

- **Folder rename**: `docs/specs/` → `docs/feature-specs/`
- **Configuration updates**: 8 files need reference updates

## Implementation Steps

### Phase 1: Preparation

1. **Verify no pending changes in docs/specs** (Manual check)
   - Action: Ensure there are no uncommitted changes in the `docs/specs` directory
   - Why: Prevent merge conflicts and lost work
   - Dependencies: None
   - Risk: Low

### Phase 2: Rename Directory

2. **Rename the directory** (Terminal command)
   - Action: Run `mv docs/specs docs/feature-specs`
   - Why: Core change that enables the new naming convention
   - Dependencies: Step 1
   - Risk: Low
   - Command: `cd /Users/ronhouben/code/private/vstkit && mv docs/specs docs/feature-specs`

### Phase 3: Update Agent Configurations

3. **Update PO.agent.md** (File: `.github/agents/PO.agent.md`)
   - Action: Replace `docs/specs` with `docs/feature-specs` (2 occurrences)
   - Why: Product Owner agent references specs for user stories and archiving
   - Dependencies: Step 2
   - Risk: Low
   - Lines to update:
     - Line 50: `/docs/specs/_archive/` → `/docs/feature-specs/_archive/`
     - Line 170: `/docs/specs/{feature-name}/user-stories.md` → `/docs/feature-specs/{feature-name}/user-stories.md`

4. **Update coder.agent.md** (File: `.github/agents/coder.agent.md`)
   - Action: Replace `docs/specs` with `docs/feature-specs` (3 occurrences)
   - Why: Coder agent references specs for implementation progress tracking
   - Dependencies: Step 2
   - Risk: Low
   - Lines to update:
     - Line 71: `docs/specs/` directory → `docs/feature-specs/` directory (2 occurrences on same line)
     - Line 170: `docs/specs/` → `docs/feature-specs/`

5. **Update planner.agent.md** (File: `.github/agents/planner.agent.md`)
   - Action: Replace `docs/specs` with `docs/feature-specs` (2 occurrences)
   - Why: Planner agent creates plans in specs directory
   - Dependencies: Step 2
   - Risk: Low
   - Lines to update:
     - Line 99: `docs/specs/[feature_name]/implementation-plan.md` → `docs/feature-specs/[feature_name]/implementation-plan.md`
     - Line 101: `docs/specs/[feature_name]/implementation-progress.md` → `docs/feature-specs/[feature_name]/implementation-progress.md`

6. **Update architect.agent.md** (File: `.github/agents/architect.agent.md`)
   - Action: Replace `docs/specs` with `docs/feature-specs` (1 occurrence)
   - Why: Architect agent creates low-level designs in specs directory
   - Dependencies: Step 2
   - Risk: Low
   - Lines to update:
     - Line 39: `docs/specs/${feature-name}/` → `docs/feature-specs/${feature-name}/`

7. **Update QA.agent.md** (File: `.github/agents/QA.agent.md`)
   - Action: Replace `docs/specs` with `docs/feature-specs` (2 occurrences)
   - Why: QA agent creates reports in specs directory
   - Dependencies: Step 2
   - Risk: Low
   - Lines to update:
     - Line 141: `docs/specs/{feature}/QA-report.md` → `docs/feature-specs/{feature}/QA-report.md`
     - Line 228: `docs/specs/{feature}/` → `docs/feature-specs/{feature}/`

8. **Update tester.agent.md** (File: `.github/agents/tester.agent.md`)
   - Action: Replace `docs/specs` with `docs/feature-specs` (4 occurrences)
   - Why: Tester agent references specs for test plans
   - Dependencies: Step 2
   - Risk: Low
   - Lines to update:
     - Line 55: `docs/specs/{feature}/` → `docs/feature-specs/{feature}/`
     - Line 57: `docs/specs/{feature}/test-plan.md` → `docs/feature-specs/{feature}/test-plan.md`
     - Line 77: `docs/specs/{feature}/test-plan.md` → `docs/feature-specs/{feature}/test-plan.md`
     - Line 84: `docs/specs/{feature}/` → `docs/feature-specs/{feature}/`

### Phase 4: Update Core Configuration

9. **Update copilot-instructions.md** (File: `.github/copilot-instructions.md`)
   - Action: Replace `docs/specs` with `docs/feature-specs` (1 occurrence)
   - Why: Core instruction file for all Copilot agents
   - Dependencies: Step 2
   - Risk: Medium (high impact if incorrect)
   - Lines to update:
     - Line 7: `/docs/specs/_archive/` → `/docs/feature-specs/_archive/`

### Phase 5: Update Documentation

10. **Update roadmap.md** (File: `docs/roadmap.md`)
    - Action: Replace `docs/specs` with `docs/feature-specs` (1 occurrence)
    - Why: Roadmap references specs directory in changelog
    - Dependencies: Step 2
    - Risk: Low
    - Lines to update:
      - Line 172: `docs/specs/linting-infrastructure/` → `docs/feature-specs/linting-infrastructure/`

### Phase 6: Verification

11. **Verify all references updated** (Manual/grep check)
    - Action: Run `grep -r "docs/specs" --include="*.md" .github/ docs/` to verify no active references remain
    - Why: Ensure no references were missed
    - Dependencies: Steps 3-10
    - Risk: Low
    - Note: References in `docs/feature-specs/_archive/` are intentionally preserved as historical records

12. **Test agent functionality** (Manual verification)
    - Action: Verify agents can still reference and create files in the new location
    - Why: Ensure the rename doesn't break agent workflows
    - Dependencies: Steps 3-10
    - Risk: Low

## Files to Modify

| File | Occurrences | Status |
|------|-------------|--------|
| `.github/copilot-instructions.md` | 1 | Required |
| `.github/agents/PO.agent.md` | 2 | Required |
| `.github/agents/coder.agent.md` | 3 | Required |
| `.github/agents/planner.agent.md` | 2 | Required |
| `.github/agents/architect.agent.md` | 1 | Required |
| `.github/agents/QA.agent.md` | 2 | Required |
| `.github/agents/tester.agent.md` | 4 | Required |
| `docs/roadmap.md` | 1 | Required |

**Total: 8 files, 16 occurrences**

## Files NOT to Modify

The following files contain references to `docs/specs` but should NOT be updated:

| File | Reason |
|------|--------|
| `docs/feature-specs/_archive/**/*.md` | Historical records - preserved as-is |

## Testing Strategy

- **Manual verification**: Run grep to ensure all active references are updated
- **Functional test**: Create a test feature spec to verify agents work correctly with the new path
- **Git verification**: Review git diff to ensure only expected changes were made

## Risks & Mitigations

- **Risk**: Agents may have cached references to old path
  - Mitigation: Restart any running agent sessions after the change

- **Risk**: External tools or scripts may reference old path
  - Mitigation: Search for external references; document the change in commit message

- **Risk**: Historical archive references may confuse users
  - Mitigation: Archive files are historical records and intentionally preserved; add note in archive README if needed

## Success Criteria

- [ ] `docs/specs` directory no longer exists
- [ ] `docs/feature-specs` directory exists with all previous contents
- [ ] All 8 configuration/documentation files updated
- [ ] `grep -r "docs/specs" --include="*.md" .github/ docs/` returns only results from `_archive` directory
- [ ] Agents can successfully create new spec files in `docs/feature-specs/`
- [ ] Git history preserved for moved files (using `git mv` or tracking through rename)

## Commit Message

```
refactor: rename docs/specs to docs/feature-specs

Rename the specs directory to feature-specs to better communicate
that this directory contains specifications for features under
active development.

Updated references in:
- .github/copilot-instructions.md
- .github/agents/*.agent.md (6 files)
- docs/roadmap.md

Note: References in docs/feature-specs/_archive/ are preserved
as historical records.
```
