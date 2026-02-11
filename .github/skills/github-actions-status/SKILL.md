# GitHub Actions Status Skill

---

name: github-actions-status
description: Check GitHub Actions workflow runs and pipeline status using GitHub CLI. Use when the user wants to view workflow status, check pipeline runs, monitor CI/CD status, or troubleshoot failing actions.

---

## Quick Reference

### Most Common Commands

```bash
# List recent workflow runs
gh run list

# View status of latest run for a specific workflow
gh run list --workflow=continuous-integration.yml --limit=1

# Watch a workflow run in real-time
gh run watch

# View logs for the latest run
gh run view --log

# View logs for a specific run
gh run view <run-id> --log

# Download artifacts from a run
gh run download <run-id>
```

## Execution Context Handling

**Not all agents have terminal/execution capabilities.** Before attempting to run `gh` commands, determine your execution context:

| Your Capabilities                  | Action                                                                        |
| ---------------------------------- | ----------------------------------------------------------------------------- |
| ✅ Have execution tools (terminal) | Run `gh run ...` commands directly                                            |
| ✅ Can invoke subagents            | Invoke `tester` or `coder` agent with run URL or `{workflow, branch, run_id}` |
| ❌ Neither                         | Provide commands to user to run manually; ask them to paste output            |

**When monitoring runs, you need:**

- Run URL (e.g., `https://github.com/owner/repo/actions/runs/12345`), OR
- Run ID + branch name, OR
- Workflow name + branch name (gets latest run)

**Example handoff:**

```
"Invoke tester agent to monitor CI status for workflow 'continuous-integration.yml'
on branch 'feature/new-feature'. Request they run:
gh run list --workflow=continuous-integration.yml --branch=feature/new-feature --limit=1"
```

## Filtering and Formatting

### By Status

```bash
# Show only failed runs
gh run list --status=failure

# Show only in-progress runs
gh run list --status=in_progress

# Show only successful runs
gh run list --status=success

# All status options: completed, action_required, cancelled, failure,
#                     neutral, skipped, stale, success, timed_out, in_progress,
#                     queued, requested, waiting
```

### By Workflow

```bash
# List runs for specific workflow
gh run list --workflow=continuous-integration.yml

# List runs for multiple workflows (use multiple --workflow flags)
gh run list --workflow=ci.yml --workflow=cd.yml
```

### By Branch

```bash
# Runs on main branch
gh run list --branch=main

# Runs on feature branch
gh run list --branch=feature/new-feature
```

### By Event

```bash
# Triggered by push
gh run list --event=push

# Triggered by pull_request
gh run list --event=pull_request

# Triggered manually
gh run list --event=workflow_dispatch
```

### Output Formatting

```bash
# JSON output for parsing
gh run list --json status,conclusion,name,headBranch,event,createdAt

# JSON with specific fields
gh run list --json status,workflowName,headBranch,conclusion --limit=5

# Human-readable with custom columns (uses --json under the hood)
gh run list --json status,conclusion,name,headBranch --jq '.[] | "\(.name) on \(.headBranch): \(.conclusion)"'
```

## Real-Time Monitoring

### Watch Current Run

```bash
# Watch the currently running workflow (interactive selection if multiple)
gh run watch

# Watch a specific run by ID
gh run watch <run-id>

# Exit after completion (useful for scripts)
gh run watch <run-id> --exit-status
```

### Get Latest Run Status

```bash
# Get status of latest run for a workflow
gh run list --workflow=ci.yml --limit=1 --json status,conclusion

# Check if latest run passed (exit code 0 = success, 1 = failure)
gh run list --workflow=ci.yml --limit=1 --json conclusion -q '.[0].conclusion' | grep -q "success"
```

## Troubleshooting Failing Workflows

### View Failure Details

```bash
# View full logs of failed run
gh run view <run-id> --log-failed

# View specific job logs
gh run view <run-id> --job=<job-id> --log

# List jobs in a run
gh run view <run-id>

# Download full logs for offline analysis
gh run view <run-id> --log > run-logs.txt
```

