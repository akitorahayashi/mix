//! Integration tests for the `mx touch` command

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Helper to setup clipboard file for tests
fn setup_clipboard(temp: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
    let clipboard_file = temp.path().join("clipboard.txt");
    fs::write(&clipboard_file, content).unwrap();
    clipboard_file
}

#[test]
fn touch_creates_context_files() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&temp, "test content");

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("touch")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    let mix_dir = temp.path().join("mix");
    assert!(mix_dir.exists());
    assert!(mix_dir.join(".gitignore").exists());
    assert!(mix_dir.join("tasks.md").exists());
}

#[test]
fn touch_nested_files() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&temp, "nested content");

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("pdt")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    assert!(temp.path().join("mix/pending/tasks.md").exists());
}

#[test]
fn touch_existing_file_shows_warning() {
    let temp = tempdir().unwrap();
    let tasks_md = temp.path().join("mix/tasks.md");
    fs::create_dir_all(tasks_md.parent().unwrap()).unwrap();
    fs::File::create(&tasks_md).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("t")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::str::contains("⚠️ Context file already exists"));
}

#[test]
fn touch_force_overwrites() {
    let temp = tempdir().unwrap();
    let tasks_md = temp.path().join("mix/tasks.md");
    fs::create_dir_all(tasks_md.parent().unwrap()).unwrap();
    fs::write(&tasks_md, "original content").unwrap();

    let clipboard_content = "new clipboard content";
    let clipboard_file = setup_clipboard(&temp, clipboard_content);

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("tk")
        .arg("-f")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file overwritten"));

    let content = fs::read_to_string(&tasks_md).unwrap();
    assert_eq!(content, clipboard_content);
}

#[test]
fn touch_without_force_preserves_existing() {
    let temp = tempdir().unwrap();
    let tasks_md = temp.path().join("mix/tasks.md");
    fs::create_dir_all(tasks_md.parent().unwrap()).unwrap();
    fs::write(&tasks_md, "original content").unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("t")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::str::contains("⚠️ Context file already exists"));

    let content = fs::read_to_string(&tasks_md).unwrap();
    assert_eq!(content, "original content");
}

#[test]
fn touch_dynamic_simple() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&temp, "dynamic content");

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("random_name")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"))
        .stdout(predicate::str::contains("random_name.md"));

    assert!(temp.path().join("mix/random_name.md").exists());
}

#[test]
fn touch_dynamic_nested() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&temp, "nested dynamic");

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("a/b/c")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    assert!(temp.path().join("mix/a/b/c.md").exists());
    assert!(temp.path().join("mix/a/b").is_dir());
    assert!(temp.path().join("mix/a").is_dir());
}

#[test]
fn touch_with_extension() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&temp, r#"{"key": "value"}"#);

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"))
        .stdout(predicate::str::contains("data.json"));

    assert!(temp.path().join("mix/data.json").exists());
    assert!(!temp.path().join("mix/data.json.md").exists());
}

#[test]
fn touch_path_traversal_rejected() {
    let temp = tempdir().unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("t")
        .arg("../hack")
        .assert()
        .failure()
        .stderr(predicate::str::contains("outside of mix"));

    assert!(!temp.path().join("hack.md").exists());
    assert!(!temp.path().join("hack").exists());
}

#[test]
fn touch_path_traversal_embedded_rejected() {
    let temp = tempdir().unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("t")
        .arg("foo/../bar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("outside of mix"));
}

#[test]
fn touch_pending_prefix() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&temp, "pending content");

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("pd-testdoc")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    assert!(temp.path().join("mix/pending/testdoc.md").exists());
}

#[test]
fn touch_pastes_clipboard_by_default() {
    let temp = tempdir().unwrap();
    let clipboard_content = "This is the clipboard content\nWith multiple lines";
    let clipboard_file = setup_clipboard(&temp, clipboard_content);

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    let tasks_file = temp.path().join("mix/tasks.md");
    assert!(tasks_file.exists());
    let content = fs::read_to_string(&tasks_file).unwrap();
    assert_eq!(content, clipboard_content);
}

#[test]
fn touch_with_numbered_alias() {
    let temp = tempdir().unwrap();
    let clipboard_content = "Numbered alias test";
    let clipboard_file = setup_clipboard(&temp, clipboard_content);

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("tk1")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    let numbered_file = temp.path().join("mix/tasks/tasks1.md");
    assert!(numbered_file.exists());
    let content = fs::read_to_string(&numbered_file).unwrap();
    assert_eq!(content, clipboard_content);
}

#[test]
fn touch_empty_clipboard() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&temp, "");

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("empty")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    let empty_file = temp.path().join("mix/empty.md");
    assert!(empty_file.exists());
    let content = fs::read_to_string(&empty_file).unwrap();
    assert_eq!(content, "");
}
