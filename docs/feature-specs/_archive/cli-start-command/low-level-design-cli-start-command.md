# Low-Level Design: CLI Start Command (`wavecraft start`)

**Status:** Draft  
**Created:** 2026-02-06  
**Author:** Architect Agent

---

## Problem Statement

After scaffolding a plugin with `wavecraft create`, users are told to run `cargo xtask dev`. This has several issues:

1. **Inconsistent naming** — Users learn `wavecraft` commands, then switch to `cargo xtask`
2. **Template complexity** — The scaffolded xtask only has `bundle`, not `dev`
3. **Missing dependency check** — Users forget `npm install` and get confusing errors
4. **Discovery** — `xtask` is an internal pattern, not a user-facing command

The solution is a `wavecraft start` command that provides a unified, intelligent development experience.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      wavecraft start                                    │
└─────────────────────────────────────────────────────────────────────────┘

            wavecraft start [--port 9000] [--ui-port 5173]
                              │
                              ▼
                    ┌─────────────────────┐
                    │  Project Detection  │
                    │  (is this a         │
                    │   Wavecraft plugin?)│
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │ No             │ Yes            │
              ▼                ▼                │
       Error: "Not a    ┌─────────────────┐     │
        Wavecraft       │ Dependency Check│     │
        project"        │ (node_modules?) │     │
                        └────────┬────────┘     │
                                 │              │
                    ┌────────────┼──────────────┤
                    │ Missing    │ Present      │
                    ▼            ▼              │
            ┌─────────────┐  ┌──────────────┐   │
            │ Prompt user │  │ Start        │   │
            │ "npm install│  │ servers      │◄──┘
            │  now?"      │  │              │
            └──────┬──────┘  └──────┬───────┘
                   │                │
          ┌────────┴───────┐       │
          │ Yes    │ No    │       │
          ▼        ▼       │       │
      Run npm   Abort with │       │
      install   message    │       │
          │                │       │
          └────────────────┴───────┘
                           │
                           ▼
              ┌────────────────────────┐
              │  Spawn Processes       │
              │  ├─ Standalone server  │
              │  └─ Vite dev server    │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │  Signal Handler        │
              │  Ctrl+C → graceful     │
              │  shutdown              │
              └────────────────────────┘
```

---

## Module Structure

```
cli/src/
├── main.rs                 # Add Start variant to Commands enum
├── commands/
│   ├── mod.rs              # Add: pub mod start;
│   ├── new.rs              # Update next steps message
│   └── start.rs            # NEW: Start command implementation
└── project/                # NEW: Project detection module
    ├── mod.rs
    └── detection.rs        # Wavecraft project detection logic
```

### New Module: `cli/src/project/detection.rs`

```rust
//! Wavecraft project detection utilities.
//!
//! Determines if the current directory (or a specified path) is a valid
//! Wavecraft plugin project by checking for required structure.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Markers that identify a Wavecraft plugin project.
pub struct ProjectMarkers {
    /// Path to ui/ directory
    pub ui_dir: PathBuf,
    /// Path to engine/ directory  
    pub engine_dir: PathBuf,
    /// Path to ui/package.json
    pub ui_package_json: PathBuf,
    /// Path to engine/Cargo.toml
    pub engine_cargo_toml: PathBuf,
}

