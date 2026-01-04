//! Integration tests for the `mx clean` command

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use tempfile::tempdir;

/// Helper to setup clipboard file for tests
fn setup_clipboard(temp: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
    let clipboard_file = temp.path().join("clipboard.txt");
    fs::write(&clipboard_file, content).unwrap();
    clipboard_file
}

#[test]
#[serial]
fn clean_full_directory() {
    let dir = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&dir, "test content");

    // Create some files
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("touch")
        .arg("tk")
        .assert()
        .success();

    assert!(dir.path().join("mx").exists());
    assert!(dir.path().join("mx/tasks.md").exists());

    // Clean all
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed mx directory"));

    assert!(!dir.path().join("mx").exists());
}

#[test]
#[serial]
fn clean_specific_file() {
    let dir = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&dir, "test content");

    // Create tk (tasks.md) and rq (requirements.md)
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .args(["touch", "tk"])
        .assert()
        .success();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .args(["touch", "rq"])
        .assert()
        .success();

    // Clean only tk
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .args(["clean", "tk"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));

    assert!(!dir.path().join("mx/tasks.md").exists());
    assert!(dir.path().join("mx/requirements.md").exists());
}

#[test]
#[serial]
fn clean_nested_and_dynamic() {
    let dir = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&dir, "test content");

    // Create tk1 (tasks/tasks1.md)
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .args(["touch", "tk1"])
        .assert()
        .success();

    assert!(dir.path().join("mx/tasks/tasks1.md").exists());

    // Clean tk1
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .args(["clean", "tk1"])
        .assert()
        .success();

    assert!(!dir.path().join("mx/tasks/tasks1.md").exists());
    // The parent 'tasks' directory should also be removed if empty
    assert!(!dir.path().join("mx/tasks").exists());
}

#[test]
#[serial]
fn clean_nonexistent_file() {
    let dir = tempdir().unwrap();

    // Ensure mx exists
    fs::create_dir(dir.path().join("mx")).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .args(["clean", "tk"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
#[serial]
fn clean_alias_cl_works() {
    let dir = tempdir().unwrap();
    let clipboard_file = setup_clipboard(&dir, "test content");

    // Create a context file
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .args(["touch", "tk"])
        .assert()
        .success();

    // Clean with alias
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&dir)
        .arg("cl")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));
}
