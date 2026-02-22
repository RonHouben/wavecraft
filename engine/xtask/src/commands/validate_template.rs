//! Template validation command - Validates that the CLI generates working projects.
//!
//! This command replicates the GitHub Actions template-validation.yml workflow
//! locally for faster iteration. It:
//! 1. Builds the CLI
//! 2. Generates a test plugin project
//! 3. Validates the generated code compiles and passes linting
//!
//! This is the local equivalent of the CI template-validation workflow.

use anyhow::{Context, Result, bail};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, Output, Stdio};
use std::time::Instant;
use std::{env, fs};
use std::{thread, time::Duration};

#[cfg(unix)]
use nix::libc;
#[cfg(unix)]
use nix::sys::signal::{Signal, killpg};
#[cfg(unix)]
use nix::unistd::{Pid, setpgid};
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::process::CommandExt;

use xtask::output::*;

const TEST_PLUGIN_INPUT_NAME: &str = "myPlugin";
const TEST_PLUGIN_INTERNAL_ID: &str = "my_plugin";
const START_SMOKE_TIMEOUT_SECS: u64 = 180;
const START_SMOKE_WS_PORT: u16 = 39091;
const START_SMOKE_UI_PORT: u16 = 39092;
const START_SMOKE_KILL_GRACE_MS: u64 = 2_000;
const START_SMOKE_POST_EXIT_DRAIN_ATTEMPTS: usize = 5;
const START_SMOKE_POST_EXIT_DRAIN_SLEEP_MS: u64 = 25;

/// Template validation configuration.
#[derive(Debug, Clone, Default)]
pub struct ValidateTemplateConfig {
    /// Show verbose output
    pub verbose: bool,
    /// Keep the generated test project (don't clean up)
    pub keep: bool,
}

/// Run the template validation command.
pub fn run(config: ValidateTemplateConfig) -> Result<()> {
    let start_time = Instant::now();

    print_header("Wavecraft Template Validation");
    println!();
    println!("Replicating CI template-validation.yml workflow locally");
    println!();

    // Get paths
    let workspace_root = xtask::paths::project_root()?;
    let cli_dir = workspace_root.join("cli");
    let parent_dir = env::temp_dir();
    let validation_root_dir = parent_dir.join("wavecraft-template-validation");
    let local_sdk_project_dir = validation_root_dir.join("test-plugin-local-sdk");
    let git_source_project_dir = validation_root_dir.join("test-plugin-git-source");
    let installed_cli_dir = validation_root_dir.join("installed-cli");
    let installed_like_cli_binary = installed_cli_dir.join("wavecraft");

    // Clean up any existing validation directory
    if validation_root_dir.exists() {
        if config.verbose {
            println!("Cleaning up existing validation workspace...");
        }
        fs::remove_dir_all(&validation_root_dir)
            .context("Failed to remove existing validation workspace")?;
    }

    fs::create_dir_all(&validation_root_dir)
        .context("Failed to create validation workspace directory")?;

    // Ensure cleanup happens even on error (unless --keep is specified)
    let cleanup_guard = CleanupGuard::new(validation_root_dir.clone(), config.keep, config.verbose);

    // Step 1: Build CLI
    print_phase("Step 1: Build Wavecraft CLI");
    build_cli(&cli_dir, config.verbose)?;
    print_success("CLI built successfully");
    println!();

    // Step 2: Generate local-sdk validation project
    print_phase("Step 2: Generate Local-SDK Test Plugin");
    let cli_binary = cli_dir.join("target/release/wavecraft");
    generate_test_plugin(
        &cli_binary,
        &workspace_root,
        &local_sdk_project_dir,
        true,
        config.verbose,
    )?;
    verify_generated_files(&local_sdk_project_dir)?;
    verify_generated_engine_identifiers(&local_sdk_project_dir)?;

    normalize_generated_ui_dependencies(&local_sdk_project_dir, &workspace_root)?;

    print_success("Local-SDK test plugin generated successfully");
    println!();

    // Step 3: Generate git-source validation project (installed-CLI simulation)
    print_phase("Step 3: Generate Git-Source Test Plugin");
    copy_cli_binary_for_git_source(&cli_binary, &installed_like_cli_binary)?;
    generate_test_plugin(
        &installed_like_cli_binary,
        &workspace_root,
        &git_source_project_dir,
        false,
        config.verbose,
    )?;
    verify_generated_files(&git_source_project_dir)?;
    verify_generated_engine_identifiers(&git_source_project_dir)?;

    print_success("Git-source test plugin generated successfully");
    println!();

    // Step 4: Validate local-sdk engine/UI/bundle path
    print_phase("Step 4: Validate Local-SDK Engine + UI + Bundle");
    validate_engine(&local_sdk_project_dir, config.verbose)?;
    validate_ui(&local_sdk_project_dir, config.verbose)?;
    validate_cli_bundle_contract(&cli_binary, &local_sdk_project_dir, config.verbose)?;

    print_success("Local-SDK validation passed");
    println!();

    // Step 5: Validate git-source startup path
    print_phase("Step 5: Validate Git-Source Startup Path");
    apply_git_source_validation_overrides(&git_source_project_dir, &workspace_root)?;
    validate_git_source_engine_modes(&git_source_project_dir, config.verbose)?;
    install_ui_dependencies(&git_source_project_dir, config.verbose)?;
    validate_git_source_start_smoke(
        &installed_like_cli_binary,
        &git_source_project_dir,
        START_SMOKE_WS_PORT,
        START_SMOKE_UI_PORT,
        config.verbose,
    )?;

    print_success("Git-source startup validation passed");
    println!();

    // Print summary
    let duration = start_time.elapsed();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status("Summary");
    println!();
    print_success_item(&format!(
        "✅ All validation checks passed ({:.1}s)",
        duration.as_secs_f64()
    ));
    print_success_item("✅ Local-SDK generated project validated (engine/ui/bundle)");
    print_success_item(
        "✅ Git-source generated project validated (_param-discovery/default/start)",
    );
    println!();
    print_success("Template validation successful!");

    // CleanupGuard will handle cleanup on drop (unless --keep is specified)
    drop(cleanup_guard);

    Ok(())
}

