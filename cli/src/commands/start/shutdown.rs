use anyhow::{Context, Result};
use command_group::GroupChild;
use console::style;
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::watch;

/// Set up Ctrl+C handler and wait for shutdown.
///
/// Audio runs in-process (via FFI) on the tokio runtime's thread pool,
/// so dropping the runtime is sufficient to stop audio. Only the UI
/// child process needs explicit cleanup.
#[derive(Debug)]
pub(super) enum ShutdownReason {
    CtrlC,
    UiExited(i32),
    UiExitedUnknown,
    ChannelClosed,
}

pub(super) fn wait_for_shutdown(
    mut ui_server: GroupChild,
    shutdown_tx: watch::Sender<bool>,
) -> Result<ShutdownReason> {
    let (tx, rx) = mpsc::channel();
    let shutdown_tx_for_handler = shutdown_tx.clone();

    ctrlc::set_handler(move || {
        let _ = shutdown_tx_for_handler.send(true);
        let _ = tx.send(());
    })
    .context("Failed to set Ctrl+C handler")?;

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(_) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                send_shutdown_signal(&shutdown_tx);
                kill_process(&mut ui_server);
                println!("{} Servers stopped", style("✓").green());
                return Ok(ShutdownReason::CtrlC);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check UI server
                if let Some(status) = ui_server
                    .try_wait()
                    .context("Failed to check UI dev server status")?
                {
                    println!();
                    println!(
                        "{} UI dev server exited unexpectedly ({}).",
                        style("✗").red(),
                        status
                    );
                    println!("{} Shutting down servers...", style("→").cyan());
                    send_shutdown_signal(&shutdown_tx);
                    println!("{} Servers stopped", style("✓").green());
                    if let Some(code) = status.code() {
                        return Ok(ShutdownReason::UiExited(code));
                    }
                    return Ok(ShutdownReason::UiExitedUnknown);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                send_shutdown_signal(&shutdown_tx);
                kill_process(&mut ui_server);
                println!("{} Servers stopped", style("✓").green());
                return Ok(ShutdownReason::ChannelClosed);
            }
        }
    }
}

fn send_shutdown_signal(shutdown_tx: &watch::Sender<bool>) {
    let _ = shutdown_tx.send(true);
}

/// Kill a child process group gracefully.
fn kill_process(child: &mut GroupChild) {
    let _ = child.kill();
    let _ = child.wait();
}
