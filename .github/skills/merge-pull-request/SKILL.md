---
name: merge-pull-request
description: Merges a Pull Request using GitHub CLI after verifying CI passes. Handles squash merge, remote branch cleanup, and local sync. Local branch is preserved. Use when user wants to merge a PR or asks to complete/finish a feature.
---

# Merge Pull Request Skill

## Purpose

Merges an approved Pull Request using `gh pr merge`, cleans up the **remote** feature branch (keeps local), and syncs the local repository.

## Prerequisites

- GitHub CLI (`gh`) installed and authenticated
- PR must exist and be **approved** (at least one approval required)
- CI checks must pass
- All discussions must be **resolved** (no unresolved discussions)

## Workflow

### Step 1: Verify PR Status

```bash
# Check PR status, reviews, and discussions
gh pr view <PR_NUMBER> --json state,reviewDecision,statusCheckRollup,reviewThreads

# Or check current PR
gh pr status
```

Verify:
- `state: OPEN`
- `reviewDecision: APPROVED` (**required** — do not merge without approval)
- All status checks passing
- **No unresolved discussions** — check `reviewThreads` for any with `isResolved: false`

**If not approved:** Stop and inform the user that the PR needs approval before merging.

**If unresolved discussions exist:** Stop and inform the user that all discussions must be resolved before merging.

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
- `--delete-branch` — Deletes **remote** branch after merge (local branch is preserved)

### Step 3: Sync Local Repository

```bash
# Switch to main and pull
git checkout main
git pull origin main
```

**Note:** Local feature branch is kept for reference. Delete manually with `git branch -d <branch>` when no longer needed.

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
| Not approved | **Do not merge.** Inform user PR requires approval first |
| Unresolved discussions | **Do not merge.** Inform user all discussions must be resolved first |
| Merge conflicts | Resolve conflicts locally, push, then merge |
| Branch protection | Ensure all requirements met |

## Example

**User says:** "Merge the PR"

**Agent workflow:**
1. Run `gh pr status` to identify current PR
2. Check approval: `gh pr view --json reviewDecision` — must be `APPROVED`
3. Check discussions: `gh pr view --json reviewThreads` — ensure none have `isResolved: false`
4. Verify CI passes with `gh pr checks`
5. Run `gh pr merge --squash --delete-branch`
6. Switch to main: `git checkout main && git pull`
7. Prune remotes: `git fetch --prune`
8. Confirm: "✅ PR #123 merged. Local branch kept for reference."

## Notes

- Always use `--squash` for cleaner history
- Always use `--delete-branch` to keep remote clean (local branch is **never** auto-deleted)
- **Check for unresolved discussions** — GitHub CLI can query `reviewThreads` to find any with `isResolved: false`
- Sync local repo immediately after merge
- Local feature branch is preserved for reference; user can delete manually with `git branch -d <branch>` when no longer needed
- If PR was created from a fork, `--delete-branch` only affects the fork