impl ProjectMarkers {
    /// Detect project markers starting from the given directory.
    ///
    /// Returns `Ok(ProjectMarkers)` if this is a valid Wavecraft project,
    /// or an error describing what's missing.
    pub fn detect(start_dir: &Path) -> Result<Self> {
        let ui_dir = start_dir.join("ui");
        let engine_dir = start_dir.join("engine");
        let ui_package_json = ui_dir.join("package.json");
        let engine_cargo_toml = engine_dir.join("Cargo.toml");

        // Check required directories
        if !ui_dir.is_dir() {
            bail!(
                "Not a Wavecraft project: missing 'ui/' directory.\n\
                 Run this command from a plugin project created with `wavecraft create`."
            );
        }

        if !engine_dir.is_dir() {
            bail!(
                "Not a Wavecraft project: missing 'engine/' directory.\n\
                 Run this command from a plugin project created with `wavecraft create`."
            );
        }

        // Check required files
        if !ui_package_json.is_file() {
            bail!("Invalid project structure: missing 'ui/package.json'");
        }

        if !engine_cargo_toml.is_file() {
            bail!("Invalid project structure: missing 'engine/Cargo.toml'");
        }

        // Optional: Verify this is a Wavecraft project by checking for wavecraft dependency
        // in Cargo.toml (could add this check later for stricter validation)

        Ok(Self {
            ui_dir,
            engine_dir,
            ui_package_json,
            engine_cargo_toml,
        })
    }
}

/// Check if UI dependencies are installed.
pub fn has_node_modules(project: &ProjectMarkers) -> bool {
    project.ui_dir.join("node_modules").is_dir()
}
```

### New Module: `cli/src/commands/start.rs`

```rust
//! Development server command - starts WebSocket + UI dev servers.