/// Build the Wavecraft CLI in release mode.
fn build_cli(cli_dir: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Building CLI in release mode...");
    }

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(cli_dir)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Failed to build CLI");
    }

    Ok(())
}

/// Generate a test plugin project using the CLI.
fn generate_test_plugin(
    cli_binary: &Path,
    workspace_root: &Path,
    output_dir: &Path,
    local_sdk: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Generating test plugin in: {}", output_dir.display());
    }

    let output_dir_arg = output_dir.display().to_string();

    let args = create_test_plugin_args(&output_dir_arg, local_sdk);
    let status = Command::new(cli_binary)
        .args(args.iter().map(String::as_str))
        .current_dir(workspace_root)
        .status()
        .context("Failed to run wavecraft create")?;

    if !status.success() {
        bail!("Failed to generate test plugin");
    }

    Ok(())
}

fn create_test_plugin_args(output_dir: &str, local_sdk: bool) -> Vec<String> {
    let mut args = vec![
        "create".to_string(),
        TEST_PLUGIN_INPUT_NAME.to_string(),
        "--vendor".to_string(),
        "Test Vendor".to_string(),
        "--email".to_string(),
        "test@example.com".to_string(),
        "--url".to_string(),
        "https://example.com".to_string(),
        "--no-git".to_string(),
        "--output".to_string(),
        output_dir.to_string(),
    ];

    if local_sdk {
        args.push("--local-sdk".to_string());
    }

    args
}

