//! Subprocess-based parameter extraction.
//!
//! Spawns `wavecraft extract-params` as a subprocess to isolate the `dlopen`
//! call. This prevents macOS static initializers in nih-plug dependencies from
//! hanging the parent process.

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use wavecraft_protocol::ParameterInfo;
use wavecraft_protocol::ProcessorInfo;

/// Default timeout for subprocess parameter extraction.
pub const DEFAULT_EXTRACT_TIMEOUT: Duration = Duration::from_secs(30);

fn spawn_extraction_subprocess(
    self_exe: &Path,
    subcommand: &str,
    dylib_path: &Path,
) -> Result<tokio::process::Child> {
    tokio::process::Command::new(self_exe)
        .arg(subcommand)
        .arg(dylib_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "Failed to spawn {} subprocess: {}",
                subcommand,
                self_exe.display()
            )
        })
}

fn take_subprocess_pipes(
    child: &mut tokio::process::Child,
) -> Result<(tokio::process::ChildStdout, tokio::process::ChildStderr)> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture subprocess stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture subprocess stderr"))?;

    Ok((stdout, stderr))
}

async fn read_subprocess_pipes(
    stdout: &mut tokio::process::ChildStdout,
    stderr: &mut tokio::process::ChildStderr,
) -> Result<(Vec<u8>, Vec<u8>)> {
    use tokio::io::AsyncReadExt;

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    stdout
        .read_to_end(&mut stdout_buf)
        .await
        .context("Failed to read subprocess stdout")?;
    stderr
        .read_to_end(&mut stderr_buf)
        .await
        .context("Failed to read subprocess stderr")?;

    Ok((stdout_buf, stderr_buf))
}

fn timeout_diagnostics(subcommand: &str, timeout: Duration, dylib_path: &Path) -> String {
    format!(
        "{} timed out after {}s. \
         This is likely caused by macOS static initializers in the plugin dylib. \
         The plugin was built with --features _param-discovery but dlopen \
         still hung. Check for transitive dependencies that register with \
         macOS system services during library load.\n\
         \n\
         Suggested actions:\n\
         • Check for new transitive dependencies in Cargo.toml\n\
         • Try: nm -gU {} | grep _init  (look for unexpected initializers)\n\
         • File a bug with the offending dependency\n\
         \n\
         Dylib: {}",
        subcommand,
        timeout.as_secs(),
        dylib_path.display(),
        dylib_path.display()
    )
}

async fn extract_via_subprocess(
    subcommand: &str,
    dylib_path: &Path,
    timeout: Duration,
) -> Result<String> {
    // Resolve the path to the current wavecraft binary
    let self_exe = std::env::current_exe().context("Failed to determine wavecraft binary path")?;

    // Spawn the extraction subprocess
    let mut child = spawn_extraction_subprocess(&self_exe, subcommand, dylib_path)?;

    // Take ownership of stdout/stderr before consuming child
    let (mut stdout, mut stderr) = take_subprocess_pipes(&mut child)?;

    // Wait for the subprocess with timeout (using wait() not wait_with_output())
    match tokio::time::timeout(timeout, child.wait()).await {
        // Subprocess completed within timeout
        Ok(Ok(status)) => {
            // Read stdout and stderr
            let (stdout_buf, stderr_buf) = read_subprocess_pipes(&mut stdout, &mut stderr).await?;

            if status.success() {
                // Parse stdout as JSON
                let stdout_str = String::from_utf8(stdout_buf)
                    .context("Subprocess stdout was not valid UTF-8")?;

                Ok(stdout_str)
            } else {
                // Subprocess exited with error
                let stderr_str = String::from_utf8_lossy(&stderr_buf);
                let code = status.code().unwrap_or(-1);

                anyhow::bail!(
                    "{} failed (exit code {}):\n{}",
                    subcommand,
                    code,
                    stderr_str
                );
            }
        }

        // Failed to wait for subprocess
        Ok(Err(e)) => {
            anyhow::bail!("Failed to wait for extraction subprocess: {}", e);
        }

        // Timeout — kill the subprocess
        Err(_) => {
            // Now we can kill because child wasn't consumed
            let _ = child.kill().await;

            anyhow::bail!("{}", timeout_diagnostics(subcommand, timeout, dylib_path));
        }
    }
}

/// Extract parameters from a plugin dylib via a subprocess.
///
/// Spawns `wavecraft extract-params <dylib_path>` and parses JSON from stdout.
/// The subprocess is killed if it exceeds the timeout.
///
/// # Safety
///
/// This isolates `dlopen` from the parent process, preventing hangs caused by
/// macOS static initializers in nih-plug dependencies. The subprocess can be
/// forcefully terminated with `SIGKILL` on timeout.
///
/// # Arguments
///
/// * `dylib_path` - Path to the plugin dylib to extract parameters from
/// * `timeout` - Maximum time to wait before killing the subprocess
///
/// # Returns
///
/// * `Ok(Vec<ParameterInfo>)` - Successfully extracted parameters
/// * `Err(_)` - Timeout, subprocess crashed, or invalid output
pub async fn extract_params_subprocess(
    dylib_path: &Path,
    timeout: Duration,
) -> Result<Vec<ParameterInfo>> {
    let json = extract_via_subprocess("extract-params", dylib_path, timeout).await?;
    serde_json::from_str(json.trim()).context("Failed to parse parameter JSON from subprocess")
}

/// Extract processor metadata from a plugin dylib via subprocess isolation.
pub async fn extract_processors_subprocess(
    dylib_path: &Path,
    timeout: Duration,
) -> Result<Vec<ProcessorInfo>> {
    let json = extract_via_subprocess("extract-processors", dylib_path, timeout).await?;
    serde_json::from_str(json.trim()).context("Failed to parse processor JSON from subprocess")
}
