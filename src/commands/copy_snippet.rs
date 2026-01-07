use crate::commands::clipboard::Clipboard;
use crate::commands::touch::{find_project_root, validate_path};
use crate::error::AppError;
use crate::storage::SnippetStorage;
use std::borrow::Cow;
use std::fs;
use std::path::Path;

pub(crate) struct CopySnippet<'a> {
    pub query: &'a str,
}

#[derive(Debug, Clone)]
pub struct CopyOutcome {
    pub key: String,
    pub relative_path: String,
    pub absolute_path: std::path::PathBuf,
}

impl CopySnippet<'_> {
    pub fn execute(
        &self,
        storage: &SnippetStorage,
        clipboard: &dyn Clipboard,
    ) -> Result<CopyOutcome, AppError> {
        let snippet = storage.resolve_snippet(self.query)?;
        let content = fs::read_to_string(&snippet.absolute_path)?;
        let project_root = find_project_root().ok();
        let expanded = expand_placeholders(&content, project_root.as_deref());
        clipboard.copy(expanded.as_ref())?;

        Ok(CopyOutcome {
            key: snippet.key,
            relative_path: snippet.relative_path,
            absolute_path: snippet.absolute_path,
        })
    }
}

fn expand_placeholders<'a>(content: &'a str, project_root: Option<&Path>) -> Cow<'a, str> {
    let Some(root) = project_root else {
        return Cow::Borrowed(content);
    };

    if !content.contains("{{") {
        return Cow::Borrowed(content);
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
                return Cow::Owned(output);
            }
        }
    }

    output.push_str(remainder);
    Cow::Owned(output)
}

fn render_placeholder(raw_token: &str, project_root: &Path) -> String {
    let trimmed = raw_token.trim();
    if trimmed.is_empty() {
        return format!("{{{{{raw_token}}}}}");
    }

    if let Err(err) = validate_path(trimmed, Path::new(trimmed)) {
        return format!("[mx error: {}]", err);
    }

    let absolute = project_root.join(trimmed);
    match fs::read_to_string(&absolute) {
        Ok(contents) => contents,
        Err(err) => format!("[mx missing: {trimmed} ({})]", err.kind()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::test_support::{recording_clipboard, TestSnippetStorage};
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
        storage.write_snippet("commands/w/wc.md", "Section:\n{{.mx/info.md}}\nDone");
        let clipboard = recording_clipboard();

        let project_root = tempdir().expect("temp project root");
        let _guard = DirGuard::set(project_root.path());
        fs::create_dir_all(project_root.path().join(".mx")).expect("create .mx");
        fs::write(project_root.path().join(".mx/info.md"), "dynamic info").expect("write info");

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
        fs::create_dir_all(root.path().join(".mx")).expect("create .mx dir");
        fs::write(root.path().join(".mx/tasks.md"), "todo list").expect("write tasks");

        let source = "Next:\n{{.mx/tasks.md}}";
        let expanded = expand_placeholders(source, Some(root.path()));

        assert_eq!(expanded, "Next:\ntodo list");
    }

    #[test]
    fn expand_placeholders_handles_missing_files() {
        let root = tempdir().expect("temp root");
        let expanded = expand_placeholders("Missing: {{.mx/none.md}}", Some(root.path()));

        assert!(expanded.contains("[mx missing: .mx/none.md"));
    }

    #[test]
    fn expand_placeholders_blocks_traversal() {
        let root = tempdir().expect("temp root");
        let expanded = expand_placeholders("{{../secret}}", Some(root.path()));

        assert!(expanded.contains("[mx error:"));
    }

    #[test]
    fn expand_placeholders_skips_when_no_root() {
        let expanded = expand_placeholders("{{.mx/tasks.md}}", None);
        assert_eq!(expanded, "{{.mx/tasks.md}}");
    }
}