fn copy_cli_binary_for_git_source(
    cli_binary: &Path,
    installed_like_cli_binary: &Path,
) -> Result<()> {
    if let Some(parent) = installed_like_cli_binary.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::copy(cli_binary, installed_like_cli_binary).with_context(|| {
        format!(
            "Failed to copy CLI binary from {} to {}",
            cli_binary.display(),
            installed_like_cli_binary.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(installed_like_cli_binary)
            .with_context(|| {
                format!(
                    "Failed to read metadata for {}",
                    installed_like_cli_binary.display()
                )
            })?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(installed_like_cli_binary, perms).with_context(|| {
            format!(
                "Failed to set executable permission on {}",
                installed_like_cli_binary.display()
            )
        })?;
    }

    Ok(())
}

fn verify_generated_engine_identifiers(project_dir: &Path) -> Result<()> {
    let engine_cargo = project_dir.join("engine/Cargo.toml");

    let cargo_contents = fs::read_to_string(&engine_cargo)
        .with_context(|| format!("Failed to read {}", engine_cargo.display()))?;

    let package_name_line = format!("name = \"{}\"", TEST_PLUGIN_INTERNAL_ID);
    if !cargo_contents.contains(&package_name_line) {
        bail!(
            "Generated engine/Cargo.toml package name mismatch: expected '{}'",
            TEST_PLUGIN_INTERNAL_ID
        );
    }

    let lib_name_line = format!("name = \"{}\"", TEST_PLUGIN_INTERNAL_ID);
    if !cargo_contents.contains("[lib]") || !cargo_contents.contains(&lib_name_line) {
        bail!(
            "Generated engine/Cargo.toml lib name mismatch: expected '{}'",
            TEST_PLUGIN_INTERNAL_ID
        );
    }

    Ok(())
}

/// Verify that all expected files were generated.
fn verify_generated_files(project_dir: &Path) -> Result<()> {
    let expected_files = [
        "engine/Cargo.toml",
        "engine/src/lib.rs",
        "ui/package.json",
        "ui/src/App.tsx",
        "README.md",
    ];

    for file in &expected_files {
        let path = project_dir.join(file);
        if !path.exists() {
            bail!("Missing expected file: {}", file);
        }
    }

    let removed_xtask_dir = project_dir.join("engine/xtask");
    if removed_xtask_dir.exists() {
        bail!(
            "Generated project should not include deprecated template xtask directory: {}",
            removed_xtask_dir.display()
        );
    }

    Ok(())
}

/// Normalize generated UI dependencies to local workspace package paths.
///
/// In SDK development mode, `wavecraft create` may emit local file dependencies
/// for Rust crates only while keeping npm package versions. For local template
/// validation we validate against current workspace UI packages to avoid stale
/// published npm package drift.
fn normalize_generated_ui_dependencies(project_dir: &Path, workspace_root: &Path) -> Result<()> {
    let generated_ui_package_json = project_dir.join("ui/package.json");
    let core_package_dir = workspace_root.join("ui/packages/core");
    let components_package_dir = workspace_root.join("ui/packages/components");

    let generated_content = fs::read_to_string(&generated_ui_package_json)
        .context("Failed to read generated ui/package.json")?;
    let mut generated: serde_json::Value = serde_json::from_str(&generated_content)
        .context("Failed to parse generated ui/package.json")?;

    let Some(deps) = generated
        .get_mut("dependencies")
        .and_then(serde_json::Value::as_object_mut)
    else {
        bail!("Generated ui/package.json missing dependencies object");
    };

    deps.insert(
        "@wavecraft/core".to_string(),
        serde_json::Value::String(format!("file:{}", core_package_dir.display())),
    );
    deps.insert(
        "@wavecraft/components".to_string(),
        serde_json::Value::String(format!("file:{}", components_package_dir.display())),
    );

    let serialized = serde_json::to_string_pretty(&generated)
        .context("Failed to serialize generated package")?;
    fs::write(&generated_ui_package_json, format!("{}\n", serialized))
        .context("Failed to write normalized generated ui/package.json")?;

    Ok(())
}

/// Apply local patch overrides for git-source simulation validation.
///
/// Generated projects in git-source mode reference published git tags, which may
/// not include unreleased changes under test. For local/CI template validation we
/// keep git-source declarations in place but add `[patch]` overrides to current
/// workspace crates so startup-path checks are deterministic and release-independent.
fn apply_git_source_validation_overrides(project_dir: &Path, workspace_root: &Path) -> Result<()> {
    let workspace_cargo = project_dir.join("Cargo.toml");

    let mut cargo_contents = fs::read_to_string(&workspace_cargo)
        .with_context(|| format!("Failed to read {}", workspace_cargo.display()))?;

    let patch_header = "[patch.\"https://github.com/RonHouben/wavecraft\"]";
    if cargo_contents.contains(patch_header) {
        return Ok(());
    }

    let nih_plug_path = workspace_root.join("engine/crates/wavecraft-nih_plug");
    let dsp_path = workspace_root.join("engine/crates/wavecraft-dsp");
    let dev_server_path = workspace_root.join("dev-server");

    let patch_block = format!(
        "\n\n{patch_header}\nwavecraft-nih_plug = {{ path = \"{}\" }}\nwavecraft-dsp = {{ path = \"{}\" }}\nwavecraft-dev-server = {{ path = \"{}\" }}\n",
        nih_plug_path.display(),
        dsp_path.display(),
        dev_server_path.display(),
    );

    cargo_contents.push_str(&patch_block);
    fs::write(&workspace_cargo, cargo_contents)
        .with_context(|| format!("Failed to update {}", workspace_cargo.display()))?;

    Ok(())
}

/// Validate engine code compilation and linting.
fn validate_engine(project_dir: &Path, verbose: bool) -> Result<()> {
    let engine_dir = project_dir.join("engine");

    // Check compilation
    if verbose {
        println!("Running cargo check...");
    }
    run_command(
        "cargo",
        &["check", "--manifest-path", "engine/Cargo.toml"],
        project_dir,
    )?;

    // Clippy
    if verbose {
        println!("Running clippy...");
    }
    run_command(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &engine_dir,
    )?;

    // Formatting
    if verbose {
        println!("Checking formatting...");
    }
    run_command("cargo", &["fmt", "--check"], &engine_dir)?;

    Ok(())
}

/// Validate UI code compilation and linting.
fn validate_ui(project_dir: &Path, verbose: bool) -> Result<()> {
    let ui_dir = project_dir.join("ui");

    // Install dependencies
    if verbose {
        println!("Installing UI dependencies...");
    }
    run_command("npm", &["install"], &ui_dir)?;

    // Lint
    if verbose {
        println!("Running ESLint...");
    }
    run_command("npm", &["run", "lint"], &ui_dir)?;

    // Format check
    if verbose {
        println!("Checking Prettier formatting...");
    }
    run_command("npm", &["run", "format:check"], &ui_dir)?;

    // Type-check
    if verbose {
        println!("Running TypeScript type-check...");
    }
    run_command("npm", &["run", "typecheck"], &ui_dir)?;

    // Build
    if verbose {
        println!("Building UI...");
    }
    run_command("npm", &["run", "build"], &ui_dir)?;

    Ok(())
}

fn install_ui_dependencies(project_dir: &Path, verbose: bool) -> Result<()> {
    let ui_dir = project_dir.join("ui");

    if verbose {
        println!("Installing UI dependencies for startup smoke...");
    }

    run_command("npm", &["install"], &ui_dir)
}

fn validate_git_source_engine_modes(project_dir: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Running discovery-mode compile check...");
    }
    run_command(
        "cargo",
        &[
            "build",
            "--manifest-path",
            "engine/Cargo.toml",
            "--lib",
            "--features",
            "_param-discovery",
        ],
        project_dir,
    )?;

    if verbose {
        println!("Running default-mode compile check...");
    }
    run_command(
        "cargo",
        &["build", "--manifest-path", "engine/Cargo.toml", "--lib"],
        project_dir,
    )
}

fn output_contains_asset_regression_signature(output: &str) -> bool {
    output.contains("include_dir!(\"$CARGO_MANIFEST_DIR/assets/ui-dist\")")
        || output.contains("assets/ui-dist") && output.contains("No such file or directory")
}

fn output_contains_ready_marker(output: &str) -> bool {
    output.contains("All servers running")
}

fn cleanup_start_smoke_ports(ws_port: u16, ui_port: u16, verbose: bool) {
    for port in [ws_port, ui_port] {
        let lsof_output = Command::new("lsof")
            .args(["-t", &format!("-iTCP:{}", port), "-sTCP:LISTEN"])
            .output();

        let Ok(output) = lsof_output else {
            continue;
        };

        if !output.status.success() {
            continue;
        }

        let pids = String::from_utf8_lossy(&output.stdout);
        for pid in pids.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if verbose {
                println!(
                    "Cleaning up lingering process on port {} (pid {})",
                    port, pid
                );
            }

            let _ = Command::new("kill").args(["-9", pid]).status();
        }
    }
}