use anyhow::{Context, Result};
use console::style;
use std::io::{self, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::project::detection::{has_node_modules, ProjectMarkers};

/// Default WebSocket server port
pub const DEFAULT_WS_PORT: u16 = 9000;

/// Default Vite dev server port
pub const DEFAULT_UI_PORT: u16 = 5173;

/// Options for the `start` command.
#[derive(Debug)]
pub struct StartCommand {
    /// WebSocket server port
    pub port: u16,
    /// Vite UI server port
    pub ui_port: u16,
    /// Auto-install dependencies without prompting
    pub install: bool,
    /// Fail if dependencies are missing (no prompt)
    pub no_install: bool,
    /// Show verbose output
    pub verbose: bool,
}

impl StartCommand {
    pub fn execute(&self) -> Result<()> {
        // 1. Detect project
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project = ProjectMarkers::detect(&cwd)?;

        // 2. Check dependencies
        if !has_node_modules(&project) {
            if self.no_install {
                anyhow::bail!(
                    "Dependencies not installed. Run `npm install` in the ui/ directory,\n\
                     or use `wavecraft start --install` to install automatically."
                );
            }

            let should_install = if self.install {
                true
            } else {
                prompt_install()?
            };

            if should_install {
                install_dependencies(&project)?;
            } else {
                anyhow::bail!("Cannot start without dependencies. Run `npm install` in ui/ first.");
            }
        }

        // 3. Start servers
        run_dev_servers(&project, self.port, self.ui_port, self.verbose)
    }
}

/// Prompt user to install dependencies.
fn prompt_install() -> Result<bool> {
    print!(
        "{} Dependencies not installed. Run npm install? [Y/n] ",
        style("?").cyan().bold()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response.is_empty() || response == "y" || response == "yes")
}

/// Install npm dependencies in the ui/ directory.
fn install_dependencies(project: &ProjectMarkers) -> Result<()> {
    println!("{} Installing dependencies...", style("→").cyan());

    let status = Command::new("npm")
        .args(["install"])
        .current_dir(&project.ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run npm install")?;

    if !status.success() {
        anyhow::bail!("npm install failed. Please check the output above and try again.");
    }

    println!("{} Dependencies installed", style("✓").green());
    Ok(())
}

/// Run both development servers.
fn run_dev_servers(
    project: &ProjectMarkers,
    ws_port: u16,
    ui_port: u16,
    verbose: bool,
) -> Result<()> {
    println!();
    println!(
        "{}",
        style("Starting Wavecraft Development Servers").cyan().bold()
    );
    println!();

    // Start WebSocket server
    println!(
        "{} Starting WebSocket server on port {}...",
        style("→").cyan(),
        ws_port
    );

    let ws_port_str = ws_port.to_string();
    let mut ws_args = vec![
        "run",
        "-p",
        "standalone",
        "--release",
        "--",
        "--dev-server",
        "--port",
        &ws_port_str,
    ];
    if verbose {
        ws_args.push("--verbose");
    }

    let ws_server = Command::new("cargo")
        .args(&ws_args)
        .current_dir(&project.engine_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start WebSocket server")?;

    // Give the server time to start
    thread::sleep(Duration::from_millis(500));

    // Start UI dev server
    println!(
        "{} Starting UI dev server on port {}...",
        style("→").cyan(),
        ui_port
    );

    let ui_port_str = format!("--port={}", ui_port);
    let ui_server = Command::new("npm")
        .args(["run", "dev", "--", &ui_port_str])
        .current_dir(&project.ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start UI dev server")?;

    // Print success message
    println!();
    println!("{}", style("✓ Both servers running!").green().bold());
    println!();
    println!("  WebSocket: ws://127.0.0.1:{}", ws_port);
    println!("  UI:        http://localhost:{}", ui_port);
    println!();
    println!("{}", style("Press Ctrl+C to stop").dim());
    println!();

    // Wait for shutdown
    wait_for_shutdown(ws_server, ui_server)
}

/// Set up Ctrl+C handler and wait for shutdown.
fn wait_for_shutdown(ws_server: Child, ui_server: Child) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .context("Failed to set Ctrl+C handler")?;

    // Wait for Ctrl+C
    let _ = rx.recv();

    println!();
    println!("{} Shutting down servers...", style("→").cyan());

    // Kill both servers
    kill_process(ws_server)?;
    kill_process(ui_server)?;

    println!("{} Servers stopped", style("✓").green());
    Ok(())
}

/// Kill a child process gracefully.
#[cfg(unix)]
fn kill_process(mut child: Child) -> Result<()> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let pid = child.id();
    // Send SIGTERM to process group
    let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);
    thread::sleep(Duration::from_millis(500));
    // Force kill if still running
    let _ = child.kill();
    Ok(())
}

#[cfg(windows)]
fn kill_process(mut child: Child) -> Result<()> {
    let _ = child.kill();
    Ok(())
}
```

### Update: `cli/src/main.rs`

Add the `Start` command variant:

```rust
mod commands;
mod project;  // NEW
mod template;
mod validation;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use commands::{NewCommand, StartCommand};  // Updated

// ... existing code ...

#[derive(Subcommand)]
enum Commands {
    /// Create a new plugin project from the template
    New {
        // ... existing fields ...
    },
    
    /// Start development servers for UI iteration
    Start {
        /// WebSocket server port (default: 9000)
        #[arg(short, long, default_value_t = 9000)]
        port: u16,

        /// Vite UI server port (default: 5173)
        #[arg(long, default_value_t = 5173)]
        ui_port: u16,

        /// Auto-install npm dependencies without prompting
        #[arg(long, conflicts_with = "no_install")]
        install: bool,

        /// Fail if dependencies missing (no prompt)
        #[arg(long, conflicts_with = "install")]
        no_install: bool,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { ... } => { ... },
        
        Commands::Start {
            port,
            ui_port,
            install,
            no_install,
            verbose,
        } => {
            let cmd = StartCommand {
                port,
                ui_port,
                install,
                no_install,
                verbose,
            };
            cmd.execute()?;
        }
    }

    Ok(())
}
```

---

## Dependencies Update

Add to `cli/Cargo.toml`:

```toml
[dependencies]
# ... existing ...
ctrlc = "3"                    # Ctrl+C handling

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["signal"] }
```

---

## CLI Interface

### Command Help

```
$ wavecraft start --help
Start development servers for UI iteration

Usage: wavecraft start [OPTIONS]

Options:
  -p, --port <PORT>     WebSocket server port [default: 9000]
      --ui-port <PORT>  Vite UI server port [default: 5173]
      --install         Auto-install npm dependencies without prompting
      --no-install      Fail if dependencies missing (no prompt)
  -v, --verbose         Show verbose output
  -h, --help            Print help
