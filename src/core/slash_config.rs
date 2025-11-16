use crate::error::AppError;
use crate::storage::SnippetStorage;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct CommandMetadata {
    pub key: String,
    pub title: String,
    pub description: String,
    pub prompt_file: PathBuf,
}

#[derive(Debug)]
pub(crate) struct SlashConfig {
    commands: Vec<CommandMetadata>,
}

impl SlashConfig {
    pub fn load_optional(storage: &SnippetStorage) -> Result<Option<Self>, AppError> {
        let path = storage.config_path();
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(Self::read_from(path)?))
    }

    pub fn load_required(storage: &SnippetStorage) -> Result<Self, AppError> {
        Self::load_optional(storage)?.ok_or_else(|| {
            AppError::config_error(format!(
                "Slash command config not found at {}",
                storage.config_path().display()
            ))
        })
    }

    fn read_from(path: &Path) -> Result<Self, AppError> {
        let raw = fs::read_to_string(path)?;
        let parsed: RawConfig = serde_yaml::from_str(&raw).map_err(|err| {
            AppError::config_error(format!("Invalid YAML in {}: {err}", path.display()))
        })?;

        let mut commands = Vec::new();
        for (key, value) in parsed.commands {
            let prompt = PathBuf::from(value.prompt_file);
            if prompt.is_absolute()
                || prompt.components().any(|component| matches!(component, Component::ParentDir))
            {
                return Err(AppError::config_error(format!(
                    "Prompt file for command '{key}' must stay within the commands directory"
                )));
            }

            commands.push(CommandMetadata {
                key,
                title: value.title,
                description: value.description,
                prompt_file: prompt,
            });
        }

        Ok(Self { commands })
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommandMetadata> {
        self.commands.iter()
    }

    pub fn into_map(self) -> HashMap<String, CommandMetadata> {
        self.commands.into_iter().map(|cmd| (cmd.key.clone(), cmd)).collect()
    }
}

#[derive(Deserialize)]
struct RawConfig {
    commands: HashMap<String, RawCommand>,
}

#[derive(Deserialize)]
struct RawCommand {
    title: String,
    description: String,
    #[serde(rename = "prompt-file")]
    prompt_file: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::TestSnippetStorage;

    #[test]
    fn load_optional_returns_none_when_missing() {
        let storage = TestSnippetStorage::new();
        assert!(SlashConfig::load_optional(&storage.storage).unwrap().is_none());
    }

    #[test]
    fn load_required_parses_commands() {
        let storage = TestSnippetStorage::new();
        storage.write_config(
            r#"---
commands:
  demo:
    title: "Demo"
    description: "Example"
    prompt-file: "commands/demo.md"
"#,
        );
        let cfg = SlashConfig::load_required(&storage.storage).expect("config should load");
        let items: Vec<_> = cfg.iter().collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].key, "demo");
    }
}