fn validate_git_source_start_smoke(
    cli_binary: &Path,
    project_dir: &Path,
    ws_port: u16,
    ui_port: u16,
    verbose: bool,
) -> Result<()> {
    ensure_generated_plugin_root(project_dir)?;

    // Ensure deterministic startup for fixed smoke-test ports across repeated runs.
    cleanup_start_smoke_ports(ws_port, ui_port, verbose);

    if verbose {
        println!(
            "Running bounded startup smoke: wavecraft start --no-install --port {} --ui-port {}",
            ws_port, ui_port
        );
    }

    let mut child = spawn_start_smoke_child(cli_binary, project_dir, ws_port, ui_port)?;
    let mut output_capture = ChildOutputCapture::new(&mut child)?;

    let start_time = Instant::now();

    loop {
        output_capture
            .drain_available()
            .context("Failed to read startup smoke output")?;

        if let Some(status) = child
            .try_wait()
            .context("Failed to poll startup smoke process")?
        {
            output_capture.drain_after_exit()?;
            let stdout = output_capture.stdout_lossy();
            let stderr = output_capture.stderr_lossy();
            let combined = format!("{}\n{}", stdout, stderr);

            cleanup_start_smoke_ports(ws_port, ui_port, verbose);

            if !status.success() {
                bail!(
                    "`wavecraft start` exited early with status {}\nstdout:\n{}\nstderr:\n{}",
                    status,
                    stdout,
                    stderr
                );
            }

            if output_contains_asset_regression_signature(combined.as_str()) {
                bail!(
                    "Detected fallback asset regression signature in startup smoke output:\n{}",
                    combined
                );
            }

            if !output_contains_ready_marker(combined.as_str()) {
                bail!(
                    "`wavecraft start` exited without reaching ready state marker\nstdout:\n{}\nstderr:\n{}",
                    stdout,
                    stderr
                );
            }

            return Ok(());
        }

        let combined = format!(
            "{}\n{}",
            output_capture.stdout_lossy(),
            output_capture.stderr_lossy()
        );

        if output_contains_ready_marker(combined.as_str()) {
            if verbose {
                println!("Startup smoke reached ready marker; terminating bounded run");
            }

            terminate_start_smoke_child(&mut child, verbose)?;
            output_capture.drain_after_exit()?;

            let stdout = output_capture.stdout_lossy();
            let stderr = output_capture.stderr_lossy();
            let combined = format!("{}\n{}", stdout, stderr);

            cleanup_start_smoke_ports(ws_port, ui_port, verbose);

            if output_contains_asset_regression_signature(combined.as_str()) {
                bail!(
                    "Detected fallback asset regression signature in startup smoke output:\n{}",
                    combined
                );
            }

            return Ok(());
        }

        if start_time.elapsed() >= Duration::from_secs(START_SMOKE_TIMEOUT_SECS) {
            terminate_start_smoke_child(&mut child, verbose)?;
            output_capture.drain_after_exit()?;
            let stdout = output_capture.stdout_lossy();
            let stderr = output_capture.stderr_lossy();
            let combined = format!("{}\n{}", stdout, stderr);

            cleanup_start_smoke_ports(ws_port, ui_port, verbose);

            if output_contains_asset_regression_signature(combined.as_str()) {
                bail!(
                    "Detected fallback asset regression signature in startup smoke output:\n{}",
                    combined
                );
            }

            if !output_contains_ready_marker(combined.as_str()) {
                bail!(
                    "Startup smoke timed out before ready marker ({START_SMOKE_TIMEOUT_SECS}s)\nstdout:\n{}\nstderr:\n{}",
                    stdout,
                    stderr
                );
            }

            return Ok(());
        }

        thread::sleep(Duration::from_millis(200));
    }
}