### Re-run Failed Workflows

```bash
# Re-run failed jobs only
gh run rerun <run-id> --failed

# Re-run entire workflow
gh run rerun <run-id>

# Re-run with debug logging
gh run rerun <run-id> --debug
```

### Check Specific Job Status

```bash
# View run with job details
gh run view <run-id>

# Get JSON output with all jobs
gh run view <run-id> --json jobs

# Extract failed job names
gh run view <run-id> --json jobs --jq '.jobs[] | select(.conclusion == "failure") | .name'
```

## Common Scenarios

### Scenario 1: Check if CI is green before merge

```bash
# Get latest CI run status on current branch
CURRENT_BRANCH=$(git branch --show-current)
gh run list --branch="$CURRENT_BRANCH" --workflow=ci.yml --limit=1 \
  --json conclusion --jq '.[0].conclusion'

# One-liner that exits 0 if success, 1 otherwise
gh run list --branch=$(git branch --show-current) --workflow=ci.yml --limit=1 \
  --json conclusion --jq '.[0].conclusion == "success"'
```

### Scenario 2: Monitor PR checks

```bash
# List all runs for a PR
gh pr checks <pr-number>

# Watch PR checks in real-time
gh pr checks <pr-number> --watch
```

### Scenario 3: Find recent failures

```bash
# Show last 10 failed runs across all workflows
gh run list --status=failure --limit=10

# Show failed runs in the last 24 hours
gh run list --status=failure --limit=50 \
  --json conclusion,createdAt,name,headBranch \
  --jq '.[] | select(.createdAt > (now - 86400 | todate))'
```

### Scenario 4: Check template validation workflow

```bash
# Use case: After CLI changes, check if template-validation workflow passed
gh run list --workflow=template-validation.yml --limit=1

# Get detailed logs if it failed
LATEST_RUN=$(gh run list --workflow=template-validation.yml --limit=1 --json databaseId --jq '.[0].databaseId')
gh run view $LATEST_RUN --log-failed
```

### Scenario 5: Monitor CD pipeline

```bash
# Watch continuous-deploy workflow
gh run list --workflow=continuous-deploy.yml --limit=5

# Get latest CD run status
gh run list --workflow=continuous-deploy.yml --limit=1 --json status,conclusion,createdAt

# Download artifacts from latest successful CD run
LATEST_SUCCESS=$(gh run list --workflow=continuous-deploy.yml --status=success --limit=1 --json databaseId --jq '.[0].databaseId')
gh run download $LATEST_SUCCESS
```

### Scenario 6: Investigate flaky tests

```bash
# Find runs of a specific workflow that failed in the last week
gh run list --workflow=ci.yml --status=failure --limit=50 \
  --json conclusion,createdAt,name,headBranch,event \
  --jq '.[] | select(.createdAt > (now - 604800 | todate)) | {branch: .headBranch, event: .event, created: .createdAt}'

# View logs of specific test job across multiple runs
for run_id in $(gh run list --workflow=ci.yml --limit=10 --json databaseId --jq '.[].databaseId'); do
  echo "=== Run $run_id ==="
  gh run view $run_id --log | grep "FAILED" || echo "No failures"
done
```

## Advanced Filtering

### Combine Multiple Filters

```bash
# Failed runs on main branch for specific workflow in last 24 hours
gh run list \
  --workflow=ci.yml \
  --branch=main \
  --status=failure \
  --limit=20 \
  --json conclusion,createdAt,headBranch \
  --jq '.[] | select(.createdAt > (now - 86400 | todate))'
```

### Custom Output Format

```bash
# Create a concise status report
gh run list --limit=5 --json name,status,conclusion,headBranch,createdAt \
  --jq '.[] | "[\(.status)] \(.name) (\(.headBranch)) - \(.conclusion // "running")"'

# Export to CSV for analysis
gh run list --limit=100 --json workflowName,conclusion,createdAt,headBranch,event \
  --jq -r '["Workflow","Conclusion","Created","Branch","Event"], (.[] | [.workflowName, .conclusion, .createdAt, .headBranch, .event]) | @csv'
```

