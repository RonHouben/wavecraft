//! Development server command - starts WebSocket + UI dev servers.

use anyhow::{Context, Result};
use console::style;
use std::io::{self, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::project::{has_node_modules, ProjectMarkers};

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
        .context("Failed to run npm install. Is npm installed and in your PATH?")?;

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
        style("Starting Wavecraft Development Servers")
            .cyan()
            .bold()
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
        .context("Failed to start WebSocket server. Is cargo installed?")?;

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
    // Send SIGTERM to process group (negative PID kills the group)
    let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);
    thread::sleep(Duration::from_millis(500));
    // Force kill if still running
    let _ = child.kill();
    Ok(())
}

/// Kill a child process on Windows.
#[cfg(windows)]
fn kill_process(mut child: Child) -> Result<()> {
    let _ = child.kill();
    Ok(())
}