fn ensure_generated_plugin_root(project_dir: &Path) -> Result<()> {
    let engine_cargo = project_dir.join("engine/Cargo.toml");
    let sdk_template_dir = project_dir.join("sdk-template");

    if !engine_cargo.is_file() || sdk_template_dir.is_dir() {
        bail!(
            "Expected generated plugin root (engine/Cargo.toml present, sdk-template absent), got: {}",
            project_dir.display()
        );
    }

    Ok(())
}

fn spawn_start_smoke_child(
    cli_binary: &Path,
    project_dir: &Path,
    ws_port: u16,
    ui_port: u16,
) -> Result<Child> {
    let mut command = Command::new(cli_binary);
    command
        .args([
            "start",
            "--no-install",
            "--port",
            &ws_port.to_string(),
            "--ui-port",
            &ui_port.to_string(),
        ])
        .current_dir(project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        // SAFETY: `pre_exec` runs in the child process after fork and before exec.
        // We only call async-signal-safe `setpgid(0, 0)` to put the startup smoke
        // process in its own process group so timeout cleanup can terminate all
        // descendants that may inherit stdio handles.
        unsafe {
            command.pre_exec(|| {
                setpgid(Pid::from_raw(0), Pid::from_raw(0))
                    .map_err(|err| io::Error::other(format!("setpgid failed: {err}")))?;
                Ok(())
            });
        }
    }

    command
        .spawn()
        .context("Failed to spawn `wavecraft start` for startup smoke")
}

