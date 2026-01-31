---
name: create-pull-request
description: Creates a Pull Request using GitHub CLI with a PR description file. Generates PR-description.md in the feature-specs folder to avoid terminal pasting issues.
---

# Create Pull Request Skill

## Purpose

This skill creates a GitHub Pull Request using the `gh` CLI tool. To avoid issues with pasting large text into the terminal, it first creates a `PR-description.md` file in `/docs/feature-specs/${featureName}/` and then uses this file as the PR body.

## Prerequisites

- GitHub CLI (`gh`) must be installed and authenticated
- Current branch must be pushed to the remote
- There must be commits to create a PR from

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

### Step 5: Create PR Description File

Create the file at `/docs/feature-specs/${featureName}/PR-description.md` with auto-generated content:

```markdown
## Summary

[Auto-generated summary based on commit messages and changed files]

## Changes

[Auto-generated list based on git diff --stat, grouped by area:]
- **Engine/DSP**: [Rust changes in engine/crates/]
- **UI**: [TypeScript/React changes in ui/src/]
- **Build/Config**: [Changes to Cargo.toml, package.json, configs]
- **Documentation**: [Changes to docs/]

## Commits

[List of commits from git log --oneline]

## Related Documentation

[Auto-linked if files exist:]
- [Implementation Plan](./implementation-plan.md)
- [Low-Level Design](./low-level-design-*.md)

## Testing

[Auto-generated based on changes:]
- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`
- [If UI changes] Manual UI verification
- [If DSP changes] Audio processing verification

## Checklist

- [ ] Code follows project coding standards
- [ ] Tests added/updated as needed
- [ ] Documentation updated
- [ ] No linting errors (`cargo xtask lint`)
```

### Step 6: Create the Pull Request

Run the following commands:

```bash
# Ensure branch is pushed
git push -u origin HEAD

# Create PR using the description file (use relative path from repo root)
gh pr create \
  --title "${PR_TITLE}" \
  --body-file "docs/feature-specs/${featureName}/PR-description.md" \
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
5. Create `docs/feature-specs/meter-improvements/PR-description.md` with auto-generated content
6. Run: `gh pr create --title "Improve meter performance and accuracy" --body-file "docs/feature-specs/meter-improvements/PR-description.md" --base main`
7. Return: "✅ PR created: https://github.com/owner/repo/pull/123"

## Notes

- The PR description file is committed to the repository for documentation purposes
- If the feature-specs folder doesn't exist for this feature, create it
- Always use relative paths from the repository root in the `--body-file` argument
- All PR details (title, description) are auto-generated from branch changes - no user input required
