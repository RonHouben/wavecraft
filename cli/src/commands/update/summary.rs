use super::{project_update, self_update};
use anyhow::{bail, Result};

/// Outcome of the summary decision logic, extracted for testability.
#[cfg_attr(test, derive(Debug, PartialEq))]
enum SummaryOutcome {
    /// Both phases completed successfully.
    AllComplete,
    /// CLI failed but project deps succeeded.
    ProjectOnlyComplete,
    /// Project dependency updates failed.
    ProjectErrors { errors: Vec<String> },
    /// CLI failed and not in a project — messages already shown inline.
    NoAction,
}

/// Determine the summary outcome from both update phases.
///
/// This is a pure function extracted from `print_summary()` for testability.
/// It decides what messages should be shown and whether the process should fail.
fn determine_summary(
    self_update: &self_update::SelfUpdateResult,
    project: &project_update::ProjectUpdateResult,
) -> SummaryOutcome {
    let cli_failed = matches!(self_update, self_update::SelfUpdateResult::Failed);
    let in_project = matches!(project, project_update::ProjectUpdateResult::Updated { .. });

    let project_errors: &[String] = match project {
        project_update::ProjectUpdateResult::Updated { errors } => errors,
        project_update::ProjectUpdateResult::NotInProject => &[],
    };

    if !project_errors.is_empty() {
        return SummaryOutcome::ProjectErrors {
            errors: project_errors.to_vec(),
        };
    }

    if cli_failed && in_project {
        return SummaryOutcome::ProjectOnlyComplete;
    }

    if cli_failed && !in_project {
        return SummaryOutcome::NoAction;
    }

    SummaryOutcome::AllComplete
}

/// Print a summary of both update phases and determine the exit code.
pub(super) fn print_summary(
    self_update: &self_update::SelfUpdateResult,
    project: &project_update::ProjectUpdateResult,
) -> Result<()> {
    let outcome = determine_summary(self_update, project);

    match outcome {
        SummaryOutcome::AllComplete => {
            println!();
            println!("✨ All updates complete");
        }
        SummaryOutcome::ProjectOnlyComplete => {
            println!();
            println!("✨ Project dependencies updated (CLI self-update skipped)");
        }
        SummaryOutcome::ProjectErrors { errors } => {
            bail!(
                "Failed to update some dependencies:\n  {}",
                errors.join("\n  ")
            );
        }
        SummaryOutcome::NoAction => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- determine_summary tests (QA-L-003) ---

    #[test]
    fn test_summary_all_complete_no_project() {
        let outcome = determine_summary(
            &self_update::SelfUpdateResult::AlreadyUpToDate,
            &project_update::ProjectUpdateResult::NotInProject,
        );
        assert_eq!(outcome, SummaryOutcome::AllComplete);
    }

    #[test]
    fn test_summary_all_complete_with_project() {
        let outcome = determine_summary(
            &self_update::SelfUpdateResult::AlreadyUpToDate,
            &project_update::ProjectUpdateResult::Updated { errors: vec![] },
        );
        assert_eq!(outcome, SummaryOutcome::AllComplete);
    }

    #[test]
    fn test_summary_updated_with_project() {
        let outcome = determine_summary(
            &self_update::SelfUpdateResult::Updated,
            &project_update::ProjectUpdateResult::Updated { errors: vec![] },
        );
        assert_eq!(outcome, SummaryOutcome::AllComplete);
    }

    #[test]
    fn test_summary_cli_failed_in_project() {
        let outcome = determine_summary(
            &self_update::SelfUpdateResult::Failed,
            &project_update::ProjectUpdateResult::Updated { errors: vec![] },
        );
        assert_eq!(outcome, SummaryOutcome::ProjectOnlyComplete);
    }

    #[test]
    fn test_summary_cli_failed_not_in_project() {
        let outcome = determine_summary(
            &self_update::SelfUpdateResult::Failed,
            &project_update::ProjectUpdateResult::NotInProject,
        );
        assert_eq!(outcome, SummaryOutcome::NoAction);
    }

    #[test]
    fn test_summary_project_errors() {
        let outcome = determine_summary(
            &self_update::SelfUpdateResult::AlreadyUpToDate,
            &project_update::ProjectUpdateResult::Updated {
                errors: vec!["Rust: compile failed".to_string()],
            },
        );
        assert_eq!(
            outcome,
            SummaryOutcome::ProjectErrors {
                errors: vec!["Rust: compile failed".to_string()],
            }
        );
    }

    #[test]
    fn test_summary_updated_with_project_errors() {
        let outcome = determine_summary(
            &self_update::SelfUpdateResult::Updated,
            &project_update::ProjectUpdateResult::Updated {
                errors: vec!["npm: fetch failed".to_string()],
            },
        );
        assert_eq!(
            outcome,
            SummaryOutcome::ProjectErrors {
                errors: vec!["npm: fetch failed".to_string()],
            }
        );
    }
}
