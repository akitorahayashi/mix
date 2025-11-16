use crate::error::AppError;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub(crate) trait Clipboard {
    fn copy(&self, text: &str) -> Result<(), AppError>;
}

pub(crate) fn clipboard_from_env() -> Result<Box<dyn Clipboard>, AppError> {
    if let Ok(path) = env::var("MIX_CLIPBOARD_FILE") {
        return Ok(Box::new(FileClipboard::new(PathBuf::from(path))?));
    }

    Ok(Box::new(SystemClipboard::detect()?))
}

struct ClipboardCommand {
    program: String,
    args: Vec<String>,
}

impl ClipboardCommand {
    fn new(program: impl Into<String>) -> Self {
        Self { program: program.into(), args: Vec::new() }
    }

    fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }
}

pub(crate) struct SystemClipboard {
    command: ClipboardCommand,
}

impl SystemClipboard {
    fn detect() -> Result<Self, AppError> {
        if let Ok(custom) = env::var("MIX_CLIPBOARD_CMD") {
            let mut parts = custom.split_whitespace();
            let program = parts
                .next()
                .ok_or_else(|| AppError::clipboard_error("MIX_CLIPBOARD_CMD is empty"))?;
            let args: Vec<String> = parts.map(|s| s.to_string()).collect();
            return Ok(Self { command: ClipboardCommand { program: program.to_string(), args } });
        }

        match env::consts::OS {
            "macos" => Ok(Self { command: ClipboardCommand::new("pbcopy") }),
            "linux" => Self::detect_linux(),
            "windows" => Ok(Self { command: ClipboardCommand::new("clip") }),
            other => Err(AppError::clipboard_error(format!(
                "Unsupported platform '{other}' for clipboard operations"
            ))),
        }
    }

    fn detect_linux() -> Result<Self, AppError> {
        if Self::command_available("wl-copy", ["--version"]) {
            return Ok(Self { command: ClipboardCommand::new("wl-copy") });
        }

        if Self::command_available("xclip", ["-version"]) {
            return Ok(Self {
                command: ClipboardCommand::new("xclip").with_args(["-selection", "clipboard"]),
            });
        }

        Err(AppError::clipboard_error(
            "No supported clipboard command found. Install wl-copy or xclip, or set MIX_CLIPBOARD_FILE.",
        ))
    }

    fn command_available<'a, I>(program: &str, args: I) -> bool
    where
        I: IntoIterator<Item = &'a str>,
    {
        Command::new(program)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }
}

impl Clipboard for SystemClipboard {
    fn copy(&self, text: &str) -> Result<(), AppError> {
        let mut cmd = Command::new(&self.command.program);
        cmd.args(&self.command.args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let mut child = cmd.spawn().map_err(|err| {
            AppError::clipboard_error(format!(
                "Failed to run clipboard command '{}': {err}",
                self.command.program
            ))
        })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|err| {
                AppError::clipboard_error(format!(
                    "Failed to send data to clipboard command: {err}"
                ))
            })?;
        }

        let status = child
            .wait()
            .map_err(|err| AppError::clipboard_error(format!("Clipboard command failed: {err}")))?;

        if status.success() {
            Ok(())
        } else {
            Err(AppError::clipboard_error(format!("Clipboard command exited with status {status}")))
        }
    }
}

pub(crate) struct FileClipboard {
    path: PathBuf,
}

impl FileClipboard {
    pub fn new(path: PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self { path })
    }
}

impl Clipboard for FileClipboard {
    fn copy(&self, text: &str) -> Result<(), AppError> {
        fs::write(&self.path, text).map_err(|err| AppError::clipboard_error(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn file_clipboard_persists_contents() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("clipboard.txt");
        let clip = FileClipboard::new(file.clone()).expect("file clipboard should init");
        clip.copy("example").expect("write should succeed");
        let saved = fs::read_to_string(&file).expect("file should exist");
        assert_eq!(saved, "example");
    }
}
