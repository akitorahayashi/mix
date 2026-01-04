use crate::error::AppError;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub(crate) trait Clipboard {
    fn copy(&self, text: &str) -> Result<(), AppError>;
    fn paste(&self) -> Result<String, AppError>;
}

pub(crate) fn clipboard_from_env() -> Result<Box<dyn Clipboard>, AppError> {
    if let Ok(path) = env::var("MX_CLIPBOARD_FILE") {
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
    copy_command: ClipboardCommand,
    paste_command: ClipboardCommand,
}

impl SystemClipboard {
    fn detect() -> Result<Self, AppError> {
        if let Ok(custom) = env::var("MX_CLIPBOARD_CMD") {
            let mut parts = custom.split_whitespace();
            let program = parts
                .next()
                .ok_or_else(|| AppError::clipboard_error("MX_CLIPBOARD_CMD is empty"))?;
            let args: Vec<String> = parts.map(|s| s.to_string()).collect();
            // For custom commands, assume same command for both copy and paste (copy via stdin, paste via stdout)
            let copy_cmd = ClipboardCommand { program: program.to_string(), args: args.clone() };
            let paste_cmd = ClipboardCommand { program: program.to_string(), args };
            return Ok(Self { copy_command: copy_cmd, paste_command: paste_cmd });
        }

        match env::consts::OS {
            "macos" => Ok(Self {
                copy_command: ClipboardCommand::new("pbcopy"),
                paste_command: ClipboardCommand::new("pbpaste"),
            }),
            "linux" => Self::detect_linux(),
            "windows" => Ok(Self {
                copy_command: ClipboardCommand::new("clip"),
                paste_command: ClipboardCommand::new("powershell").with_args([
                    "-noprofile",
                    "-command",
                    "Get-Clipboard",
                ]),
            }),
            other => Err(AppError::clipboard_error(format!(
                "Unsupported platform '{other}' for clipboard operations"
            ))),
        }
    }

    fn detect_linux() -> Result<Self, AppError> {
        if Self::command_available("wl-copy", ["--version"]) {
            return Ok(Self {
                copy_command: ClipboardCommand::new("wl-copy"),
                paste_command: ClipboardCommand::new("wl-paste"),
            });
        }

        if Self::command_available("xclip", ["-version"]) {
            return Ok(Self {
                copy_command: ClipboardCommand::new("xclip").with_args(["-selection", "clipboard"]),
                paste_command: ClipboardCommand::new("xclip").with_args([
                    "-selection",
                    "clipboard",
                    "-o",
                ]),
            });
        }

        Err(AppError::clipboard_error(
            "No supported clipboard command found. Install wl-copy or xclip, or set MX_CLIPBOARD_FILE.",
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
        let mut cmd = Command::new(&self.copy_command.program);
        cmd.args(&self.copy_command.args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let mut child = cmd.spawn().map_err(|err| {
            AppError::clipboard_error(format!(
                "Failed to run clipboard command '{}': {err}",
                self.copy_command.program
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

    fn paste(&self) -> Result<String, AppError> {
        let output = Command::new(&self.paste_command.program)
            .args(&self.paste_command.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| {
                AppError::clipboard_error(format!(
                    "Failed to run paste command '{}': {err}",
                    self.paste_command.program
                ))
            })?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|err| {
                AppError::clipboard_error(format!("Clipboard content is not valid UTF-8: {err}"))
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::clipboard_error(format!(
                "Paste command exited with status {}. Stderr: {}",
                output.status,
                stderr.trim()
            )))
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

    fn paste(&self) -> Result<String, AppError> {
        match fs::read_to_string(&self.path) {
            Ok(content) => Ok(content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(AppError::clipboard_error(e.to_string())),
        }
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

    #[test]
    fn file_clipboard_paste_reads_contents() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("clipboard.txt");
        fs::write(&file, "test content").expect("write test file");
        let clip = FileClipboard::new(file).expect("file clipboard should init");
        let content = clip.paste().expect("paste should succeed");
        assert_eq!(content, "test content");
    }

    #[test]
    fn file_clipboard_roundtrip() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("clipboard.txt");
        let clip = FileClipboard::new(file).expect("file clipboard should init");

        let original = "roundtrip test\nwith newlines";
        clip.copy(original).expect("copy should succeed");
        let retrieved = clip.paste().expect("paste should succeed");
        assert_eq!(retrieved, original);
    }

    #[test]
    fn file_clipboard_paste_nonexistent_returns_empty() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("nonexistent.txt");
        let clip = FileClipboard::new(file).expect("file clipboard should init");
        let content = clip.paste().expect("paste should succeed on nonexistent file");
        assert_eq!(content, "");
    }
}
