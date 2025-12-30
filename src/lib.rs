//! Library entry point exposing the mix CLI command handlers.

pub mod error;

mod commands;
mod storage;

use commands::clean;
use commands::clipboard::clipboard_from_env;
use commands::copy_snippet::CopySnippet;
use commands::list_snippets;
use commands::touch;
use error::AppError;
use storage::SnippetStorage;
use std::path::PathBuf;

pub use commands::clean::CleanOutcome;

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

pub struct TouchOutcome {
    pub key: String,
    pub path: PathBuf,
    pub existed: bool,
    pub overwritten: bool,
}

pub fn clean_context(key: Option<String>) -> Result<CleanOutcome, AppError> {
    clean::clean(key)
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

pub fn touch_context(key: &str, paste: bool, force: bool) -> Result<TouchOutcome, AppError> {
    let outcome = touch::touch(key, force)?;

    // Paste if:
    // 1. File was just created (!existed)
    // 2. OR file was overwritten (overwritten)
    if paste && (!outcome.existed || outcome.overwritten) {
        let clipboard = clipboard_from_env()?;
        let content = clipboard.paste()?;
        std::fs::write(&outcome.path, content)?;
    }

    Ok(TouchOutcome {
        key: outcome.key,
        path: outcome.path,
        existed: outcome.existed,
        overwritten: outcome.overwritten,
    })
}
