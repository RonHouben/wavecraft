use anyhow::{Context, Result};
use regex::Regex;

const TSCONFIG_PATHS_MARKER: &str = r#""@wavecraft/core": ["../../ui/packages/core/src/index.ts"]"#;
const TSCONFIG_PATHS_SNIPPET: &str = r#"    /* SDK development — resolve @wavecraft packages from monorepo source */
        "baseUrl": ".",
        "paths": {
            "@wavecraft/core": ["../../ui/packages/core/src/index.ts"],
            "@wavecraft/core/*": ["../../ui/packages/core/src/*"],
            "@wavecraft/components": ["../../ui/packages/components/src/index.ts"],
            "@wavecraft/components/*": ["../../ui/packages/components/src/*"]
        }"#;

pub(super) fn inject_tsconfig_paths_if_needed(content: &str) -> Result<String> {
    // Only attempt tsconfig injection for tsconfig-like JSON content.
    if !content.contains("\"compilerOptions\"") {
        return Ok(content.to_string());
    }

    // Idempotent only when both paths marker and baseUrl already exist.
    if content.contains(TSCONFIG_PATHS_MARKER) && content.contains("\"baseUrl\"") {
        return Ok(content.to_string());
    }

    let Some((compiler_options_start, compiler_options_end)) =
        find_object_bounds_after_key(content, "\"compilerOptions\"")
    else {
        return Ok(content.to_string());
    };

    let compiler_options_content = &content[compiler_options_start + 1..compiler_options_end];
    if compiler_options_content.contains("\"paths\"") {
        if compiler_options_content.contains("\"baseUrl\"") {
            return Ok(content.to_string());
        }

        if let Some(paths_pos_rel) = compiler_options_content.find("\"paths\"") {
            let paths_pos = compiler_options_start + 1 + paths_pos_rel;
            let insertion_point = content[..paths_pos]
                .rfind('\n')
                .map_or(paths_pos, |idx| idx + 1);
            let indent = &content[insertion_point..paths_pos];

            let mut injected = String::with_capacity(content.len() + 32);
            injected.push_str(&content[..insertion_point]);
            injected.push_str(indent);
            injected.push_str("\"baseUrl\": \".\",\n\n");
            injected.push_str(&content[insertion_point..]);
            return Ok(injected);
        }

        return Ok(content.to_string());
    }

    let anchor_re = Regex::new(
        r#"\"(noFallthroughCasesInSwitch|noUnusedParameters|noUnusedLocals|strict|jsx|noEmit|moduleResolution|target)\"\s*:\s*[^\n]*"#,
    )
    .context("Invalid regex for tsconfig anchor detection")?;

    if let Some(anchor) = anchor_re.find(compiler_options_content) {
        let anchor_start = compiler_options_start + 1 + anchor.start();
        let anchor_end = compiler_options_start + 1 + anchor.end();
        let anchor_text = &content[anchor_start..anchor_end];
        let comma = comma_if_needed(anchor_text);
        let has_following_properties =
            has_jsonc_property_after_anchor(&content[anchor_end..compiler_options_end]);

        let mut injected = String::with_capacity(content.len() + 256);
        injected.push_str(&content[..anchor_end]);
        injected.push_str(comma);
        injected.push_str("\n\n");
        injected.push_str(TSCONFIG_PATHS_SNIPPET);
        if has_following_properties {
            injected.push(',');
        }
        injected.push_str(&content[anchor_end..]);
        return Ok(injected);
    }

    let trimmed = compiler_options_content.trim_end();
    let has_properties = trimmed.contains('"') && trimmed.contains(':');
    let comma = if has_properties {
        comma_if_needed(trimmed)
    } else {
        ""
    };

    let mut injected = String::with_capacity(content.len() + 256);
    injected.push_str(&content[..compiler_options_end]);
    injected.push_str(comma);
    injected.push_str("\n\n");
    injected.push_str(TSCONFIG_PATHS_SNIPPET);
    injected.push('\n');
    injected.push_str(&content[compiler_options_end..]);

    Ok(injected)
}

fn has_jsonc_property_after_anchor(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        index = skip_jsonc_whitespace_and_commas(bytes, index);

        if index >= bytes.len() {
            return false;
        }

        if skip_jsonc_comment(bytes, &mut index) {
            continue;
        }

        return bytes[index] == b'"';
    }

    false
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

        if ch == b'/' && cursor + 1 < bytes.len() && skip_jsonc_comment(bytes, &mut cursor) {
            continue;
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

fn comma_if_needed(segment: &str) -> &'static str {
    if segment.trim_end().ends_with(',') {
        ""
    } else {
        ","
    }
}

fn skip_jsonc_whitespace_and_commas(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && (bytes[index].is_ascii_whitespace() || bytes[index] == b',') {
        index += 1;
    }
    index
}

fn skip_jsonc_comment(bytes: &[u8], index: &mut usize) -> bool {
    if *index + 1 >= bytes.len() || bytes[*index] != b'/' {
        return false;
    }

    if bytes[*index + 1] == b'/' {
        *index += 2;
        while *index < bytes.len() && bytes[*index] != b'\n' {
            *index += 1;
        }
        return true;
    }

    if bytes[*index + 1] == b'*' {
        *index += 2;
        while *index + 1 < bytes.len() {
            if bytes[*index] == b'*' && bytes[*index + 1] == b'/' {
                *index += 2;
                break;
            }
            *index += 1;
        }
        return true;
    }

    false
}
