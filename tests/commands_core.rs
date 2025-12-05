use mix::copy_snippet;
use serial_test::serial;
use std::io;

#[test]
#[serial]
fn copy_missing_snippet_surfaces_not_found() {
    // We expect an error. If we are running in an environment without clipboard tools,
    // we might get ClipboardError (ErrorKind::Other) before we get NotFound.
    // However, the intent of this test seems to be verifying NotFound for missing snippets.
    // To reliably test this, we should mock clipboard or accept that the error might differ if clipboard fails first.

    // In CI/Sandbox without xclip/wl-copy, `copy_snippet` calls `clipboard_from_env()`.
    // If that fails, we get AppError::ClipboardError -> io::ErrorKind::Other.

    // So we check if it is either NotFound or Other (clipboard failure).
    // Or better, we can assume that if we get "Other", it's the clipboard issue, which is acceptable here since we can't easily mock env in this test without setting up the file.

    // Actually, `mix::copy_snippet` uses `clipboard_from_env`. We can set `MIX_CLIPBOARD_FILE` to a temp file to pass the clipboard check.

    let temp_dir = std::env::temp_dir();
    let clip_file = temp_dir.join("test_clipboard_commands_core.txt");
    std::env::set_var("MIX_CLIPBOARD_FILE", &clip_file);

    let err = copy_snippet("missing").expect_err("copy should fail without snippets");
    assert_eq!(err.kind(), io::ErrorKind::NotFound);

    std::env::remove_var("MIX_CLIPBOARD_FILE");
}
