use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use tempfile::tempdir;

#[test]
#[serial]
fn test_clean_full_directory() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    // Create some files
    cmd.current_dir(&dir)
        .arg("touch")
        .arg("tk")
        .assert()
        .success();

    assert!(dir.path().join(".mix").exists());
    assert!(dir.path().join(".mix/tasks.md").exists());

    // Clean all
    let mut cmd = Command::cargo_bin("mix").unwrap();
    cmd.current_dir(&dir)
        .arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed .mix directory"));

    assert!(!dir.path().join(".mix").exists());
}

#[test]
#[serial]
fn test_clean_specific_file() {
    let dir = tempdir().unwrap();

    // Create tk (tasks.md) and rq (requirements.md)
    Command::cargo_bin("mix").unwrap()
        .current_dir(&dir)
        .args(&["touch", "tk"])
        .assert()
        .success();

    Command::cargo_bin("mix").unwrap()
        .current_dir(&dir)
        .args(&["touch", "rq"])
        .assert()
        .success();

    // Clean only tk
    Command::cargo_bin("mix").unwrap()
        .current_dir(&dir)
        .args(&["clean", "tk"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));

    assert!(!dir.path().join(".mix/tasks.md").exists());
    assert!(dir.path().join(".mix/requirements.md").exists());
}

#[test]
#[serial]
fn test_clean_nested_and_dynamic() {
    let dir = tempdir().unwrap();

    // Create tk1 (tasks/tasks1.md)
    Command::cargo_bin("mix").unwrap()
        .current_dir(&dir)
        .args(&["touch", "tk1"])
        .assert()
        .success();

    assert!(dir.path().join(".mix/tasks/tasks1.md").exists());

    // Clean tk1
    Command::cargo_bin("mix").unwrap()
        .current_dir(&dir)
        .args(&["clean", "tk1"])
        .assert()
        .success();

    assert!(!dir.path().join(".mix/tasks/tasks1.md").exists());
    // The parent 'tasks' directory should also be removed if empty (optional feature, but implemented)
    assert!(!dir.path().join(".mix/tasks").exists());
}

#[test]
#[serial]
fn test_clean_nonexistent_file() {
    let dir = tempdir().unwrap();

    // Ensure .mix exists
    fs::create_dir(dir.path().join(".mix")).unwrap();

    Command::cargo_bin("mix").unwrap()
        .current_dir(&dir)
        .args(&["clean", "tk"])
        .assert()
        .failure() // Should fail or print error
        .stderr(predicate::str::contains("File not found"));
}
