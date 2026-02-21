use anyhow::Result;

mod project_update;
mod self_update;
mod summary;

/// Current CLI version, known at compile time.
pub(super) const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Update the CLI and (if in a project) project dependencies.
pub fn run(skip_self: bool) -> Result<()> {
    // Phase 1: CLI self-update (always runs unless re-exec'd)
    let self_update_result = if skip_self {
        println!("✅ CLI updated to {}", CURRENT_VERSION);
        self_update::SelfUpdateResult::AlreadyUpToDate
    } else {
        self_update::update_cli()
    };

    // If CLI was updated, re-exec the new binary for phase 2
    if matches!(self_update_result, self_update::SelfUpdateResult::Updated) {
        return self_update::reexec_with_new_binary();
    }

    // Phase 2: Project dependency update (context-dependent)
    let project_result = project_update::update_project_deps();

    // Summary and exit code
    summary::print_summary(&self_update_result, &project_result)
}