```

### Example Invocations

```bash
# Default: ports 9000 + 5173, prompt if deps missing
wavecraft start

# Custom ports
wavecraft start --port 8000 --ui-port 3000

# CI/scripting: auto-install deps
wavecraft start --install

# CI/scripting: fail fast if deps missing
wavecraft start --no-install

# Debug mode
wavecraft start --verbose
```

---

## Update "Next Steps" Output

Change in `cli/src/commands/new.rs`:

```rust
// Before
println!("  cargo xtask dev    # Start development servers");

// After  
println!("  wavecraft start    # Start development servers");
```

---

## Error Messages

### Not a Wavecraft Project

```
$ wavecraft start
Error: Not a Wavecraft project: missing 'ui/' directory.
Run this command from a plugin project created with `wavecraft create`.
```

### Dependencies Missing (with --no-install)

```
$ wavecraft start --no-install
Error: Dependencies not installed. Run `npm install` in the ui/ directory,
or use `wavecraft start --install` to install automatically.
```

### npm install Failed

```
$ wavecraft start
? Dependencies not installed. Run npm install? [Y/n] y
→ Installing dependencies...
npm ERR! code ERESOLVE
...
Error: npm install failed. Please check the output above and try again.
```

---

## Process Management

### Signal Handling

The command uses `ctrlc` crate for cross-platform Ctrl+C handling:

1. Both servers are spawned as child processes
2. Main thread blocks on the signal channel
3. On Ctrl+C:
   - Unix: Send SIGTERM to process groups (kills child processes too)
   - Windows: Call `child.kill()` on each process

### Process Lifetime

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Process Lifecycle                                   │
└─────────────────────────────────────────────────────────────────────────┘

Time ─────────────────────────────────────────────────────────────────────►

wavecraft start
    │
    ├─► spawn(cargo run -p standalone)  ────────────────────┐
    │                                                       │
    │   sleep(500ms)                                        │
    │                                                       │
    ├─► spawn(npm run dev)  ────────────────────────────────┤
    │                                                       │
    │   print("Both servers running!")                      │
    │                                                       │
    │   recv(ctrl_c_channel)  ◄─────────── [user: Ctrl+C]   │
    │                                                       │
    ├─► kill(ws_server)  ──────────────────────────────────►X
    │                                                       │
    └─► kill(ui_server)  ──────────────────────────────────►X
    
    exit(0)
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_project_detection_valid() {
        let tmp = TempDir::new().unwrap();
        
        // Create valid project structure
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();
        
        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_detection_missing_ui() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        
        let result = ProjectMarkers::detect(tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing 'ui/'"));
    }

    #[test]
    fn test_has_node_modules_false() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("ui")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();
        
        let project = ProjectMarkers::detect(tmp.path()).unwrap();
        assert!(!has_node_modules(&project));
    }

    #[test]
    fn test_has_node_modules_true() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("ui/node_modules")).unwrap();
        fs::create_dir_all(tmp.path().join("engine")).unwrap();
        fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
        fs::write(tmp.path().join("engine/Cargo.toml"), "[package]").unwrap();
        
        let project = ProjectMarkers::detect(tmp.path()).unwrap();
        assert!(has_node_modules(&project));
    }
}
```

### Integration Tests (Manual)

| Test Case | Steps | Expected |
|-----------|-------|----------|
| Happy path | `wavecraft create test && cd test && wavecraft start --install` | Both servers start, URLs printed |
| No project | `cd /tmp && wavecraft start` | Error: "Not a Wavecraft project" |
| Missing deps | `wavecraft create test && cd test && wavecraft start --no-install` | Error about missing deps |
| Custom ports | `wavecraft start -p 8000 --ui-port 3000` | Servers on specified ports |
| Ctrl+C | Start servers, press Ctrl+C | Both servers stop cleanly |

