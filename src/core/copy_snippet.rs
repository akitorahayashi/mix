use crate::core::clipboard::Clipboard;
use crate::error::AppError;
use crate::storage::SnippetStorage;
use std::fs;

pub(crate) struct CopySnippet<'a> {
    pub query: &'a str,
}

#[derive(Debug, Clone)]
pub(crate) struct CopyOutput {
    pub key: String,
    pub relative_path: String,
    pub absolute_path: std::path::PathBuf,
}

impl CopySnippet<'_> {
    pub fn execute(
        &self,
        storage: &SnippetStorage,
        clipboard: &dyn Clipboard,
    ) -> Result<CopyOutput, AppError> {
        let snippet = storage.resolve_snippet(self.query)?;
        let content = fs::read_to_string(&snippet.absolute_path)?;
        clipboard.copy(&content)?;

        Ok(CopyOutput {
            key: snippet.key,
            relative_path: snippet.relative_path,
            absolute_path: snippet.absolute_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::{recording_clipboard, TestSnippetStorage};

    #[test]
    fn copy_snippet_pushes_contents_to_clipboard() {
        let storage = TestSnippetStorage::new();
        let snippet_path = storage.write_snippet("commands/w/wc.md", "example content");
        let clipboard = recording_clipboard();

        let output = CopySnippet { query: "wc" }
            .execute(&storage.storage, clipboard.as_ref())
            .expect("copy should succeed");

        assert_eq!(output.key, "wc");
        assert_eq!(output.relative_path, "w/wc");
        assert_eq!(output.absolute_path, snippet_path);
        assert_eq!(clipboard.contents(), "example content");
    }

    #[test]
    fn copy_snippet_requires_existing_file() {
        let storage = TestSnippetStorage::new();
        let clipboard = recording_clipboard();

        let err = CopySnippet { query: "missing" }
            .execute(&storage.storage, clipboard.as_ref())
            .expect_err("copy should fail for missing snippet");

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
