use crate::core::clipboard::clipboard_from_env;
use crate::core::copy_snippet::CopySnippet;
use crate::core::generate_slash_commands;
use crate::core::list_snippets;
use crate::error::AppError;
use crate::storage::SnippetStorage;
use std::path::PathBuf;

pub use crate::core::generate_slash_commands::SlashTarget;

pub enum SlashRequest {
    All,
    Only(SlashTarget),
}

#[derive(Debug, Clone)]
pub struct CopyOutcome {
    pub key: String,
    pub relative_path: String,
    pub absolute_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ListEntry {
    pub key: String,
    pub relative_path: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SlashGenerationOutcome {
    pub target: SlashTarget,
    pub path: PathBuf,
}

pub fn copy_snippet(query: &str) -> Result<CopyOutcome, AppError> {
    let storage = SnippetStorage::new_default()?;
    let clipboard = clipboard_from_env()?;
    let result = CopySnippet { query }.execute(&storage, clipboard.as_ref())?;
    Ok(CopyOutcome {
        key: result.key,
        relative_path: result.relative_path,
        absolute_path: result.absolute_path,
    })
}

pub fn list_snippets() -> Result<Vec<ListEntry>, AppError> {
    let storage = SnippetStorage::new_default()?;
    let entries = list_snippets::list(&storage)?;
    Ok(entries
        .into_iter()
        .map(|entry| ListEntry {
            key: entry.key,
            relative_path: entry.relative_path,
            title: entry.title,
            description: entry.description,
        })
        .collect())
}

pub fn generate_slash_commands(
    request: SlashRequest,
) -> Result<Vec<SlashGenerationOutcome>, AppError> {
    let storage = SnippetStorage::new_default()?;
    let targets: Vec<SlashTarget> = match request {
        SlashRequest::All => SlashTarget::ALL.to_vec(),
        SlashRequest::Only(target) => vec![target],
    };
    let artifacts = generate_slash_commands::generate(&storage, &targets)?;
    Ok(artifacts
        .into_iter()
        .map(|artifact| SlashGenerationOutcome { target: artifact.target, path: artifact.path })
        .collect())
}
