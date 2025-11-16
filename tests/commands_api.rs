mod common;

use common::TestContext;
use mix::{copy_snippet, generate_slash_commands, list_snippets, SlashRequest, SlashTarget};
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
fn list_snippets_returns_metadata() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    let entries = list_snippets().expect("list via API succeeds");
    assert_eq!(entries.len(), 2);
    assert!(entries
        .iter()
        .any(|entry| entry.key == "wc" && entry.title.as_deref() == Some("Work on Tasks")));
}

#[test]
#[serial]
fn generate_slash_commands_for_claude() {
    let ctx = TestContext::new();
    ctx.install_sample_catalog();

    let artifacts = generate_slash_commands(SlashRequest::Only(SlashTarget::Claude))
        .expect("slash generation succeeds");
    assert!(!artifacts.is_empty());

    let claude_prompt = ctx.claude_dir().join("wc.md");
    assert!(claude_prompt.exists(), "Claude prompt should be written");
    let content = fs::read_to_string(claude_prompt).expect("Claude prompt readable");
    assert!(content.contains("title:"));
}
