mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn copy_subcommand_works() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file("clipboard.txt");

    // Test `mix copy wc`
    ctx.cli()
        .arg("copy")
        .arg("wc")
        .assert()
        .success()
        .stdout(predicate::str::contains("Copied 'wc'"));

    let captured = fs::read_to_string(&clipboard).expect("clipboard file should exist");
    assert!(captured.contains("/wc"), "clipboard should hold snippet contents");
}

#[test]
#[serial]
fn copy_alias_c_works() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file("clipboard_alias.txt");

    // Test `mix c wc`
    ctx.cli().arg("c").arg("wc").assert().success().stdout(predicate::str::contains("Copied 'wc'"));

    let captured = fs::read_to_string(&clipboard).expect("clipboard file should exist");
    assert!(captured.contains("/wc"), "clipboard should hold snippet contents");
}

#[test]
#[serial]
fn copy_subcommand_missing_snippet_fails() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let _ = ctx.clipboard_file("clipboard_fail.txt");

    // Test `mix copy unknown`
    ctx.cli()
        .arg("copy")
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No snippet named"));
}
