use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;

pub(super) fn apply_dependency_rewrites(
    content: &str,
    sdk_path: &Path,
    sdk_crates: &[&str],
) -> Result<String> {
    let mut result = content.to_string();

    // Replace the main wavecraft dependency (with Cargo rename)
    // Match: wavecraft = { package = "wavecraft-nih_plug", git = "...", tag = "..." }
    let wavecraft_git_pattern = r#"wavecraft\s*=\s*\{\s*package\s*=\s*"wavecraft-nih_plug"\s*,\s*git\s*=\s*"https://github\.com/RonHouben/wavecraft"\s*,\s*tag\s*=\s*"[^"]*"\s*\}"#;
    let wavecraft_path_replacement = format!(
        r#"wavecraft = {{ package = "wavecraft-nih_plug", path = "{}/wavecraft-nih_plug" }}"#,
        sdk_path.display()
    );
    let wavecraft_re = Regex::new(wavecraft_git_pattern)
        .context("Invalid regex pattern for wavecraft-nih_plug")?;
    result = wavecraft_re
        .replace_all(&result, wavecraft_path_replacement.as_str())
        .to_string();

    // Replace individual SDK crate dependencies
    for crate_name in sdk_crates {
        // Match flexible git dependency patterns:
        // - Simple: crate = { git = "...", tag = "..." }
        // - With package: crate = { package = "crate", git = "...", tag = "..." }
        // - With optional: crate = { git = "...", tag = "...", optional = true }
        // - With features: crate = { git = "...", tag = "...", features = ["..."] }
        // - With both: crate = { package = "crate", git = "...", tag = "...", optional = true, features = [...] }
        let git_pattern = format!(
            r#"(?s)({}\s*=\s*\{{\s*)(?:package\s*=\s*"[^"]*"\s*,\s*)?git\s*=\s*"https://github\.com/RonHouben/wavecraft"\s*,\s*tag\s*=\s*"[^"]*"\s*((?:,\s*[^}}]*)?)\}}"#,
            regex::escape(crate_name)
        );

        let re = Regex::new(&git_pattern)
            .with_context(|| format!("Invalid regex pattern for crate: {}", crate_name))?;

        // Perform replacement preserving package and any extra attributes
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let prefix = &caps[1]; // "crate = { "
                let extra_attrs = &caps[2]; // ", optional = true, features = [...]" or empty

                // Check if package attribute exists in the original
                let package_attr = if caps[0].contains("package") {
                    format!(r#"package = "{}", "#, crate_name)
                } else {
                    String::new()
                };

                format!(
                    r#"{}{}path = "{}/{}"{} }}"#,
                    prefix,
                    package_attr,
                    sdk_path.display(),
                    crate_name,
                    extra_attrs
                )
            })
            .to_string();
    }

    // Handle wavecraft-dev-server separately — it lives at the repo root (dev-server/),
    // not under engine/crates/ like the other SDK crates.
    let sdk_root = sdk_path
        .parent()
        .and_then(|engine| engine.parent())
        .unwrap_or(sdk_path);
    let dev_server_git_pattern = r#"(?s)(wavecraft-dev-server\s*=\s*\{\s*)(?:package\s*=\s*"[^"]*"\s*,\s*)?git\s*=\s*"https://github\.com/RonHouben/wavecraft"\s*,\s*tag\s*=\s*"[^"]*"\s*((?:,\s*[^}]*)?)}"#;
    let dev_server_re = Regex::new(dev_server_git_pattern)
        .context("Invalid regex pattern for wavecraft-dev-server")?;
    result = dev_server_re
        .replace_all(&result, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let extra_attrs = &caps[2];
            let package_attr = if caps[0].contains("package") {
                r#"package = "wavecraft-dev-server", "#.to_string()
            } else {
                String::new()
            };
            format!(
                r#"{}{}path = "{}/dev-server"{} }}"#,
                prefix,
                package_attr,
                sdk_root.display(),
                extra_attrs
            )
        })
        .to_string();

    Ok(result)
}

pub(super) fn apply_npm_dependency_rewrites(content: &str, sdk_path: &Path) -> Result<String> {
    let sdk_root = sdk_path
        .parent()
        .and_then(|engine| engine.parent())
        .unwrap_or(sdk_path);

    // Replace npm package dependencies with local file paths in SDK mode.
    // This ensures generated UI projects validate against local monorepo packages
    // instead of potentially stale published npm versions.
    let core_file_dep = json_file_dep_value(&sdk_root.join("ui/packages/core"));
    let components_file_dep = json_file_dep_value(&sdk_root.join("ui/packages/components"));

    let core_dep_re = Regex::new(r#"\"@wavecraft/core\"\s*:\s*\"[^\"]+\""#)
        .context("Invalid regex pattern for @wavecraft/core npm dependency")?;
    let mut result = core_dep_re
        .replace_all(
            content,
            format!(r#""@wavecraft/core": "{}""#, core_file_dep),
        )
        .to_string();

    let components_dep_re = Regex::new(r#"\"@wavecraft/components\"\s*:\s*\"[^\"]+\""#)
        .context("Invalid regex pattern for @wavecraft/components npm dependency")?;
    result = components_dep_re
        .replace_all(
            &result,
            format!(r#""@wavecraft/components": "{}""#, components_file_dep),
        )
        .to_string();

    Ok(result)
}

fn json_file_dep_value(path: &Path) -> String {
    format!("file:{}", path.display()).replace('\\', "\\\\")
}
