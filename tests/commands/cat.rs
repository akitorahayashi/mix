//! Integration tests for the `mx cat` command

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn cat_displays_file_contents() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(&mx_dir).unwrap();

    let expected_content = "# Tasks\n\n- Task 1\n- Task 2\n";
    fs::write(mx_dir.join("tasks.md"), expected_content).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::eq(expected_content));
}

#[test]
fn cat_alias_ct_works() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(&mx_dir).unwrap();

    let content = "Requirements document";
    fs::write(mx_dir.join("requirements.md"), content).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("ct")
        .arg("rq")
        .assert()
        .success()
        .stdout(predicate::eq(content));
}

#[test]
fn cat_file_not_found_shows_error() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".mx")).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("⚠️"))
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn cat_nested_alias_works() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(mx_dir.join("pending")).unwrap();

    let content = "Pending tasks content";
    fs::write(mx_dir.join("pending/tasks.md"), content).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("pdt")
        .assert()
        .success()
        .stdout(predicate::eq(content));
}

#[test]
fn cat_dynamic_path_works() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(mx_dir.join("docs")).unwrap();

    let content = "Specification document";
    fs::write(mx_dir.join("docs/spec.md"), content).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("docs/spec")
        .assert()
        .success()
        .stdout(predicate::eq(content));
}

#[test]
fn cat_numbered_alias_works() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(mx_dir.join("tasks")).unwrap();

    let content = "Tasks 1 content";
    fs::write(mx_dir.join("tasks/tasks1.md"), content).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("tk1")
        .assert()
        .success()
        .stdout(predicate::eq(content));
}

#[test]
fn cat_pending_prefix_works() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(mx_dir.join("pending")).unwrap();

    let content = "Pending tasks via pd- prefix";
    fs::write(mx_dir.join("pending/tasks.md"), content).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("pd-tk")
        .assert()
        .success()
        .stdout(predicate::eq(content));
}

#[test]
fn cat_rejects_path_traversal() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".mx")).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("../etc/passwd")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid path"));
}

#[test]
fn cat_empty_file_succeeds() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(&mx_dir).unwrap();

    fs::write(mx_dir.join("empty.md"), "").unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("empty")
        .assert()
        .success()
        .stdout(predicate::eq(""));
}

#[test]
fn cat_directory_shows_error() {
    let temp = tempdir().unwrap();
    let mx_dir = temp.path().join(".mx");
    fs::create_dir_all(mx_dir.join("somedir")).unwrap();

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("somedir")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a file"));
}

#[test]
fn cat_with_touch_integration() {
    let temp = tempdir().unwrap();
    let clipboard_file = temp.path().join("clipboard.txt");
    let content = "Content from clipboard";
    fs::write(&clipboard_file, content).unwrap();

    // First, create a file with touch
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .env("MX_CLIPBOARD_FILE", &clipboard_file)
        .arg("touch")
        .arg("tk")
        .assert()
        .success();

    // Then read it back with cat
    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::eq(content));
}

#[test]
fn cat_without_mx_directory_shows_error() {
    let temp = tempdir().unwrap();
    // Don't create .mx directory

    Command::cargo_bin("mx")
        .unwrap()
        .current_dir(&temp)
        .arg("cat")
        .arg("tk")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
