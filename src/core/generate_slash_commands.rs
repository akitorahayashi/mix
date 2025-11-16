use crate::core::slash_config::{CommandMetadata, SlashConfig};
use crate::error::AppError;
use crate::storage::SnippetStorage;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SlashTarget {
    Codex,
    Claude,
    Gemini,
}

impl SlashTarget {
    pub const ALL: [SlashTarget; 3] =
        [SlashTarget::Codex, SlashTarget::Claude, SlashTarget::Gemini];

    pub fn label(&self) -> &'static str {
        match self {
            SlashTarget::Codex => "codex",
            SlashTarget::Claude => "claude",
            SlashTarget::Gemini => "gemini",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SlashGenerationOutcome {
    pub target: SlashTarget,
    pub path: PathBuf,
}

pub(crate) fn generate(
    storage: &SnippetStorage,
    targets: &[SlashTarget],
) -> Result<Vec<SlashGenerationOutcome>, AppError> {
    if targets.is_empty() {
        return Ok(Vec::new());
    }

    let config = SlashConfig::load_required(storage)?;
    let mut destinations = BTreeMap::new();
    for target in targets {
        let dest = destination_for_target(*target)?;
        clean_destination(&dest)?;
        destinations.insert(*target, dest);
    }

    let mut artifacts = Vec::new();
    for command in config.iter() {
        let prompt_path = storage.resolve_prompt_path(&command.prompt_file)?;
        let prompt_content = fs::read_to_string(&prompt_path)?;

        for target in targets {
            let dest = destinations
                .get(target)
                .expect("destination map should contain entries for every target");
            let (relative_name, rendered) = render(*target, command, &prompt_content)?;
            let output_path = dest.join(relative_name);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, rendered)?;
            artifacts.push(SlashGenerationOutcome { target: *target, path: output_path });
        }
    }

    Ok(artifacts)
}

fn render(
    target: SlashTarget,
    command: &CommandMetadata,
    prompt: &str,
) -> Result<(PathBuf, String), AppError> {
    match target {
        SlashTarget::Codex => {
            Ok((PathBuf::from(format!("{}.md", command.key)), prompt.to_string()))
        }
        SlashTarget::Claude => {
            let content = format!(
                "---\n{}{}---\n\n{}",
                format_yaml_field("title", &command.title),
                format_yaml_field("description", &command.description),
                prompt.trim_end_matches('\n')
            );
            Ok((PathBuf::from(format!("{}.md", command.key)), content))
        }
        SlashTarget::Gemini => {
            let description_json = serde_json::to_string(&command.description).map_err(|err| {
                AppError::config_error(format!("Failed to serialize description: {err}"))
            })?;
            let mut body = String::new();
            body.push_str(&format!("description = {}\n\n", description_json));
            body.push_str("prompt = \"\"\"\n");
            body.push_str(prompt.trim_end_matches('\n'));
            body.push_str("\n\"\"\"\n");
            Ok((PathBuf::from(format!("{}.toml", command.key)), body))
        }
    }
}

fn format_yaml_field(field: &str, value: &str) -> String {
    let escaped = value.replace('"', "\\\"").replace('\n', "\\n");
    format!("{field}: \"{escaped}\"\n")
}

fn destination_for_target(target: SlashTarget) -> Result<PathBuf, AppError> {
    let env_var = match target {
        SlashTarget::Codex => "MIX_CODEX_DIR",
        SlashTarget::Claude => "MIX_CLAUDE_DIR",
        SlashTarget::Gemini => "MIX_GEMINI_DIR",
    };

    if let Ok(path) = env::var(env_var) {
        return Ok(PathBuf::from(path));
    }

    let home = env::var("HOME").map_err(|_| {
        AppError::config_error("HOME environment variable not set for slash generation")
    })?;
    let base = PathBuf::from(home);
    let path = match target {
        SlashTarget::Codex => base.join(".codex").join("prompts"),
        SlashTarget::Claude => base.join(".claude").join("commands"),
        SlashTarget::Gemini => base.join(".gemini").join("commands"),
    };
    Ok(path)
}

fn clean_destination(path: &Path) -> Result<(), AppError> {
    // Safety guard: avoid catastrophic deletion if MIX_*_DIR is misconfigured.
    if !path.is_absolute() || path.parent().is_none() {
        return Err(AppError::config_error(format!(
            "Refusing to clean unsafe destination: {}",
            path.display()
        )));
    }

    fs::create_dir_all(path)?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            fs::remove_dir_all(&entry_path)?;
        } else {
            fs::remove_file(&entry_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::TestSnippetStorage;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn generates_all_targets() {
        let storage = TestSnippetStorage::new();
        storage.write_snippet("commands/w/wc.md", "prompt text");
        storage.write_config(
            r#"---
commands:
  wc:
    title: "Work"
    description: "Do work"
    prompt-file: "commands/w/wc.md"
"#,
        );

        let temp = tempdir().unwrap();
        let codex_dir = temp.path().join("codex");
        let claude_dir = temp.path().join("claude");
        let gemini_dir = temp.path().join("gemini");
        env::set_var("MIX_CODEX_DIR", &codex_dir);
        env::set_var("MIX_CLAUDE_DIR", &claude_dir);
        env::set_var("MIX_GEMINI_DIR", &gemini_dir);

        let mut artifacts =
            generate(&storage.storage, &SlashTarget::ALL).expect("generation succeeds");
        artifacts.sort_by(|a, b| a.target.cmp(&b.target));
        assert_eq!(artifacts.len(), 3);
        for artifact in artifacts {
            let contents = fs::read_to_string(&artifact.path).expect("file should exist");
            match artifact.target {
                SlashTarget::Codex => assert_eq!(contents, "prompt text"),
                SlashTarget::Claude => assert!(contents.contains("title:")),
                SlashTarget::Gemini => assert!(contents.contains("prompt =")),
            }
        }

        env::remove_var("MIX_CODEX_DIR");
        env::remove_var("MIX_CLAUDE_DIR");
        env::remove_var("MIX_GEMINI_DIR");
    }
}
