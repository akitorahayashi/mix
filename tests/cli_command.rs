mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

fn test_command_variant(args: &[&str], clipboard_name: &str) {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file(clipboard_name);

    ctx.cli().args(args).assert().success().stdout(predicate::str::contains("Copied 'wc'"));

    let captured = fs::read_to_string(&clipboard).expect("clipboard file should exist");
    assert!(captured.contains("/wc"), "clipboard should hold snippet contents");
}

#[test]
#[serial]
fn command_subcommand_works() {
    test_command_variant(&["command", "wc"], "clipboard.txt");
}

#[test]
#[serial]
fn command_alias_c_works() {
    test_command_variant(&["c", "wc"], "clipboard_alias.txt");
}

#[test]
#[serial]
fn command_subcommand_missing_snippet_fails() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let _ = ctx.clipboard_file("clipboard_fail.txt");

    // Test `mix command unknown`
    ctx.cli()
        .arg("command")
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No snippet named"));
}
