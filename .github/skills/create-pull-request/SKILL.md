---
name: create-pull-request
description: Creates a Pull Request using GitHub CLI with a PR summary file. Generates PR-summary.md in the feature-specs folder to avoid terminal pasting issues.
---

# Create Pull Request Skill

## Purpose

This skill creates a GitHub Pull Request using the `gh` CLI tool. To avoid issues with pasting large text into the terminal, it first creates a `PR-summary.md` file in `/docs/feature-specs/${featureName}/` and then uses this file as the PR body.

## Prerequisites

- GitHub CLI (`gh`) must be installed and authenticated
- Current branch must be pushed to the remote
- There must be commits to create a PR from
- The executing agent must have terminal execution tools (see [Terminal Handoff](#terminal-handoff) if you don't)

### Terminal Handoff

If you don't have terminal execution tools, hand off to the **Coder** agent with the following scope constraint:

> **Scope constraint:** Execute ONLY the git and gh CLI commands provided below. Do NOT edit code, fix warnings, refactor, or take any action beyond running these commands and reporting their output. If a command fails, report the error — do not attempt autonomous fixes.

Include in your handoff:

1. The exact commands to run (from Steps 2 and 6)
2. The PR title (from Step 4)
3. The path to the PR summary file (from Step 5)
4. This scope constraint verbatim

## Workflow

### Step 1: Determine Feature Name and Base Branch

Extract the feature name from the current branch name:

- `feature/my-feature` → `my-feature`
- `bugfix/fix-description` → `fix-description`

Determine the base branch by checking which remote branches exist:

- Default to `main` if it exists
- Fall back to `develop` or `master` if `main` doesn't exist

### Step 2: Analyze Changes

Gather all context automatically:

```bash
# Get the merge base
BASE_BRANCH="main"  # or detected base branch
MERGE_BASE=$(git merge-base HEAD origin/${BASE_BRANCH})

# List all commits
git log --oneline ${MERGE_BASE}..HEAD

# Get detailed commit messages for context
git log --pretty=format:"%s%n%n%b" ${MERGE_BASE}..HEAD

# Review changed files with stats
git diff --stat ${MERGE_BASE}

# Get list of changed files by type
git diff --name-only ${MERGE_BASE}
```

### Step 3: Check Existing Documentation

Look for existing feature documentation:

1. `/docs/feature-specs/${featureName}/implementation-plan.md` - for planned changes
2. `/docs/feature-specs/${featureName}/implementation-progress.md` - for completed work
3. `/docs/feature-specs/${featureName}/low-level-design-*.md` - for design context
4. `/docs/feature-specs/${featureName}/user-stories.md` - for requirements

### Step 4: Generate PR Title

Auto-generate the PR title based on:

1. If single commit: Use the commit message subject
2. If multiple commits: Summarize the overall change based on:
   - The feature name from the branch
   - The types of files changed (e.g., "Add meter component" if UI components added)
   - Commit message patterns (e.g., "fix:", "feat:", "refactor:")

Examples:

- Branch `feature/meter-improvements` with perf commits → "Improve meter performance and accuracy"
- Branch `bugfix/slider-crash` → "Fix slider crash on invalid input"
- Branch `cs-1234-add-reverb` → "CS-1234: Add reverb effect"

### Step 5: Create PR Summary File

**⚠️ Use the `edit` tool to create this file - DO NOT use terminal commands like `echo`.**

Create the file at `/docs/feature-specs/${featureName}/PR-summary.md` using the template from [assets/PR-summary-template.md](assets/PR-summary-template.md).

**Tool usage:**

- Use the `edit` tool or equivalent file creation capability
- Never use `echo`, `cat`, or other terminal commands for file creation
- The file editor ensures proper formatting and allows immediate editing if needed

Fill in the auto-generated content for each section:

- **Summary**: Based on commit messages and changed files
- **Changes**: Grouped by area (Engine/DSP, UI, Build/Config, Documentation)
- **Commits**: List from `git log --oneline`
- **Related Documentation**: Auto-link existing feature-spec files
- **Testing**: Based on types of changes made
- **Checklist**: Standard quality checks

### Step 6: Create the Pull Request

Run the following commands:

```bash
# Ensure branch is pushed
git push -u origin HEAD

# Create PR using the summary file (use relative path from repo root)
gh pr create \
  --title "${PR_TITLE}" \
  --body-file "docs/feature-specs/${featureName}/PR-summary.md" \
  --base "${BASE_BRANCH}"
```

### Step 7: Confirm Success

After successful PR creation:

1. Display the PR URL to the user
2. Optionally open the PR in the browser with `gh pr view --web`

## Error Handling

- If `gh` is not installed: Instruct user to install with `brew install gh`
- If not authenticated: Run `gh auth login`
- If branch not pushed: Push with `git push -u origin HEAD`
- If PR already exists: Inform user and provide link to existing PR

## Example Usage

**User says:** "Create a PR for my changes"

**Agent workflow:**

1. Determine feature name from branch: `feature/meter-improvements` → `meter-improvements`
2. Analyze commits: 3 commits about performance improvements
3. Check changed files: `ui/src/components/Meter.tsx`, `ui/src/lib/audio-math.ts`
4. Auto-generate title: "Improve meter performance and accuracy"
5. Create `docs/feature-specs/meter-improvements/PR-summary.md` with auto-generated content
6. Run: `gh pr create --title "Improve meter performance and accuracy" --body-file "docs/feature-specs/meter-improvements/PR-summary.md" --base main`
7. Return: "✅ PR created: https://github.com/owner/repo/pull/123"

## Notes

- The PR summary file is committed to the repository for documentation purposes
- If the feature-specs folder doesn't exist for this feature, create it
- Always use relative paths from the repository root in the `--body-file` argument
- All PR details (title, description) are auto-generated from branch changes - no user input required
