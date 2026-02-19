use anyhow::{Context, Result};
use console::style;
use regex::Regex;

use crate::project::ProjectMarkers;

const SDK_TSCONFIG_PATHS_MARKER: &str =
    r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#;
const SDK_TSCONFIG_PATHS_SNIPPET: &str = r#"    /* SDK development — resolve @wavecraft packages from monorepo source */
    "baseUrl": ".",
    "paths": {
      "@wavecraft/core": ["../../ui/packages/core/src/index.ts"],
      "@wavecraft/core/*": ["../../ui/packages/core/src/*"],
      "@wavecraft/components": ["../../ui/packages/components/src/index.ts"],
      "@wavecraft/components/*": ["../../ui/packages/components/src/*"]
    }"#;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TsconfigPathsInjection {
    Updated(String),
    Unchanged,
    Warning(&'static str),
}

fn find_object_bounds_after_key(content: &str, key: &str) -> Option<(usize, usize)> {
    let key_start = content.find(key)?;
    let bytes = content.as_bytes();
    let mut index = key_start + key.len();

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    if index >= bytes.len() || bytes[index] != b':' {
        return None;
    }
    index += 1;

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    while index < bytes.len() && bytes[index] != b'{' {
        index += 1;
    }

    if index >= bytes.len() || bytes[index] != b'{' {
        return None;
    }

    let open_index = index;
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut is_escaped = false;
    let mut cursor = open_index;

    while cursor < bytes.len() {
        let ch = bytes[cursor];

        if in_string {
            if is_escaped {
                is_escaped = false;
            } else if ch == b'\\' {
                is_escaped = true;
            } else if ch == b'"' {
                in_string = false;
            }
            cursor += 1;
            continue;
        }

        if ch == b'"' {
            in_string = true;
            cursor += 1;
            continue;
        }

        if ch == b'/' && cursor + 1 < bytes.len() {
            let next = bytes[cursor + 1];
            if next == b'/' {
                cursor += 2;
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
                continue;
            }

            if next == b'*' {
                cursor += 2;
                while cursor + 1 < bytes.len() {
                    if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
                        cursor += 2;
                        break;
                    }
                    cursor += 1;
                }
                continue;
            }
        }

        if ch == b'{' {
            depth += 1;
        } else if ch == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((open_index, cursor));
            }
        }

        cursor += 1;
    }

    None
}

pub(crate) fn apply_sdk_tsconfig_paths(content: &str) -> Result<TsconfigPathsInjection> {
    if !content.contains("\"compilerOptions\"") {
        return Ok(TsconfigPathsInjection::Warning(
            "could not inject SDK TypeScript paths: `compilerOptions` block not found",
        ));
    }

    if content.contains(SDK_TSCONFIG_PATHS_MARKER) {
        return Ok(TsconfigPathsInjection::Unchanged);
    }

    let (compiler_options_start, compiler_options_end) =
        match find_object_bounds_after_key(content, "\"compilerOptions\"") {
            Some(bounds) => bounds,
            None => return Ok(TsconfigPathsInjection::Warning(
                "could not inject SDK TypeScript paths: failed to locate `compilerOptions` object",
            )),
        };

    let compiler_options_content = &content[compiler_options_start + 1..compiler_options_end];
    if compiler_options_content.contains("\"paths\"") {
        return Ok(TsconfigPathsInjection::Warning(
            "could not auto-inject SDK TypeScript paths: `compilerOptions.paths` already exists, please add @wavecraft mappings manually",
        ));
    }

    let anchor_re = Regex::new(
        r#"\"(noFallthroughCasesInSwitch|noUnusedParameters|noUnusedLocals|strict|jsx|noEmit|moduleResolution|target)\"\s*:\s*[^\n]*"#,
    )
    .context("Invalid regex for tsconfig anchor detection")?;

    if let Some(anchor) = anchor_re.find(compiler_options_content) {
        let anchor_start = compiler_options_start + 1 + anchor.start();
        let anchor_end = compiler_options_start + 1 + anchor.end();
        let anchor_text = &content[anchor_start..anchor_end];
        let needs_comma = !anchor_text.trim_end().ends_with(',');
        let comma = if needs_comma { "," } else { "" };
        let has_following_properties =
            has_jsonc_property_after_anchor(&content[anchor_end..compiler_options_end]);

        let mut updated = String::with_capacity(content.len() + 256);
        updated.push_str(&content[..anchor_end]);
        updated.push_str(comma);
        updated.push_str("\n\n");
        updated.push_str(SDK_TSCONFIG_PATHS_SNIPPET);
        if has_following_properties {
            updated.push(',');
        }
        updated.push_str(&content[anchor_end..]);

        return Ok(TsconfigPathsInjection::Updated(updated));
    }

    let trimmed = compiler_options_content.trim_end();
    let has_properties = trimmed.contains('"') && trimmed.contains(':');
    let needs_comma = has_properties && !trimmed.ends_with(',');
    let comma = if needs_comma { "," } else { "" };

    let mut updated = String::with_capacity(content.len() + 256);
    updated.push_str(&content[..compiler_options_end]);
    updated.push_str(comma);
    updated.push_str("\n\n");
    updated.push_str(SDK_TSCONFIG_PATHS_SNIPPET);
    updated.push('\n');
    updated.push_str(&content[compiler_options_end..]);

    Ok(TsconfigPathsInjection::Updated(updated))
}

