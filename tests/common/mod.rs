//! Shared testing utilities for the mx CLI.

use assert_cmd::Command;
use std::cell::RefCell;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestContext {
    root: TempDir,
    work_dir: PathBuf,
    original_home: Option<OsString>,
    env_overrides: RefCell<Vec<(String, Option<OsString>)>>,
}

#[allow(dead_code)]
impl TestContext {
    /// Create a new isolated environment and point `HOME` at it.
    pub fn new() -> Self {
        let root = TempDir::new().expect("Failed to create temp directory");
        let work_dir = root.path().join("work");
        fs::create_dir_all(&work_dir).expect("Failed to create test work dir");

        let original_home = env::var_os("HOME");
        env::set_var("HOME", root.path());

        let ctx = Self { root, work_dir, original_home, env_overrides: RefCell::new(Vec::new()) };
        fs::create_dir_all(ctx.commands_root()).expect("Failed to create commands root");
        ctx
    }

    pub fn home(&self) -> &Path {
        self.root.path()
    }

    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    pub fn commands_root(&self) -> PathBuf {
        self.home().join(".config").join("mx").join("commands")
    }

    pub fn config_path(&self) -> PathBuf {
        self.home().join(".config").join("mx").join("config.yml")
    }

    pub fn cli(&self) -> Command {
        self.cli_in(self.work_dir())
    }

    pub fn cli_in<P: AsRef<Path>>(&self, dir: P) -> Command {
        let mut cmd = Command::cargo_bin("mx").expect("Failed to locate mx binary");
        cmd.current_dir(dir.as_ref()).env("HOME", self.home());
        cmd
    }

    pub fn write_snippet(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.commands_root().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create snippet parent");
        }
        fs::write(&path, contents).expect("Failed to write snippet");
        path
    }

    pub fn write_config(&self, contents: &str) {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config parent");
        }
        fs::write(path, contents).expect("Failed to write config file");
    }

    /// Seed a minimal catalog of snippets + config for integration tests.
    pub fn install_sample_catalog(&self) {
        self.write_snippet("w/wc.md", "# /wc\nPlan critically\n");
        self.write_snippet("sdd/sdd-0-rq.md", "Requirements prompt\n");
        self.write_config(
            r#"---
commands:
  wc:
    title: "Work on Tasks"
    description: "Critical planning workflow"
    prompt-file: "commands/w/wc.md"
  sdd-0-rq:
    title: "SDD Step 0"
    description: "Requirements capture"
    prompt-file: "commands/sdd/sdd-0-rq.md"
"#,
        );
    }

    pub fn clipboard_file(&self, name: &str) -> PathBuf {
        let file = self.work_dir().join(name);
        self.set_env("MX_CLIPBOARD_FILE", file.to_string_lossy());
        file
    }

    pub fn codex_dir(&self) -> PathBuf {
        self.home().join(".codex").join("prompts")
    }

    pub fn claude_dir(&self) -> PathBuf {
        self.home().join(".claude").join("commands")
    }

    pub fn gemini_dir(&self) -> PathBuf {
        self.home().join(".gemini").join("commands")
    }

    pub fn set_env<S: AsRef<str>>(&self, key: &str, value: S) {
        self.remember_env(key);
        env::set_var(key, value.as_ref());
    }

    fn remember_env(&self, key: &str) {
        // HOME is already tracked separately via `original_home`.
        if key == "HOME" {
            return;
        }
        let mut overrides = self.env_overrides.borrow_mut();
        if overrides.iter().any(|(existing, _)| existing == key) {
            return;
        }
        overrides.push((key.to_string(), env::var_os(key)));
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if let Some(original) = &self.original_home {
            env::set_var("HOME", original);
        } else {
            env::remove_var("HOME");
        }

        for (key, value) in self.env_overrides.borrow().iter() {
            match value {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
    }
}
