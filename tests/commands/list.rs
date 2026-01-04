//! Integration tests for the `mx list` command

use super::super::common::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn list_command_prints_snippets() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    ctx.cli()
        .arg("list")
        .assert()
        .success()
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
        .stdout(predicate::str::contains("wc").and(predicate::str::contains("sdd/sdd-0-rq")));
}
