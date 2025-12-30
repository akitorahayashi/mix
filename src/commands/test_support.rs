use crate::commands::clipboard::Clipboard;
use crate::error::AppError;
use crate::storage::SnippetStorage;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub(crate) struct TestSnippetStorage {
    root: TempDir,
    pub storage: SnippetStorage,
}

impl TestSnippetStorage {
    pub fn new() -> Self {
        let root = TempDir::new().expect("tempdir for storage");
        let storage = SnippetStorage::from_root(root.path()).expect("storage init should succeed");
        Self { root, storage }
    }

    pub fn write_snippet<P: AsRef<Path>>(&self, relative_path: P, contents: &str) -> PathBuf {
        let absolute = self.root.path().join(relative_path.as_ref());
        if let Some(parent) = absolute.parent() {
            fs::create_dir_all(parent).expect("create snippet parent");
        }
        fs::write(&absolute, contents).expect("write snippet");
        absolute
    }

    pub fn write_config(&self, contents: &str) {
        let path = self.root.path().join("config.yml");
        fs::write(path, contents).expect("write config");
    }
}

struct RecordingClipboard {
    buffer: RefCell<String>,
}

impl Default for RecordingClipboard {
    fn default() -> Self {
        Self { buffer: RefCell::new(String::new()) }
    }
}

impl Clipboard for RecordingClipboard {
    fn copy(&self, text: &str) -> Result<(), AppError> {
        self.buffer.replace(text.to_string());
        Ok(())
    }

    fn paste(&self) -> Result<String, AppError> {
        Ok(self.buffer.borrow().clone())
    }
}

pub(crate) struct RecordingClipboardHandle {
    clipboard: RecordingClipboard,
}

impl RecordingClipboardHandle {
    pub fn contents(&self) -> String {
        self.clipboard.buffer.borrow().clone()
    }

    #[allow(dead_code)]
    pub fn set_contents(&self, content: &str) {
        self.clipboard.buffer.replace(content.to_string());
    }

    pub fn as_ref(&self) -> &dyn Clipboard {
        &self.clipboard
    }
}

pub(crate) fn recording_clipboard() -> RecordingClipboardHandle {
    RecordingClipboardHandle { clipboard: RecordingClipboard::default() }
}
