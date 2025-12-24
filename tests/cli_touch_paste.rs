use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Helper to create a clipboard file with content
fn setup_clipboard_file(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
    let clipboard_file = dir.join("clipboard.txt");
    fs::write(&clipboard_file, content).expect("write clipboard file");
    clipboard_file
}

/// Helper to run mix touch with paste flag and verify success
fn run_touch_paste(
    temp_dir: &std::path::Path,
    clipboard_file: &std::path::Path,
    key: &str,
    flag: &str,
) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("mix").unwrap();
    cmd.current_dir(temp_dir)
        .env("MIX_CLIPBOARD_FILE", clipboard_file)
        .arg("t")
        .arg(key)
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"))
}

/// Helper to verify file content matches expected
fn verify_file_content(file_path: &std::path::Path, expected_content: &str) {
    assert!(file_path.exists());
    let content = fs::read_to_string(file_path).expect("read file");
    assert_eq!(content, expected_content);
}

#[test]
fn test_touch_paste_creates_file_with_clipboard_content() {
    let temp = tempdir().unwrap();
    let clipboard_content = "This is the clipboard content\nWith multiple lines";
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    run_touch_paste(temp.path(), &clipboard_file, "tk", "--paste");

    let tasks_file = temp.path().join(".mix/tasks.md");
    verify_file_content(&tasks_file, clipboard_content);
}

#[test]
fn test_touch_paste_short_flag() {
    let temp = tempdir().unwrap();
    let clipboard_content = "Short flag test";
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    run_touch_paste(temp.path(), &clipboard_file, "rq", "-p");

    let requirements_file = temp.path().join(".mix/requirements.md");
    verify_file_content(&requirements_file, clipboard_content);
}

#[test]
fn test_touch_paste_does_not_overwrite_existing_file() {
    let temp = tempdir().unwrap();
    let original_content = "Original content that should not be overwritten";
    let clipboard_content = "New clipboard content";

    // Create existing file
    let tasks_file = temp.path().join(".mix/tasks.md");
    fs::create_dir_all(tasks_file.parent().unwrap()).unwrap();
    fs::write(&tasks_file, original_content).unwrap();

    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    let mut cmd = Command::cargo_bin("mix").unwrap();
    cmd.current_dir(&temp)
        .env("MIX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("tk")
        .arg("--paste")
        .assert()
        .success()
        .stdout(predicate::str::contains("⚠️ Context file already exists"));

    // File should still have original content
    let content = fs::read_to_string(&tasks_file).expect("read tasks file");
    assert_eq!(content, original_content);
}

#[test]
fn test_touch_paste_with_dynamic_path() {
    let temp = tempdir().unwrap();
    let clipboard_content = "Dynamic path content";
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    run_touch_paste(temp.path(), &clipboard_file, "docs/spec", "-p");

    let spec_file = temp.path().join(".mix/docs/spec.md");
    verify_file_content(&spec_file, clipboard_content);
}

#[test]
fn test_touch_paste_with_nested_alias() {
    let temp = tempdir().unwrap();
    let clipboard_content = "Nested alias content";
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    run_touch_paste(temp.path(), &clipboard_file, "pdt", "--paste");

    let pending_tasks = temp.path().join(".mix/pending/tasks.md");
    verify_file_content(&pending_tasks, clipboard_content);
}

#[test]
fn test_touch_paste_with_different_extension() {
    let temp = tempdir().unwrap();
    let clipboard_content = r#"{"key": "value", "number": 42}"#;
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    run_touch_paste(temp.path(), &clipboard_file, "config.json", "-p");

    let config_file = temp.path().join(".mix/config.json");
    verify_file_content(&config_file, clipboard_content);
}

#[test]
fn test_touch_paste_empty_clipboard() {
    let temp = tempdir().unwrap();
    let clipboard_file = setup_clipboard_file(temp.path(), "");

    run_touch_paste(temp.path(), &clipboard_file, "empty", "-p");

    let empty_file = temp.path().join(".mix/empty.md");
    verify_file_content(&empty_file, "");
}

#[test]
fn test_touch_without_paste_flag_does_not_write_clipboard() {
    let temp = tempdir().unwrap();
    let clipboard_content = "This should not be written";
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    let mut cmd = Command::cargo_bin("mix").unwrap();
    cmd.current_dir(&temp)
        .env("MIX_CLIPBOARD_FILE", &clipboard_file)
        .arg("t")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    let tasks_file = temp.path().join(".mix/tasks.md");
    assert!(tasks_file.exists());
    let content = fs::read_to_string(&tasks_file).expect("read tasks file");
    // File should exist but be empty (just created by touch)
    assert_eq!(content, "");
}

#[test]
fn test_touch_paste_with_numbered_alias() {
    let temp = tempdir().unwrap();
    let clipboard_content = "Numbered alias test";
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    run_touch_paste(temp.path(), &clipboard_file, "tk1", "-p");

    let numbered_file = temp.path().join(".mix/tasks/tasks1.md");
    verify_file_content(&numbered_file, clipboard_content);
}

#[test]
fn test_touch_paste_multiline_content() {
    let temp = tempdir().unwrap();
    let clipboard_content = "Line 1\nLine 2\nLine 3\n\nLine 5 with blank line before";
    let clipboard_file = setup_clipboard_file(temp.path(), clipboard_content);

    run_touch_paste(temp.path(), &clipboard_file, "multiline", "-p");

    let multiline_file = temp.path().join(".mix/multiline.md");
    verify_file_content(&multiline_file, clipboard_content);
}
