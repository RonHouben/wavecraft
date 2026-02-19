use anyhow::{Context, Result};
use console::style;
use std::io::{self, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};

use crate::project::{has_node_modules, ProjectMarkers};

pub(super) fn ensure_dependencies(
    project: &ProjectMarkers,
    install: bool,
    no_install: bool,
) -> Result<()> {
    if !has_node_modules(project) {
        if no_install {
            anyhow::bail!(
                "Dependencies not installed. Run `npm install` in the ui/ directory,\n\
                 or use `wavecraft start --install` to install automatically."
            );
        }

        let should_install = if install { true } else { prompt_install()? };

        if should_install {
            install_dependencies(project)?;
        } else {
            anyhow::bail!("Cannot start without dependencies. Run `npm install` in ui/ first.");
        }
    }

    Ok(())
}

pub(super) fn ensure_ports_available(ws_port: u16, ui_port: u16) -> Result<()> {
    ensure_port_available(ws_port, "WebSocket server", "--port")?;
    ensure_port_available(ui_port, "UI dev server", "--ui-port")?;
    Ok(())
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

fn ensure_port_available(port: u16, label: &str, flag: &str) -> Result<()> {
    match TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(err) => anyhow::bail!(
            "{} port {} is already in use ({}). Stop the process using it or run `wavecraft start {}` with a free port.",
            label,
            port,
            err,
            flag
        ),
    }
}
