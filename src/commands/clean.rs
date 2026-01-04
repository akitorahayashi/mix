use crate::commands::touch::{find_project_root, resolve_path, validate_path};
use crate::error::AppError;
use std::fs;

pub struct CleanOutcome {
    pub message: String,
}

pub fn clean(key: Option<String>) -> Result<CleanOutcome, AppError> {
    let root = find_project_root()?;
    let mx_dir = root.join("mx");

    match key {
        None => {
            // mx clean (Delete mx root)
            if mx_dir.exists() {
                fs::remove_dir_all(&mx_dir)?;
                Ok(CleanOutcome { message: "Removed mx directory".to_string() })
            } else {
                Ok(CleanOutcome { message: "mx directory not found".to_string() })
            }
        }
        Some(k) => {
            // mx clean tk (Delete specific file)
            let relative_path = resolve_path(&k);

            // Validate path for security (no traversal or absolute paths)
            validate_path(&k, &relative_path)?;

            let target_path = mx_dir.join(&relative_path);

            if target_path.exists() {
                fs::remove_file(&target_path)?;

                // Optional: Attempt to remove empty parent dirs
                // We walk up from the file's parent until we hit mx
                if let Some(parent) = target_path.parent() {
                    for p in parent.ancestors() {
                        if !p.starts_with(&mx_dir) || p == mx_dir {
                            break;
                        }
                        // Attempt to remove the directory. This will fail if it's not empty,
                        // which is what we want. If it fails, we stop.
                        if fs::remove_dir(p).is_err() {
                            break;
                        }
                    }
                }

                Ok(CleanOutcome { message: format!("Removed {}", target_path.display()) })
            } else {
                Err(AppError::not_found(format!("File not found: {}", target_path.display())))
            }
        }
    }
}
