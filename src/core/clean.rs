use crate::core::touch::{find_project_root, resolve_path};
use crate::error::AppError;
use std::fs;

pub struct CleanOutcome {
    pub message: String,
}

pub fn clean(key: Option<String>) -> Result<CleanOutcome, AppError> {
    let root = find_project_root()?;
    let mix_dir = root.join(".mix");

    match key {
        None => {
            // mix clean (Delete .mix root)
            if mix_dir.exists() {
                fs::remove_dir_all(&mix_dir)?;
                Ok(CleanOutcome { message: "Removed .mix directory".to_string() })
            } else {
                Ok(CleanOutcome { message: ".mix directory not found".to_string() })
            }
        }
        Some(k) => {
            // mix clean tk (Delete specific file)
            let relative_path = resolve_path(&k);
            let target_path = mix_dir.join(&relative_path);

            if target_path.exists() {
                fs::remove_file(&target_path)?;

                // Optional: Attempt to remove empty parent dirs
                // We walk up from the file's parent until we hit .mix
                if let Some(mut parent) = target_path.parent() {
                    while parent.starts_with(&mix_dir) && parent != mix_dir {
                        // Attempt to remove the directory. This will fail if it's not empty, which is exactly what we want.
                        if fs::remove_dir(parent).is_err() {
                            break;
                        }
                        if let Some(p) = parent.parent() {
                            parent = p;
                        } else {
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