fn terminate_start_smoke_child(child: &mut Child, verbose: bool) -> Result<()> {
    #[cfg(unix)]
    {
        let pid_raw = child.id() as i32;
        if verbose {
            println!(
                "Startup smoke timeout reached, terminating process group {}",
                pid_raw
            );
        }

        let process_group = Pid::from_raw(pid_raw);

        // Best effort graceful stop first.
        let _ = killpg(process_group, Signal::SIGTERM);
        if wait_for_child_exit(child, Duration::from_millis(START_SMOKE_KILL_GRACE_MS))? {
            return Ok(());
        }

        if verbose {
            println!(
                "Process group {} still running after SIGTERM, sending SIGKILL",
                pid_raw
            );
        }

        let _ = killpg(process_group, Signal::SIGKILL);
        if wait_for_child_exit(child, Duration::from_millis(START_SMOKE_KILL_GRACE_MS))? {
            return Ok(());
        }

        bail!(
            "Failed to terminate startup smoke process group {} within bounded timeout",
            pid_raw
        );
    }

    #[cfg(not(unix))]
    {
        if verbose {
            println!("Startup smoke timeout reached, killing process");
        }

        let _ = child.kill();
        if wait_for_child_exit(child, Duration::from_millis(START_SMOKE_KILL_GRACE_MS))? {
            return Ok(());
        }

        bail!("Failed to terminate startup smoke process within bounded timeout");
    }
}

fn wait_for_child_exit(child: &mut Child, timeout: Duration) -> Result<bool> {
    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .context("Failed while waiting for startup smoke process to exit")?
            .is_some()
        {
            return Ok(true);
        }

        if started.elapsed() >= timeout {
            return Ok(false);
        }

        thread::sleep(Duration::from_millis(50));
    }
}

struct ChildOutputCapture {
    stdout: ChildStdout,
    stderr: ChildStderr,
    stdout_buf: Vec<u8>,
    stderr_buf: Vec<u8>,
}

