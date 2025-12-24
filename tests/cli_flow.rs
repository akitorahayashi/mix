mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn user_can_list_copy_and_touch() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    // 1. List
    ctx.cli().arg("list").assert().success().stdout(predicate::str::contains("wc"));

    // 2. Copy
    let clipboard = ctx.clipboard_file("flow.txt");
    ctx.cli().args(["c", "wc"]).assert().success();
    let clip = fs::read_to_string(clipboard).expect("clipboard file available");
    assert!(clip.contains("/wc"));

    // 3. Touch
    ctx.cli().args(["touch", "tk"]).assert().success().stdout(predicate::str::contains("created"));
    let tasks_md = ctx.work_dir().join(".mix/tasks.md");
    assert!(tasks_md.exists());

    // 4. Touch again (idempotent)
    ctx.cli().args(["touch", "tk"]).assert().success().stdout(predicate::str::contains("found"));
}
