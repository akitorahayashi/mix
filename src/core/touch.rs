use crate::error::AppError;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Predefined alias mappings for quick access to common context files.
static ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("tk", "tasks.md");
    m.insert("rq", "requirements.md");
    m.insert("rv", "review.md");
    m.insert("df", "diff.md");
    m.insert("pdt", "pending/tasks.md");
    m.insert("pdr", "pending/requirements.md");
    m.insert("wn", "warnings.md");
    m.insert("er", "error.md");
    m
});

pub struct TouchOutcome {
    pub key: String,
    pub path: PathBuf,
    pub existed: bool,
}

/// Resolves an input key to a relative path within the `.mix/` directory.
///
/// Resolution priority:
/// 1. Check if key matches a predefined alias
/// 2. If not, treat as a relative path and auto-append `.md` if no extension present
pub fn resolve_path(key: &str) -> PathBuf {
    // 1. Check alias map
    if let Some(mapped) = ALIASES.get(key) {
        return PathBuf::from(mapped);
    }

    // 2. Dynamic "tk{N}" Pattern
    if key.starts_with("tk") {
        let remainder = &key[2..];
        // Ensure remainder is non-empty and all numeric
        if !remainder.is_empty() && remainder.chars().all(char::is_numeric) {
            return PathBuf::from(format!("tasks/tasks{}.md", remainder));
        }
    }

    // 3. Generate dynamic path
    let mut path = PathBuf::from(key);

    // 4. Extension completion (if no extension specified)
    if path.extension().is_none() {
        path.set_extension("md");
    }

    path
}

/// Validates that the given key and resolved path are safe (no path traversal).
///
/// Returns an error if:
/// - The key contains `..` (parent directory reference)
/// - The resolved path contains absolute paths or traversal components
pub fn validate_path(key: &str, resolved: &Path) -> Result<(), AppError> {
    // First, perform a simple string check on the input key. This is a fast
    // rejection for the most common traversal attempts.
    if key.contains("..") {
        return Err(AppError::path_traversal(
            "Invalid path. Cannot create files outside of .mix directory.",
        ));
    }

    // For a more robust check, inspect the components of the resolved path.
    // This correctly handles various edge cases like absolute paths on both
    // Unix and Windows, without needing the path to exist on disk (which is
    // a flaw in the canonicalize approach).
    for component in resolved.components() {
        match component {
            // Only allow normal path components and current directory references.
            std::path::Component::Normal(_) | std::path::Component::CurDir => (),

            // Reject anything else. This covers:
            // - `RootDir` (`/`): Blocks absolute paths.
            // - `ParentDir` (`..`): Blocks traversal not caught by the string check.
            // - `Prefix` (`C:`): Blocks Windows absolute paths.
            _ => {
                return Err(AppError::path_traversal(
                    "Invalid path. Cannot create files outside of .mix directory.",
                ));
            }
        }
    }

    Ok(())
}
pub fn touch(key: &str) -> Result<TouchOutcome, AppError> {
    let root = find_project_root()?;
    let mix_dir = root.join(".mix");

    // 1. Create .mix directory
    if !mix_dir.exists() {
        fs::create_dir(&mix_dir)?;
    }

    // 2. Create .gitignore atomically
    let gitignore = mix_dir.join(".gitignore");
    let gitignore_exists = gitignore.exists();
    if !gitignore_exists {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&gitignore)?;
        writeln!(file, "*")?;
    }

    // 3. Resolve key to relative path (alias or dynamic)
    let relative_path = resolve_path(key);

    // 4. Validate path for security (no traversal or absolute paths)
    validate_path(key, &relative_path)?;

    let target_path = mix_dir.join(&relative_path);

    // 5. Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // 6. Create file atomically if not exists
    let existed = match OpenOptions::new().write(true).create_new(true).open(&target_path) {
        Ok(_) => false,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => true,
        Err(e) => return Err(e.into()),
    };

    Ok(TouchOutcome { key: key.to_string(), path: target_path, existed })
}

