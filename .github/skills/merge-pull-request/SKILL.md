---
name: merge-pull-request
description: Merges a Pull Request using GitHub CLI after verifying CI passes. Handles squash merge, branch cleanup, and local sync. Use when user wants to merge a PR or asks to complete/finish a feature.
---

# Merge Pull Request Skill

## Purpose

Merges an approved Pull Request using `gh pr merge`, cleans up the feature branch, and syncs the local repository.

## Prerequisites

- GitHub CLI (`gh`) installed and authenticated
- PR must exist and be approved (or no required reviews)
- CI checks must pass

## Workflow

### Step 1: Verify PR Status

```bash
# Check PR status and CI
gh pr status

# Or check specific PR
gh pr view --json state,reviewDecision,statusCheckRollup
```

Verify:
- `state: OPEN`
- `reviewDecision: APPROVED` (if reviews required)
- All status checks passing

### Step 2: Merge the PR

Use squash merge (default for this project):

```bash
# Squash merge with auto-generated commit message
gh pr merge --squash --delete-branch

# Or with custom commit message
gh pr merge --squash --delete-branch --subject "feat: Feature title" --body "Description"
```

**Merge options:**
- `--squash` — Combines all commits into one (preferred)
- `--rebase` — Rebases commits onto base branch
- `--merge` — Creates merge commit
- `--delete-branch` — Deletes remote branch after merge

### Step 3: Sync Local Repository

```bash
# Switch to main and pull
git checkout main
git pull origin main

# Delete local feature branch
git branch -d feature/branch-name

# Prune stale remote tracking branches
git fetch --prune
```

### Step 4: Confirm Success

```bash
# Verify merge
gh pr view <PR_NUMBER> --json state,mergedAt

# Check main branch has the changes
git log --oneline -5
```

## Error Handling

| Error | Solution |
|-------|----------|
| CI checks failing | Wait for CI or investigate failures |
| Review required | Request review or get approval |
| Merge conflicts | Resolve conflicts locally, push, then merge |
| Branch protection | Ensure all requirements met |

## Example

**User says:** "Merge the PR"

**Agent workflow:**
1. Run `gh pr status` to identify current PR
2. Verify CI passes with `gh pr checks`
3. Run `gh pr merge --squash --delete-branch`
4. Switch to main: `git checkout main && git pull`
5. Clean up: `git branch -d feature/branch-name && git fetch --prune`
6. Confirm: "✅ PR #123 merged and branch cleaned up"

## Notes

- Always use `--squash` for cleaner history
- Always use `--delete-branch` to keep remote clean
- Sync local repo immediately after merge
- If PR was created from a fork, `--delete-branch` only affects the fork
