mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn user_can_list_copy_and_generate() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    ctx.cli().arg("list").assert().success().stdout(predicate::str::contains("wc"));

    let clipboard = ctx.clipboard_file("flow.txt");
    ctx.cli().arg("wc").assert().success();
    let clip = fs::read_to_string(clipboard).expect("clipboard file available");
    assert!(clip.contains("/wc"));

    ctx.cli().args(["slash", "gemini"]).assert().success();
    let toml = ctx.gemini_dir().join("wc.toml");
    let toml_body = fs::read_to_string(toml).expect("gemini file exists");
    assert!(toml_body.contains("description"));
}
