use crate::commands::touch::{find_project_root, resolve_path, validate_path};
use crate::error::AppError;
use std::fs;

/// Displays the contents of a context file from the `.mx/` directory.
///
/// This command reuses the same path resolution logic as `touch`, supporting:
/// - Predefined aliases (tk, rq, pdt, etc.)
/// - Dynamic numbered aliases (tk1, tk2, etc.)
/// - Pending prefix (pd-tk, pd-rq, etc.)
/// - Custom relative paths with automatic .md extension
///
/// # Arguments
///
/// * `key` - The key to resolve to a file path (e.g., "tk", "rq", "docs/spec")
///
/// # Returns
///
/// The file contents as a String, or an error if:
/// - The file does not exist
/// - Path traversal is attempted
/// - The project root cannot be found
///
/// # Examples
///
/// ```no_run
/// use mx::cat_context;
///
/// // Read the tasks file
/// let content = cat_context("tk").expect("Failed to read tasks");
/// println!("{}", content);
/// ```
pub fn cat(key: &str) -> Result<String, AppError> {
    // Find the project root directory (where .mx/ directory is or should be)
    let root = find_project_root()?;

    // Resolve the key to a relative path (e.g., "tk" -> "tasks.md")
    let relative_path = resolve_path(key);

    // Validate the path to prevent traversal attacks
    validate_path(key, &relative_path)?;

    // Build the full path to the file
    let mx_dir = root.join(".mx");
    let full_path = mx_dir.join(&relative_path);

    // Check if the file exists
    if !full_path.exists() {
        return Err(AppError::not_found(format!(
            "⚠️ Context file not found: {}",
            relative_path.display()
        )));
    }

    // Check if it's a file (not a directory)
    if !full_path.is_file() {
        return Err(AppError::not_found(format!(
            "⚠️ Path is not a file: {}",
            relative_path.display()
        )));
    }

    // Read and return the file contents
    fs::read_to_string(&full_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read {}: {}", relative_path.display(), e),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn cat_reads_existing_file() {
        let temp = tempdir().unwrap();
        env::set_current_dir(&temp).unwrap();

        // Create a context file with known content
        let mx_dir = temp.path().join(".mx");
        fs::create_dir_all(&mx_dir).unwrap();
        let tasks_path = mx_dir.join("tasks.md");
        let expected_content = "# Test Tasks\n\n- Task 1\n- Task 2\n";
        fs::write(&tasks_path, expected_content).unwrap();

        // Read it back using cat
        let result = cat("tk").unwrap();
        assert_eq!(result, expected_content);
    }

    #[test]
    fn cat_returns_error_for_missing_file() {
        let temp = tempdir().unwrap();
        env::set_current_dir(&temp).unwrap();

        // Ensure .mx directory exists but file doesn't
        fs::create_dir_all(temp.path().join(".mx")).unwrap();

        let result = cat("tk");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("⚠️"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn cat_rejects_path_traversal() {
        let temp = tempdir().unwrap();
        env::set_current_dir(&temp).unwrap();

        let result = cat("../etc/passwd");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::PathTraversal(_)));
    }

    #[test]
    fn cat_handles_empty_file() {
        let temp = tempdir().unwrap();
        env::set_current_dir(&temp).unwrap();

        // Create an empty file
        let mx_dir = temp.path().join(".mx");
        fs::create_dir_all(&mx_dir).unwrap();
        let empty_path = mx_dir.join("empty.md");
        fs::write(&empty_path, "").unwrap();

        let result = cat("empty").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn cat_resolves_aliases_correctly() {
        let temp = tempdir().unwrap();
        env::set_current_dir(&temp).unwrap();

        // Create files for different aliases
        let mx_dir = temp.path().join(".mx");
        fs::create_dir_all(&mx_dir).unwrap();

        // Standard alias
        let content = "requirements content";
        fs::write(mx_dir.join("requirements.md"), content).unwrap();
        assert_eq!(cat("rq").unwrap(), content);

        // Nested alias
        fs::create_dir_all(mx_dir.join("pending")).unwrap();
        let nested_content = "pending tasks";
        fs::write(mx_dir.join("pending/tasks.md"), nested_content).unwrap();
        assert_eq!(cat("pdt").unwrap(), nested_content);
    }

    #[test]
    fn cat_errors_on_directory() {
        let temp = tempdir().unwrap();
        env::set_current_dir(&temp).unwrap();

        // Create a directory with .md extension to simulate the edge case
        let mx_dir = temp.path().join(".mx");
        fs::create_dir_all(mx_dir.join("somedir.md")).unwrap();

        let result = cat("somedir.md");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("⚠️"));
        assert!(err_msg.contains("not a file"));
    }
}
