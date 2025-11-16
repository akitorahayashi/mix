mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn list_command_prints_snippets() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    ctx.cli()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("wc").and(predicate::str::contains("SDD Step 0")));
}

#[test]
#[serial]
fn copy_command_uses_clipboard_override() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file("clipboard.txt");

    ctx.cli().arg("wc").assert().success().stdout(predicate::str::contains("Copied 'wc'"));

    let captured = fs::read_to_string(clipboard).expect("clipboard file should exist");
    assert!(captured.contains("/wc"), "clipboard should hold snippet contents");
}

#[test]
#[serial]
fn slash_command_generates_codex_files() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    ctx.cli().args(["slash", "codex"]).assert().success().stdout(predicate::str::contains("codex"));

    let codex_prompt = ctx.codex_dir().join("wc.md");
    assert!(codex_prompt.exists(), "codex prompt should be generated");
    let content = fs::read_to_string(codex_prompt).expect("codex prompt readable");
    assert!(content.contains("Plan critically"));
}

#[test]
#[serial]
fn slash_all_generates_every_target() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    ctx.cli().args(["slash", "all"]).assert().success();

    assert!(ctx.codex_dir().join("wc.md").exists());
    assert!(ctx.claude_dir().join("wc.md").exists());
    assert!(ctx.gemini_dir().join("wc.toml").exists());
}

#[test]
#[serial]
fn copy_missing_snippet_fails() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    ctx.cli()
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No snippet named"));
}
