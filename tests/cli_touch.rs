use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_touch_creates_context_files() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("touch")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    let mix_dir = temp.path().join(".mix");
    assert!(mix_dir.exists());
    assert!(mix_dir.join(".gitignore").exists());
    assert!(mix_dir.join("tasks.md").exists());
}

#[test]
fn test_touch_nested_files() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("pdt")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    assert!(temp.path().join(".mix/pending/tasks.md").exists());
}

#[test]
fn test_touch_existing_file() {
    let temp = tempdir().unwrap();
    let tasks_md = temp.path().join(".mix/tasks.md");
    fs::create_dir_all(tasks_md.parent().unwrap()).unwrap();
    fs::File::create(&tasks_md).unwrap();

    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("tk")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file found"));
}

#[test]
fn test_list_still_works() {
    let temp = tempdir().unwrap();
    // Setup minimal environment for list
    // We need env var or default path for mix snippets
    // But mix list uses SnippetStorage::new_default() which looks at env var MIX_HOME or defaults.
    // We can set MIX_HOME to our temp dir.

    // Create a dummy snippet
    let snippets_dir = temp.path().join("snippets");
    fs::create_dir_all(&snippets_dir).unwrap();
    fs::File::create(snippets_dir.join("test.md")).unwrap();

    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.env("MIX_HOME", temp.path()).arg("list").assert().success();
}

// === Dynamic Path Resolution Tests ===

#[test]
fn test_touch_dynamic_simple() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("random_name")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"))
        .stdout(predicate::str::contains("random_name.md"));

    assert!(temp.path().join(".mix/random_name.md").exists());
}

#[test]
fn test_touch_dynamic_nested() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("a/b/c")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    assert!(temp.path().join(".mix/a/b/c.md").exists());
    assert!(temp.path().join(".mix/a/b").is_dir());
    assert!(temp.path().join(".mix/a").is_dir());
}

#[test]
fn test_touch_with_extension() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"))
        .stdout(predicate::str::contains("data.json"));

    assert!(temp.path().join(".mix/data.json").exists());
    // Should NOT have .md.json
    assert!(!temp.path().join(".mix/data.json.md").exists());
}

#[test]
fn test_touch_with_txt_extension() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("logs.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    assert!(temp.path().join(".mix/logs.txt").exists());
}

#[test]
fn test_touch_path_traversal_rejected() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("../hack")
        .assert()
        .failure()
        .stderr(predicate::str::contains("outside of .mix"));

    // Ensure no file was created outside .mix
    assert!(!temp.path().join("hack.md").exists());
    assert!(!temp.path().join("hack").exists());
}

#[test]
fn test_touch_path_traversal_embedded_rejected() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("foo/../bar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("outside of .mix"));
}

#[test]
fn test_touch_dynamic_existing_file() {
    let temp = tempdir().unwrap();
    let custom_file = temp.path().join(".mix/custom.md");
    fs::create_dir_all(custom_file.parent().unwrap()).unwrap();
    fs::File::create(&custom_file).unwrap();

    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("custom")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file found"));
}

#[test]
fn test_touch_nested_with_extension() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("mix").unwrap();

    cmd.current_dir(&temp)
        .arg("t")
        .arg("docs/spec.yaml")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Context file created"));

    assert!(temp.path().join(".mix/docs/spec.yaml").exists());
    assert!(temp.path().join(".mix/docs").is_dir());
}
