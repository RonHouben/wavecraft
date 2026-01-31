# Skill: Run GitHub Actions Locally

Use this skill to test GitHub Actions workflows locally before pushing to GitHub.

## Prerequisites

- **Docker Desktop** must be installed and running
- **act** must be installed: `brew install act`

## Commands

### List Available Jobs

```bash
act -l
```

### Run All Workflows

```bash
act
```

### Run Specific Workflow

```bash
act -W .github/workflows/ci.yml
```

### Run Specific Job

```bash
act -j <job-name>
```

### Run on Specific Event

```bash
act push
act pull_request
```

### Dry Run (Preview Without Executing)

```bash
act -n
```

### Pass Secrets

```bash
# Use GitHub CLI token
act -s GITHUB_TOKEN="$(gh auth token)"

# Use secrets file (.secrets with KEY=value format)
act --secret-file .secrets
```

### Use Smaller Docker Images (Faster)

```bash
act -P ubuntu-latest=catthehacker/ubuntu:act-22.04
```

## Limitations

### macOS Runners Cannot Be Emulated

GitHub's `macos-latest` runners cannot be simulated locally. For workflows with macOS jobs:

1. **Substitute with Linux** (won't work for macOS-specific steps):
   ```bash
   act -P macos-latest=catthehacker/ubuntu:act-latest
   ```

2. **Run only Linux jobs**:
   ```bash
   act -j lint  # Run specific Linux job
   ```

3. **Test macOS commands manually** on your Mac

### First Run Downloads Large Images

The first run downloads ~12GB Docker images. Be patient or use smaller images.

## Workflow for This Project

Since VstKit CI has macOS-specific jobs (signing, notarization, Rust builds), use this approach:

1. **Test lint jobs locally** (if running on ubuntu):
   ```bash
   act -j lint
   ```

2. **For macOS jobs**, either:
   - Push to GitHub and check CI results
   - Run the commands manually (e.g., `cargo xtask lint`, `cargo xtask test`)

## Debugging Failed Runs

```bash
# Verbose output
act -v

# Very verbose output
act -vv

# Keep containers running after failure for inspection
act --reuse
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Docker not running | Start Docker Desktop |
| Permission denied | Run `docker ps` to verify Docker access |
| Job uses `macos-latest` | Skip job or substitute with Linux image |
| Missing secrets | Use `-s` flag or `--secret-file` |
| Image pull fails | Check internet connection, try smaller image |