impl ChildOutputCapture {
    fn new(child: &mut Child) -> Result<Self> {
        let stdout = child
            .stdout
            .take()
            .context("Startup smoke stdout pipe was not available")?;
        let stderr = child
            .stderr
            .take()
            .context("Startup smoke stderr pipe was not available")?;

        #[cfg(unix)]
        {
            set_nonblocking(stdout.as_raw_fd())
                .context("Failed to set startup smoke stdout non-blocking")?;
            set_nonblocking(stderr.as_raw_fd())
                .context("Failed to set startup smoke stderr non-blocking")?;
        }

        Ok(Self {
            stdout,
            stderr,
            stdout_buf: Vec::new(),
            stderr_buf: Vec::new(),
        })
    }

    fn drain_available(&mut self) -> Result<usize> {
        let mut total_read = 0;
        total_read += drain_nonblocking(&mut self.stdout, &mut self.stdout_buf)
            .context("Failed draining startup smoke stdout")?;
        total_read += drain_nonblocking(&mut self.stderr, &mut self.stderr_buf)
            .context("Failed draining startup smoke stderr")?;
        Ok(total_read)
    }

    fn drain_after_exit(&mut self) -> Result<()> {
        for _ in 0..START_SMOKE_POST_EXIT_DRAIN_ATTEMPTS {
            let bytes = self.drain_available()?;
            if bytes == 0 {
                break;
            }
            thread::sleep(Duration::from_millis(START_SMOKE_POST_EXIT_DRAIN_SLEEP_MS));
        }

        Ok(())
    }

    fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout_buf).into_owned()
    }

    fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr_buf).into_owned()
    }
}

fn drain_nonblocking<R: Read>(reader: &mut R, buffer: &mut Vec<u8>) -> io::Result<usize> {
    let mut total_read = 0;
    let mut chunk = [0_u8; 8 * 1024];

    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                total_read += n;
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        }
    }

    Ok(total_read)
}

#[cfg(unix)]
fn set_nonblocking(fd: i32) -> Result<()> {
    // SAFETY: `fd` is an open file descriptor owned by the child pipe handles.
    // `F_GETFL` does not modify memory and returns the descriptor flags.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error()).context("Failed to read file descriptor flags");
    }

    // SAFETY: `fd` is valid and we only update file status flags by OR-ing
    // `O_NONBLOCK` onto existing flags.
    let status = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if status < 0 {
        return Err(io::Error::last_os_error()).context("Failed to set non-blocking flag");
    }

    Ok(())
}

/// Validate the CLI-owned bundle command contract.
fn validate_cli_bundle_contract(
    cli_binary: &Path,
    project_dir: &Path,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Validating CLI bundle command contract...");
    }

    let help_output = run_command_capture(
        cli_binary
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid CLI binary path"))?,
        &["bundle", "--help"],
        project_dir,
    )?;
    assert_output_contains(
        &help_output,
        "--install",
        "Expected `wavecraft bundle --help` to include --install flag",
    )?;

    run_command(
        cli_binary
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid CLI binary path"))?,
        &["bundle"],
        project_dir,
    )?;

    if cfg!(target_os = "macos") {
        run_command(
            cli_binary
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid CLI binary path"))?,
            &["bundle", "--install"],
            project_dir,
        )?;
    } else if verbose {
        println!("Skipping `wavecraft bundle --install` validation on non-macOS host");
    }

    Ok(())
}

/// Run a command and check for success.
fn run_command(cmd: &str, args: &[&str], cwd: &Path) -> Result<()> {
    let mut command = if cmd == "npm" {
        xtask::npm_command()
    } else {
        Command::new(cmd)
    };

    let status = command
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!("Command failed: {} {}", cmd, args.join(" "));
    }

    Ok(())
}

/// Run a command, capture stdout/stderr, and require success.
fn run_command_capture(cmd: &str, args: &[&str], cwd: &Path) -> Result<Output> {
    let mut command = if cmd == "npm" {
        xtask::npm_command()
    } else {
        Command::new(cmd)
    };

    let output = command
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Command failed: {} {}\nstdout:\n{}\nstderr:\n{}",
            cmd,
            args.join(" "),
            stdout,
            stderr
        );
    }

    Ok(output)
}