## Workflow Management

### Trigger Workflows

```bash
# Trigger workflow manually (workflow_dispatch)
gh workflow run continuous-integration.yml

# Trigger with inputs
gh workflow run ci.yml --field debug=true --field environment=staging

# Trigger on specific branch
gh workflow run ci.yml --ref feature/new-feature
```

### List Available Workflows

```bash
# Show all workflows in repo
gh workflow list

# Show details of specific workflow
gh workflow view continuous-integration.yml
```

### Enable/Disable Workflows

```bash
# Disable a workflow
gh workflow disable continuous-integration.yml

# Enable a workflow
gh workflow enable continuous-integration.yml
```

## Troubleshooting Tips

### Permission Issues

If you get permission errors:

```bash
# Check auth status
gh auth status

# Re-authenticate with additional scopes
gh auth login --scopes workflow
```

### Run ID vs Run Number

- **Run ID** (databaseId): Unique identifier across all runs (e.g., 12345678)
- **Run Number**: Sequential number per workflow (e.g., #42)

Most commands accept run ID. Get it with:

```bash
gh run list --limit=1 --json databaseId --jq '.[0].databaseId'
```

### Pagination

By default, `gh run list` shows 10-20 results. Use `--limit` to increase:

```bash
# Get up to 100 results
gh run list --limit=100

# Note: GitHub API has rate limits; avoid excessive requests
```

## Integration with Other Tools

### Use in Scripts

```bash
#!/bin/bash
# Wait for CI to complete before proceeding

WORKFLOW="ci.yml"
BRANCH=$(git branch --show-current)

echo "Waiting for CI on branch: $BRANCH"

# Get latest run ID for this branch
RUN_ID=$(gh run list --workflow="$WORKFLOW" --branch="$BRANCH" --limit=1 --json databaseId --jq '.[0].databaseId')

if [ -z "$RUN_ID" ]; then
  echo "No runs found for $WORKFLOW on $BRANCH"
  exit 1
fi

# Watch until completion
gh run watch "$RUN_ID" --exit-status

if [ $? -eq 0 ]; then
  echo "✅ CI passed"
  exit 0
else
  echo "❌ CI failed"
  gh run view "$RUN_ID" --log-failed
  exit 1
fi
```

### Combine with jq for Analysis

```bash
# Calculate success rate for a workflow
gh run list --workflow=ci.yml --limit=100 --json conclusion \
  | jq '[.[] | select(.conclusion != null)] | group_by(.conclusion) | map({conclusion: .[0].conclusion, count: length})'

# Find slowest runs
gh run list --limit=50 --json name,createdAt,updatedAt,conclusion \
  | jq '.[] | select(.conclusion == "success") | {name: .name, duration: (((.updatedAt | fromdateiso8601) - (.createdAt | fromdateiso8601)) / 60), conclusion: .conclusion}' \
  | jq -s 'sort_by(.duration) | reverse | .[0:10]'
```

## Quick Reference Card

| Task                    | Command                           |
| ----------------------- | --------------------------------- |
| List recent runs        | `gh run list`                     |
| Check specific workflow | `gh run list --workflow=ci.yml`   |
| View run details        | `gh run view <run-id>`            |
| View logs               | `gh run view <run-id> --log`      |
| Watch live run          | `gh run watch`                    |
| Failed runs only        | `gh run list --status=failure`    |
| Re-run failed jobs      | `gh run rerun <run-id> --failed`  |
| Download artifacts      | `gh run download <run-id>`        |
| Trigger workflow        | `gh workflow run <workflow-name>` |
| PR checks               | `gh pr checks <pr-number>`        |

## Common Exit Codes

- `0`: Success / workflow passed
- `1`: Failure / workflow failed or command error
- `2`: Command-specific errors (e.g., not found)

When using `gh run watch --exit-status`, the exit code reflects the run's conclusion.
