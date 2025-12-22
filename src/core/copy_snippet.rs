use crate::core::clipboard::Clipboard;
use crate::core::touch::{find_project_root, validate_path};
use crate::error::AppError;
use crate::storage::SnippetStorage;
use std::fs;
use std::path::Path;

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
        let project_root = find_project_root().ok();
        let expanded = expand_placeholders(&content, project_root.as_deref());
        clipboard.copy(&expanded)?;

        Ok(CopyOutput {
            key: snippet.key,
            relative_path: snippet.relative_path,
            absolute_path: snippet.absolute_path,
        })
    }
}

fn expand_placeholders(content: &str, project_root: Option<&Path>) -> String {
    let Some(root) = project_root else {
        return content.to_string();
    };

    if !content.contains("{{") {
        return content.to_string();
    }

    let mut remainder = content;
    let mut output = String::with_capacity(content.len());

    while let Some(start) = remainder.find("{{") {
        output.push_str(&remainder[..start]);
        let tail = &remainder[start + 2..];

        match tail.find("}}") {
            Some(end) => {
                let token = &tail[..end];
                output.push_str(&render_placeholder(token, root));
                remainder = &tail[end + 2..];
            }
            None => {
                output.push_str(&remainder[start..]);
                return output;
            }
        }
    }

    output.push_str(remainder);
    output
}

fn render_placeholder(raw_token: &str, project_root: &Path) -> String {
    let trimmed = raw_token.trim();
    if trimmed.is_empty() {
        return format!("{{{{{raw_token}}}}}");
    }

    if let Err(err) = validate_path(trimmed, Path::new(trimmed)) {
        return format!("[mix error: {}]", err);
    }

    let absolute = project_root.join(trimmed);
    match fs::read_to_string(&absolute) {
        Ok(contents) => contents,
        Err(err) => format!("[mix missing: {trimmed} ({})]", err.kind()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::{recording_clipboard, TestSnippetStorage};
    use serial_test::serial;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    struct DirGuard {
        original: std::path::PathBuf,
    }

    impl DirGuard {
        fn set(dir: &Path) -> Self {
            let original = env::current_dir().expect("read cwd");
            env::set_current_dir(dir).expect("set cwd");
            Self { original }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }

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
    #[serial]
    fn copy_snippet_expands_placeholders_into_clipboard() {
        let storage = TestSnippetStorage::new();
        storage.write_snippet("commands/w/wc.md", "Section:\n{{.mix/info.md}}\nDone");
        let clipboard = recording_clipboard();

        let project_root = tempdir().expect("temp project root");
        let _guard = DirGuard::set(project_root.path());
        fs::create_dir_all(project_root.path().join(".mix")).expect("create .mix");
        fs::write(project_root.path().join(".mix/info.md"), "dynamic info").expect("write info");

        let result = CopySnippet { query: "wc" }
            .execute(&storage.storage, clipboard.as_ref())
            .expect("copy should succeed");

        assert_eq!(result.key, "wc");
        assert!(clipboard.contents().contains("dynamic info"));
        assert_eq!(clipboard.contents(), "Section:\ndynamic info\nDone");
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

    #[test]
    fn expand_placeholders_inserts_file_contents() {
        let root = tempdir().expect("temp root");
        fs::create_dir_all(root.path().join(".mix")).expect("create mix dir");
        fs::write(root.path().join(".mix/tasks.md"), "todo list").expect("write tasks");

        let source = "Next:\n{{.mix/tasks.md}}";
        let expanded = expand_placeholders(source, Some(root.path()));

        assert_eq!(expanded, "Next:\ntodo list");
    }

    #[test]
    fn expand_placeholders_handles_missing_files() {
        let root = tempdir().expect("temp root");
        let expanded = expand_placeholders("Missing: {{.mix/none.md}}", Some(root.path()));

        assert!(expanded.contains("[mix missing: .mix/none.md"));
    }

    #[test]
    fn expand_placeholders_blocks_traversal() {
        let root = tempdir().expect("temp root");
        let expanded = expand_placeholders("{{../secret}}", Some(root.path()));

        assert!(expanded.contains("[mix error:"));
    }

    #[test]
    fn expand_placeholders_skips_when_no_root() {
        let expanded = expand_placeholders("{{.mix/tasks.md}}", None);
        assert_eq!(expanded, "{{.mix/tasks.md}}");
    }
}
