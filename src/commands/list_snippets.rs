use crate::error::AppError;
use crate::storage::SnippetStorage;

#[derive(Debug, Clone)]
pub(crate) struct ListEntry {
    pub key: String,
    pub relative_path: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub(crate) fn list(storage: &SnippetStorage) -> Result<Vec<ListEntry>, AppError> {
    let snippets = storage.enumerate_snippets()?;

    let mut entries: Vec<ListEntry> = snippets
        .into_iter()
        .map(|snippet| ListEntry {
            key: snippet.key,
            relative_path: snippet.relative_path,
            title: None,
            description: None,
        })
        .collect();

    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::test_support::TestSnippetStorage;

    #[test]
    fn list_merges_metadata_when_available() {
        let storage = TestSnippetStorage::new();
        storage.write_snippet("commands/w/wc.md", "content");
        storage.write_snippet("commands/sn.md", "another");
        storage.write_config(
            r#"---
commands:
  wc:
    title: "Work critical"
    description: "Plan and execute"
    prompt-file: "commands/w/wc.md"
"#,
        );

        let entries = list(&storage.storage).expect("list should succeed");
        assert_eq!(entries.len(), 2);
        let wc = entries.iter().find(|e| e.key == "wc").unwrap();
        assert_eq!(wc.title.as_deref(), None); // Metadata support removed
        assert_eq!(wc.description.as_deref(), None); // Metadata support removed
        assert!(entries.iter().any(|e| e.key == "sn" && e.title.is_none()));
    }

    #[test]
    fn list_handles_empty_storage() {
        let storage = TestSnippetStorage::new();
        let entries = list(&storage.storage).expect("list should succeed");
        assert!(entries.is_empty());
    }
}