fn has_jsonc_property_after_anchor(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && (bytes[index].is_ascii_whitespace() || bytes[index] == b',') {
            index += 1;
        }

        if index >= bytes.len() {
            return false;
        }

        if bytes[index] == b'/' && index + 1 < bytes.len() {
            if bytes[index + 1] == b'/' {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                continue;
            }

            if bytes[index + 1] == b'*' {
                index += 2;
                while index + 1 < bytes.len() {
                    if bytes[index] == b'*' && bytes[index + 1] == b'/' {
                        index += 2;
                        break;
                    }
                    index += 1;
                }
                continue;
            }
        }

        return bytes[index] == b'"';
    }

    false
}

pub(crate) fn ensure_sdk_ui_paths_for_typescript(project: &ProjectMarkers) -> Result<()> {
    if !project.sdk_mode {
        return Ok(());
    }

    let tsconfig_path = project.ui_dir.join("tsconfig.json");
    if !tsconfig_path.is_file() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&tsconfig_path)
        .with_context(|| format!("Failed to read {}", tsconfig_path.display()))?;

    match apply_sdk_tsconfig_paths(&content)? {
        TsconfigPathsInjection::Updated(updated) => {
            std::fs::write(&tsconfig_path, updated)
                .with_context(|| format!("Failed to write {}", tsconfig_path.display()))?;

            println!(
                "{} Enabled SDK TypeScript path mappings in {}",
                style("✓").green(),
                tsconfig_path.display()
            );
        }
        TsconfigPathsInjection::Unchanged => {}
        TsconfigPathsInjection::Warning(message) => {
            println!("{} {}", style("⚠").yellow(), message);
            println!(
                "  Add @wavecraft path mappings manually in {} if needed.",
                tsconfig_path.display()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{apply_sdk_tsconfig_paths, TsconfigPathsInjection};

    #[test]
    fn injects_sdk_paths_when_missing() {
        let input = r#"{
    "compilerOptions": {
        "strict": true,
        "noFallthroughCasesInSwitch": true
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        let TsconfigPathsInjection::Updated(output) = output else {
            panic!("should inject");
        };

        assert!(output.contains(r#""baseUrl": ".""#));
        assert!(output.contains(r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#));
        assert!(output
            .contains(r#""@wavecraft/components": ["../../ui/packages/components/src/index.ts"]"#));
    }

    #[test]
    fn is_idempotent_when_paths_present() {
        let input = r#"{
    "compilerOptions": {
        "noFallthroughCasesInSwitch": true,
        "baseUrl": ".",
        "paths": {
            "@wavecraft/core": ["../../ui/packages/core/src/index.ts"]
        }
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");
        assert_eq!(output, TsconfigPathsInjection::Unchanged);
    }

    #[test]
    fn injects_when_primary_anchor_is_missing() {
        let input = r#"{
    "compilerOptions": {
        "strict": true
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        let TsconfigPathsInjection::Updated(output) = output else {
            panic!("should inject using fallback anchor");
        };

        assert!(output.contains(r#""baseUrl": ".""#));
        assert!(output.contains(r#""paths": {"#));
    }

    #[test]
    fn returns_warning_when_compiler_options_missing() {
        let input = r#"{
    "include": ["src"]
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        assert_eq!(
            output,
            TsconfigPathsInjection::Warning(
                "could not inject SDK TypeScript paths: `compilerOptions` block not found"
            )
        );
    }

    #[test]
    fn injects_paths_with_trailing_comma_before_following_property() {
        let input = r#"{
    "compilerOptions": {
        "moduleResolution": "bundler",
        "allowSyntheticDefaultImports": true,
        "types": ["node"]
    }
}"#;

        let output = apply_sdk_tsconfig_paths(input).expect("should parse");

        let TsconfigPathsInjection::Updated(output) = output else {
            panic!("should inject");
        };

        assert!(
            output.contains("\"@wavecraft/components/*\": [\"../../ui/packages/components/src/*\"]\n    },\n        \"allowSyntheticDefaultImports\""),
            "Expected trailing comma after injected paths block before following property:\n{}",
            output
        );
        assert!(
            !output.contains("\"@wavecraft/components/*\": [\"../../ui/packages/components/src/*\"]\n    }\n        \"allowSyntheticDefaultImports\""),
            "Injected paths block must not be adjacent to next property without comma:\n{}",
            output
        );
    }
}
