use mix::copy_snippet;
use serial_test::serial;
use std::io;

#[test]
#[serial]
fn copy_missing_snippet_surfaces_not_found() {
    let err = copy_snippet("missing").expect_err("copy should fail without snippets");
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}