pub fn find_project_root() -> Result<PathBuf, AppError> {
    // For now, assume current directory is root or we look for .git
    // But simplest is to use current directory.
    // If we want to be robust, we can look for .git up the tree.
    // Given the request "Creates .mix/ in the project root", implies CWD usually.
    std::env::current_dir().map_err(AppError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

    // === resolve_path tests ===

    #[test]
    fn test_resolve_path_alias_tk() {
        let path = resolve_path("tk");
        assert_eq!(path, PathBuf::from("tasks.md"));
    }

    #[test]
    fn test_resolve_path_alias_pdt() {
        let path = resolve_path("pdt");
        assert_eq!(path, PathBuf::from("pending/tasks.md"));
    }

    #[test]
    fn test_resolve_path_dynamic_alias_tk1() {
        let path = resolve_path("tk1");
        assert_eq!(path, PathBuf::from("tasks/tasks1.md"));
    }

    #[test]
    fn test_resolve_path_dynamic_alias_tk99() {
        let path = resolve_path("tk99");
        assert_eq!(path, PathBuf::from("tasks/tasks99.md"));
    }

    #[test]
    fn test_resolve_path_dynamic_simple() {
        let path = resolve_path("myfile");
        assert_eq!(path, PathBuf::from("myfile.md"));
    }

    #[test]
    fn test_resolve_path_dynamic_nested() {
        let path = resolve_path("a/b/c");
        assert_eq!(path, PathBuf::from("a/b/c.md"));
    }

    #[test]
    fn test_resolve_path_with_extension_json() {
        let path = resolve_path("data.json");
        assert_eq!(path, PathBuf::from("data.json"));
    }

    #[test]
    fn test_resolve_path_with_extension_txt() {
        let path = resolve_path("logs.txt");
        assert_eq!(path, PathBuf::from("logs.txt"));
    }

    #[test]
    fn test_resolve_path_with_extension_md() {
        let path = resolve_path("notes.md");
        assert_eq!(path, PathBuf::from("notes.md"));
    }

    // === validate_path tests ===

    #[test]
    fn test_validate_path_simple_ok() {
        let resolved = PathBuf::from("test.md");
        assert!(validate_path("test", &resolved).is_ok());
    }

    #[test]
    fn test_validate_path_nested_ok() {
        let resolved = PathBuf::from("a/b/c.md");
        assert!(validate_path("a/b/c", &resolved).is_ok());
    }

    #[test]
    fn test_validate_path_traversal_dotdot() {
        let resolved = PathBuf::from("../hack.md");
        let result = validate_path("../hack", &resolved);
        assert!(result.is_err());
        if let Err(AppError::PathTraversal(msg)) = result {
            assert!(msg.contains("outside of .mix"));
        } else {
            panic!("Expected PathTraversal error");
        }
    }

    #[test]
    fn test_validate_path_traversal_embedded() {
        let resolved = PathBuf::from("foo/../bar.md");
        let result = validate_path("foo/../bar", &resolved);
        assert!(result.is_err());
    }

    // === touch integration tests ===

    #[test]
    #[serial]
    fn test_touch_creates_mix_and_gitignore() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let outcome = touch("tk").unwrap();

        assert!(dir.path().join(".mix").exists());
        assert!(dir.path().join(".mix/.gitignore").exists());
        let gitignore_content = fs::read_to_string(dir.path().join(".mix/.gitignore")).unwrap();
        assert_eq!(gitignore_content.trim(), "*");
        assert_eq!(outcome.key, "tk");
        assert!(outcome.path.ends_with(".mix/tasks.md"));
        assert!(!outcome.existed);
    }

    #[test]
    #[serial]
    fn test_touch_nested_file() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let outcome = touch("pdt").unwrap();

        assert!(dir.path().join(".mix/pending/tasks.md").exists());
        assert!(!outcome.existed);
    }

    #[test]
    #[serial]
    fn test_touch_idempotency() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        touch("tk").unwrap();
        let outcome = touch("tk").unwrap();

        assert!(outcome.existed);
    }

    #[test]
    #[serial]
    fn test_touch_dynamic_creates_file() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let outcome = touch("random_name").unwrap();

        assert!(dir.path().join(".mix/random_name.md").exists());
        assert!(!outcome.existed);
        assert!(outcome.path.ends_with("random_name.md"));
    }

    #[test]
    #[serial]
    fn test_touch_dynamic_nested_creates_directories() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let outcome = touch("a/b/c").unwrap();

        assert!(dir.path().join(".mix/a/b/c.md").exists());
        assert!(dir.path().join(".mix/a/b").is_dir());
        assert!(!outcome.existed);
    }

    #[test]
    #[serial]
    fn test_touch_with_extension_preserves() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let outcome = touch("data.json").unwrap();

        assert!(dir.path().join(".mix/data.json").exists());
        assert!(!dir.path().join(".mix/data.json.md").exists());
        assert!(!outcome.existed);
    }

    #[test]
    #[serial]
    fn test_touch_path_traversal_rejected() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let result = touch("../hack");

        assert!(result.is_err());
        if let Err(AppError::PathTraversal(_)) = result {
            // Expected
        } else {
            panic!("Expected PathTraversal error");
        }
        // Ensure no file was created outside .mix
        assert!(!dir.path().join("hack.md").exists());
    }
}
