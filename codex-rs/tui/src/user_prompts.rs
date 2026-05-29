use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use codex_protocol::user_input::MAX_USER_INPUT_TEXT_CHARS;

const PROMPTS_DIR: &str = "prompts";
const PROMPT_EXT: &str = "md";
const FRONTMATTER_DELIMITER: &str = "---";
const MAX_PROMPT_FILE_BYTES: u64 = MAX_USER_INPUT_TEXT_CHARS as u64 * 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UserPromptMetadata {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) body: Arc<str>,
}

pub(crate) fn list_user_prompts(codex_home: &Path) -> Vec<UserPromptMetadata> {
    let prompts_dir = codex_home.join(PROMPTS_DIR);
    let Ok(entries) = fs::read_dir(prompts_dir) else {
        return Vec::new();
    };

    let mut prompts = entries
        .filter_map(Result::ok)
        .filter_map(|entry| metadata_from_path(entry.path()))
        .collect::<Vec<_>>();
    prompts.sort_by(|a, b| a.name.cmp(&b.name));
    prompts
}

pub(crate) fn prompt_command_name(prompt_name: &str) -> String {
    format!("prompt:{prompt_name}")
}

pub(crate) fn prompt_name_from_command(name: &str) -> Option<&str> {
    let prompt_name = name.strip_prefix("prompt:")?;
    valid_prompt_name(prompt_name).then_some(prompt_name)
}

fn metadata_from_path(path: PathBuf) -> Option<UserPromptMetadata> {
    if path.extension().and_then(|ext| ext.to_str()) != Some(PROMPT_EXT) {
        return None;
    }
    let name = path.file_stem()?.to_str()?.to_string();
    if !valid_prompt_name(&name) {
        return None;
    }
    if fs::metadata(&path).ok()?.len() > MAX_PROMPT_FILE_BYTES {
        return None;
    }
    let (description, body) = fs::read_to_string(path)
        .ok()
        .map(|contents| parse_prompt_document(&contents))?;
    if body.chars().count() > MAX_USER_INPUT_TEXT_CHARS {
        return None;
    }
    Some(UserPromptMetadata {
        name,
        description,
        body: Arc::from(body),
    })
}

fn valid_prompt_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn parse_prompt_document(contents: &str) -> (Option<String>, String) {
    let Some(after_opening) = contents.strip_prefix(FRONTMATTER_DELIMITER) else {
        return (None, contents.trim().to_string());
    };
    let after_opening = after_opening
        .strip_prefix("\r\n")
        .or_else(|| after_opening.strip_prefix('\n'));
    let Some(after_opening) = after_opening else {
        return (None, contents.trim().to_string());
    };

    let mut frontmatter = Vec::new();
    let mut body_start = None;
    let mut offset = FRONTMATTER_DELIMITER.len()
        + (contents[FRONTMATTER_DELIMITER.len()..].len() - after_opening.len());
    for line in after_opening.split_inclusive('\n') {
        let line_without_ending = line.trim_end_matches(['\r', '\n']);
        if line_without_ending == FRONTMATTER_DELIMITER {
            body_start = Some(offset + line.len());
            break;
        }
        frontmatter.push(line_without_ending);
        offset += line.len();
    }

    let Some(body_start) = body_start else {
        return (None, contents.trim().to_string());
    };
    let description = frontmatter
        .into_iter()
        .find_map(|line| line.strip_prefix("description:").map(str::trim))
        .map(unquote_yaml_string)
        .filter(|value| !value.is_empty());
    (description, contents[body_start..].trim().to_string())
}

fn unquote_yaml_string(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses_desktop_prompt_frontmatter_and_body() {
        let (description, body) = parse_prompt_document(
            "---\n\
description: Implement tasks from an OpenSpec change\n\
argument-hint: command arguments\n\
---\n\
\n\
Implement tasks from an OpenSpec change.\n",
        );

        assert_eq!(
            description,
            Some("Implement tasks from an OpenSpec change".to_string())
        );
        assert_eq!(body, "Implement tasks from an OpenSpec change.");
    }

    #[test]
    fn lists_markdown_prompts_by_file_stem() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let prompts_dir = codex_home.path().join(PROMPTS_DIR);
        fs::create_dir_all(&prompts_dir).expect("create prompts dir");
        fs::write(
            prompts_dir.join("opsx-apply.md"),
            "---\ndescription: Apply change\n---\n\nApply.",
        )
        .expect("write prompt");
        fs::write(prompts_dir.join("ignored.txt"), "ignore").expect("write ignored");

        assert_eq!(
            list_user_prompts(codex_home.path()),
            vec![UserPromptMetadata {
                name: "opsx-apply".to_string(),
                description: Some("Apply change".to_string()),
                body: Arc::from("Apply."),
            }]
        );
    }

    #[test]
    fn rejects_prompt_command_names_with_path_separators() {
        assert_eq!(prompt_name_from_command("prompt:../secret"), None);
        assert_eq!(prompt_name_from_command("prompt:.."), None);
        assert_eq!(
            prompt_name_from_command("prompt:opsx-apply"),
            Some("opsx-apply")
        );
    }

    #[test]
    fn empty_prompt_body_is_loaded_for_explicit_submit_error() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let prompts_dir = codex_home.path().join(PROMPTS_DIR);
        fs::create_dir_all(&prompts_dir).expect("create prompts dir");
        fs::write(
            prompts_dir.join("empty.md"),
            "---\ndescription: Empty\n---\n\n",
        )
        .expect("write prompt");

        assert_eq!(
            list_user_prompts(codex_home.path()),
            vec![UserPromptMetadata {
                name: "empty".to_string(),
                description: Some("Empty".to_string()),
                body: Arc::from(""),
            }]
        );
    }
}
