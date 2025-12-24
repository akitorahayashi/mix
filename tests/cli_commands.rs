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
        // Metadata titles removed, just checking for keys/paths
        .stdout(predicate::str::contains("wc").and(predicate::str::contains("sdd/sdd-0-rq")));
}

#[test]
#[serial]
fn list_alias_ls_works() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    ctx.cli()
        .arg("ls")
        .assert()
        .success()
        // Metadata titles removed, just checking for keys/paths
        .stdout(predicate::str::contains("wc").and(predicate::str::contains("sdd/sdd-0-rq")));
}

#[test]
#[serial]
fn copy_command_uses_clipboard_override() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file("clipboard.txt");

    ctx.cli().args(["c", "wc"]).assert().success().stdout(predicate::str::contains("Copied 'wc'"));

    let captured = fs::read_to_string(clipboard).expect("clipboard file should exist");
    assert!(captured.contains("/wc"), "clipboard should hold snippet contents");
}

#[test]
#[serial]
fn copy_missing_snippet_fails() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    // Ensure clipboard is configured so we don't fail on clipboard check
    let _ = ctx.clipboard_file("clipboard_fail.txt");

    ctx.cli()
        .args(["c", "unknown"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No snippet named"));
}

#[test]
#[serial]
fn version_flag_works() {
    let ctx = TestContext::new();

    ctx.cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
#[serial]
fn clean_alias_cl_works() {
    let ctx = TestContext::new();

    // Create a context file
    ctx.cli().current_dir(ctx.work_dir()).arg("touch").arg("tk").assert().success();

    // Clean with alias
    ctx.cli()
        .current_dir(ctx.work_dir())
        .arg("cl")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));
}
