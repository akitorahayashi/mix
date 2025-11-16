use crate::error::AppError;
use std::env;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub(crate) struct SnippetStorage {
    commands_root: PathBuf,
    config_path: PathBuf,
}

impl SnippetStorage {
    pub fn new_default() -> Result<Self, AppError> {
        if let Ok(custom) = env::var("MIX_COMMANDS_ROOT") {
            return Self::from_root(PathBuf::from(custom));
        }

        let home = env::var("HOME")
            .map_err(|_| AppError::config_error("HOME environment variable not set"))?;
        let root = PathBuf::from(home).join(".config").join("mix");
        Self::from_root(root)
    }

    pub fn from_root<P: AsRef<Path>>(root: P) -> Result<Self, AppError> {
        let root = root.as_ref().to_path_buf();
        Ok(Self { commands_root: root.join("commands"), config_path: root.join("config.yml") })
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn enumerate_snippets(&self) -> Result<Vec<SnippetFile>, AppError> {
        if !self.commands_root.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in WalkDir::new(&self.commands_root) {
            let entry = entry.map_err(|err| AppError::config_error(err.to_string()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            let relative = entry
                .path()
                .strip_prefix(&self.commands_root)
                .map_err(|_| AppError::config_error("Unable to derive relative snippet path"))?;
            let relative_without_ext = relative.with_extension("");
            let relative_path = path_to_string(&relative_without_ext)?;
            let key = entry
                .path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| AppError::config_error("Snippet names must be valid UTF-8"))?
                .to_string();

            files.push(SnippetFile { key, relative_path, absolute_path: entry.into_path() });
        }

        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(files)
    }

    pub fn resolve_snippet(&self, raw_query: &str) -> Result<SnippetFile, AppError> {
        let normalized = normalize_query(raw_query)?;
        let mut exact_matches = Vec::new();
        let mut key_matches = Vec::new();
        let candidate_key =
            normalized.rsplit('/').next().unwrap_or(normalized.as_str()).to_string();

        for snippet in self.enumerate_snippets()? {
            if snippet.relative_path == normalized {
                exact_matches.push(snippet);
            } else if snippet.key == normalized || snippet.key == candidate_key {
                key_matches.push(snippet);
            }
        }

        if !exact_matches.is_empty() {
            if exact_matches.len() == 1 {
                return Ok(exact_matches.into_iter().next().unwrap());
            }
            return Err(AppError::config_error(format!(
                "Multiple snippets match '{raw_query}': {}",
                join_paths(&exact_matches)
            )));
        }

        if key_matches.is_empty() {
            return Err(AppError::not_found(format!(
                "No snippet named '{raw_query}' under {}",
                self.commands_root.display()
            )));
        }

        if key_matches.len() > 1 {
            return Err(AppError::config_error(format!(
                "Multiple snippets share the name '{raw_query}': {}",
                join_paths(&key_matches)
            )));
        }

        Ok(key_matches.into_iter().next().unwrap())
    }

    pub fn resolve_prompt_path(&self, relative: &Path) -> Result<PathBuf, AppError> {
        let cleaned = sanitize_relative_path(relative)?;
        let trimmed =
            cleaned.strip_prefix("commands/").unwrap_or(cleaned.as_str()).trim_start_matches('/');
        let resolved = if trimmed.is_empty() {
            self.commands_root.clone()
        } else {
            self.commands_root.join(trimmed)
        };

        if !resolved.starts_with(&self.commands_root) {
            return Err(AppError::config_error(format!(
                "Prompt path '{}' escapes the commands directory",
                relative.display()
            )));
        }

        if !resolved.exists() {
            return Err(AppError::not_found(format!(
                "Prompt file '{}' was not found",
                relative.display()
            )));
        }

        if !resolved.is_file() {
            return Err(AppError::config_error(format!(
                "Prompt path '{}' must point to a file",
                relative.display()
            )));
        }

        Ok(resolved)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SnippetFile {
    pub key: String,
    pub relative_path: String,
    pub absolute_path: PathBuf,
}

fn path_to_string(path: &Path) -> Result<String, AppError> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => parts.push(
                segment
                    .to_str()
                    .ok_or_else(|| AppError::config_error("Snippet paths must be UTF-8"))?
                    .to_string(),
            ),
            Component::CurDir => continue,
            _ => {
                return Err(AppError::config_error(
                    "Snippet paths cannot include traversal components",
                ));
            }
        }
    }

    Ok(parts.join("/"))
}

fn normalize_query(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return Err(AppError::config_error("Snippet name cannot be empty"));
    }

    let mut normalized = trimmed.replace('\\', "/");
    if let Some(stripped) = normalized.strip_prefix("commands/") {
        normalized = stripped.to_string();
    }
    if let Some(stripped) = normalized.strip_suffix(".md") {
        normalized = stripped.to_string();
    }

    ensure_safe_segments(&normalized)?;
    Ok(normalized)
}

fn sanitize_relative_path(path: &Path) -> Result<String, AppError> {
    path_to_string(path)
}

fn ensure_safe_segments(value: &str) -> Result<(), AppError> {
    if value.split('/').any(|segment| segment.is_empty() || segment == "..") {
        return Err(AppError::config_error(
            "Snippet paths cannot contain empty or traversal segments",
        ));
    }
    Ok(())
}

fn join_paths(snippets: &[SnippetFile]) -> String {
    snippets.iter().map(|s| s.relative_path.clone()).collect::<Vec<_>>().join(", ")
}
