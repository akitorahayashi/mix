mod common;

use common::TestContext;
use mix::{copy_snippet, list_snippets};
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn copy_snippet_via_library_api() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();
    let clipboard = ctx.clipboard_file("api_clipboard.txt");

    let outcome = copy_snippet("wc").expect("copy via API succeeds");
    assert_eq!(outcome.key, "wc");
    assert!(outcome.relative_path.contains("wc"));

    let captured = fs::read_to_string(clipboard).expect("clipboard file exists");
    assert!(captured.contains("/wc"));
}

#[test]
#[serial]
fn list_snippets_works() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    let entries = list_snippets().expect("list via API succeeds");
    assert_eq!(entries.len(), 2);
    // Metadata is no longer returned, so we just check existence
    assert!(entries.iter().any(|entry| entry.key == "wc"));
}
