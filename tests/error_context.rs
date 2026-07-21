// `YangError` must identify what failed and where.
//
// Loading used to report "YANG file parse error" with no indication of
// which file, having printed the real parol diagnostic to stdout and
// then dropped it. These tests pin the replacement: the error names the
// file or module, and the parse diagnostic survives on the error chain
// for callers that want the position of the offending token.

use libyang::{YangError, YangStore};
use std::error::Error;

const YANG_DIR: &str = "tests/yang";

fn load_err(name: &str) -> YangError {
    let mut store = YangStore::new();
    store.add_path(YANG_DIR);
    store
        .read_with_resolve(name)
        .expect_err("module is expected to fail loading")
}

#[test]
fn missing_module_names_the_module() {
    let err = load_err("no-such-module");
    match &err {
        YangError::FileNotFound { name } => assert_eq!(name, "no-such-module"),
        other => panic!("expected FileNotFound, got {other:?}"),
    }
    // The name has to reach the rendered message, not just the struct —
    // most callers only ever print the error.
    assert!(
        err.to_string().contains("no-such-module"),
        "message should name the module: {err}"
    );
}

#[test]
fn parse_failure_names_the_file() {
    // tests/yang/malformed.yang has a `type` statement with no argument.
    let err = load_err("malformed");
    match &err {
        YangError::ParseError { path, .. } => {
            assert!(
                path.ends_with("malformed.yang"),
                "expected the offending path, got {path:?}"
            );
        }
        other => panic!("expected ParseError, got {other:?}"),
    }
    assert!(
        err.to_string().contains("malformed.yang"),
        "message should name the file: {err}"
    );
}

#[test]
fn parse_failure_keeps_the_parol_diagnostic() {
    let err = load_err("malformed");

    // The diagnostic is reachable as the error's source ...
    let source = err.source().expect("parse error should have a source");

    // ... and still carries the position of the offending token, which
    // is the part that makes the failure actionable. The `type`
    // statement is on line 7, so the parser trips at the following
    // line; assert on the file:line shape rather than an exact number.
    let detail = format!("{source:?}");
    assert!(
        detail.contains("start_line: 8"),
        "expected token position in the preserved diagnostic: {detail}"
    );
    assert!(
        detail.contains("malformed.yang"),
        "expected the file name in the preserved diagnostic: {detail}"
    );
}

#[test]
fn a_healthy_module_still_loads() {
    // Guard against the error plumbing accidentally rejecting good input.
    let mut store = YangStore::new();
    store.add_path(YANG_DIR);
    store
        .read_with_resolve("augment-order-target")
        .expect("valid module loads");
    assert!(store.find_module("augment-order-target").is_some());
}
