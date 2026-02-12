//! Subprocess-based parameter extraction.
//!
//! Spawns `wavecraft extract-params` as a subprocess to isolate the `dlopen`
//! call. This prevents macOS static initializers in nih-plug dependencies from
//! hanging the parent process.

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use wavecraft_protocol::ParameterInfo;

/// Default timeout for subprocess parameter extraction.
pub const DEFAULT_EXTRACT_TIMEOUT: Duration = Duration::from_secs(30);

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
    // Resolve the path to the current wavecraft binary
    let self_exe = std::env::current_exe().context("Failed to determine wavecraft binary path")?;

    // Spawn the extraction subprocess
    let mut child = tokio::process::Command::new(&self_exe)
        .arg("extract-params")
        .arg(dylib_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "Failed to spawn parameter extraction subprocess: {}",
                self_exe.display()
            )
        })?;

    // Take ownership of stdout/stderr before consuming child
    let mut stdout = child.stdout.take().expect("stdout not captured");
    let mut stderr = child.stderr.take().expect("stderr not captured");

    // Wait for the subprocess with timeout (using wait() not wait_with_output())
    match tokio::time::timeout(timeout, child.wait()).await {
        // Subprocess completed within timeout
        Ok(Ok(status)) => {
            // Read stdout and stderr
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();

            use tokio::io::AsyncReadExt;
            stdout
                .read_to_end(&mut stdout_buf)
                .await
                .context("Failed to read subprocess stdout")?;
            stderr
                .read_to_end(&mut stderr_buf)
                .await
                .context("Failed to read subprocess stderr")?;

            if status.success() {
                // Parse stdout as JSON
                let stdout_str = String::from_utf8(stdout_buf)
                    .context("Subprocess stdout was not valid UTF-8")?;

                let params: Vec<ParameterInfo> = serde_json::from_str(stdout_str.trim())
                    .context("Failed to parse parameter JSON from subprocess")?;

                Ok(params)
            } else {
                // Subprocess exited with error
                let stderr_str = String::from_utf8_lossy(&stderr_buf);
                let code = status.code().unwrap_or(-1);

                anyhow::bail!(
                    "Parameter extraction failed (exit code {}):\n{}",
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

            anyhow::bail!(
                "Parameter extraction timed out after {}s. \
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
                timeout.as_secs(),
                dylib_path.display(),
                dylib_path.display()
            );
        }
    }
}