---

## Migration Notes

### Existing Users

Users who learned `cargo xtask dev` can continue using it if they're in the monorepo. For scaffolded projects, `wavecraft start` becomes the standard.

### Template Updates

No changes needed to the template's xtask — it stays focused on `bundle`. The `start` command lives in the global CLI.

### Documentation Updates

Update these docs after implementation:
- `README.md` — Getting started section
- `docs/guides/sdk-getting-started.md` — Development workflow

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| npm not in PATH | Low | Medium | Clear error message suggesting npm install |
| Port conflict | Medium | Low | User can specify custom ports |
| Process cleanup on crash | Low | Low | OS cleans up orphaned processes |
| Windows process group handling | Medium | Medium | Fallback to individual kill() |

---

## Alternatives Considered

### 1. Add `dev` to Template xtask

**Rejected** — Duplicates code in every scaffolded project. Updates require re-scaffolding.

### 2. Keep `cargo xtask dev` Only

**Rejected** — Poor UX for new users learning the CLI.

### 3. Use `wavecraft dev` Instead of `wavecraft start`

**Considered** — `start` is more intuitive for "starting the development environment". `dev` could imply "development build" vs "production build". Either works; `start` chosen for clarity.

---

## Related Changes: Relaxed Plugin Name Validation

As part of this UX improvement, we also relax the `wavecraft create` name validation rules.

### Before (Too Strict)

```
$ wavecraft create myCoolPlugin
Error: Invalid plugin name 'myCoolPlugin'. Must start with a lowercase letter
and contain only lowercase letters, numbers, hyphens, or underscores.
```

### After (Flexible)

```
$ wavecraft create myCoolPlugin
✓ Plugin project created successfully!
```

### Validation Rules

| Rule | Before | After |
|------|--------|-------|
| Starting character | Lowercase letter only | Any letter (upper or lower) |
| Body characters | `[a-z0-9_-]` | `[a-zA-Z0-9_-]` |
| Reserved keywords | Blocked (match, async, etc.) | Blocked (unchanged) |
| Std library names | Blocked (std, core, etc.) | Blocked (unchanged) |

### Rationale

1. **Cargo allows mixed case** — crates.io treats names case-insensitively but allows mixed case
2. **Common naming patterns** — `myCoolPlugin`, `MyPlugin`, `my-plugin` are all valid
3. **Less friction** — Users shouldn't have to think about case rules

### Updated Regex

```rust
// Before
let pattern = Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();

// After
let pattern = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap();
```

### Test Cases

**Valid names:**
- `my-plugin` ✓
- `my_plugin` ✓
- `MyPlugin` ✓
- `myCoolPlugin` ✓
- `My-Cool-Plugin` ✓

**Invalid names:**
- `123plugin` ✗ (starts with number)
- `-plugin` ✗ (starts with hyphen)
- `std` ✗ (reserved)
- `match` ✗ (Rust keyword)

---

## Open Questions

1. **Future: `wavecraft build`?** — Should we add a build command too, or leave that to xtask?
2. **Monorepo detection?** — Should we detect if running in the SDK monorepo and suggest `cargo xtask dev` instead?
3. **Auto-open browser?** — Should we add `--open` flag to open UI in default browser?

---

## Checklist for Implementation

- [x] Relax plugin name validation to allow mixed case
- [ ] Create `cli/src/project/mod.rs` and `detection.rs`
- [ ] Create `cli/src/commands/start.rs`
- [ ] Update `cli/src/commands/mod.rs` to export `StartCommand`
- [ ] Update `cli/src/main.rs` with `Start` variant
- [ ] Update `cli/Cargo.toml` with new dependencies
- [ ] Update `cli/src/commands/new.rs` next steps message
- [ ] Add unit tests for project detection
- [ ] Bump version to 0.8.0
- [ ] Manual testing on scaffolded project
- [ ] Update documentation