fn assert_output_contains(output: &Output, needle: &str, message: &str) -> Result<()> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    if !combined.contains(needle) {
        bail!(
            "{}\nMissing substring: '{}'\nCommand output:\n{}",
            message,
            needle,
            combined
        );
    }

    Ok(())
}

/// Print a phase header.
fn print_phase(name: &str) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status(name);
}

/// Cleanup guard to ensure test project is removed even on error.
struct CleanupGuard {
    path: PathBuf,
    keep: bool,
    verbose: bool,
}

impl CleanupGuard {
    fn new(path: PathBuf, keep: bool, verbose: bool) -> Self {
        Self {
            path,
            keep,
            verbose,
        }
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if !self.keep && self.path.exists() {
            if self.verbose {
                println!();
                println!("Cleaning up test project...");
            }
            if let Err(e) = fs::remove_dir_all(&self.path) {
                eprintln!("Warning: Failed to clean up test project: {}", e);
            } else if self.verbose {
                println!("Test project removed: {}", self.path.display());
            }
        } else if self.keep && self.path.exists() {
            println!();
            println!("Test project kept at: {}", self.path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = ValidateTemplateConfig::default();
        assert!(!config.verbose);
        assert!(!config.keep);
    }

    #[test]
    fn test_create_test_plugin_args_include_output_mode() {
        let args = create_test_plugin_args("/tmp/test-plugin", false);

        assert!(args.iter().any(|arg| arg == "--output"));

        let output_idx = args
            .iter()
            .position(|arg| arg == "--output")
            .expect("--output flag missing");
        assert_eq!(
            args.get(output_idx + 1),
            Some(&"/tmp/test-plugin".to_string())
        );
    }

    #[test]
    fn test_create_test_plugin_args_local_sdk_mode() {
        let args = create_test_plugin_args("/tmp/test-plugin", true);
        assert!(args.iter().any(|arg| arg == "--local-sdk"));
    }

    #[test]
    fn test_create_test_plugin_args_git_source_mode() {
        let args = create_test_plugin_args("/tmp/test-plugin", false);
        assert!(!args.iter().any(|arg| arg == "--local-sdk"));
    }

    #[test]
    fn test_output_contains_asset_regression_signature_detects_include_dir_path() {
        let output = r#"panic at include_dir!("$CARGO_MANIFEST_DIR/assets/ui-dist")"#;
        assert!(output_contains_asset_regression_signature(output));
    }

    #[test]
    fn test_output_contains_asset_regression_signature_detects_missing_assets_dir() {
        let output = "assets/ui-dist: No such file or directory";
        assert!(output_contains_asset_regression_signature(output));
    }

    #[test]
    fn test_output_contains_asset_regression_signature_ignores_unrelated_output() {
        let output = "All servers running";
        assert!(!output_contains_asset_regression_signature(output));
    }

    #[test]
    fn test_output_contains_ready_marker_detects_expected_text() {
        let output = "...\nAll servers running\n...";
        assert!(output_contains_ready_marker(output));
    }

    #[test]
    fn test_output_contains_ready_marker_ignores_unrelated_output() {
        let output = "Server starting";
        assert!(!output_contains_ready_marker(output));
    }

    #[test]
    fn test_ensure_generated_plugin_root_accepts_generated_project_shape() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("engine")).expect("create engine dir");
        fs::write(root.join("engine/Cargo.toml"), "[package]\nname = \"x\"\n")
            .expect("write engine cargo");

        ensure_generated_plugin_root(root).expect("expected generated project root to pass");
    }

    #[test]
    fn test_ensure_generated_plugin_root_rejects_sdk_repo_shape() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("engine")).expect("create engine dir");
        fs::write(root.join("engine/Cargo.toml"), "[package]\nname = \"x\"\n")
            .expect("write engine cargo");
        fs::create_dir_all(root.join("sdk-template")).expect("create sdk-template dir");

        let err = ensure_generated_plugin_root(root).expect_err("expected repo shape to fail");
        assert!(
            err.to_string().contains("Expected generated plugin root"),
            "unexpected error: {err}"
        );
    }
}
