//! Integration tests for the `mx command` (copy snippet) command

use super::super::common::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn command_subcommand_works() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file("clipboard.txt");

    ctx.cli()
        .args(["command", "wc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Copied 'wc'"));

    let captured = fs::read_to_string(&clipboard).expect("clipboard file should exist");
    assert!(captured.contains("/wc"), "clipboard should hold snippet contents");
}

#[test]
#[serial]
fn command_alias_c_works() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file("clipboard_alias.txt");

    ctx.cli()
        .args(["c", "wc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Copied 'wc'"));

    let captured = fs::read_to_string(&clipboard).expect("clipboard file should exist");
    assert!(captured.contains("/wc"), "clipboard should hold snippet contents");
}

#[test]
#[serial]
fn command_subcommand_missing_snippet_fails() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let _ = ctx.clipboard_file("clipboard_fail.txt");

    ctx.cli()
        .arg("command")
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No snippet named"));
}
