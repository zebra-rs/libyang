// RFC 7950 §6.1.3 — unquoted-string acceptance.
//
// `units`, `description`, `organization`, `contact`, `reference` all
// take a YANG string argument. The spec allows either the quoted or
// the unquoted form. Before this test landed, libyang's grammar
// required quotes for everything that went through `Ystring`, which
// rejected RFC-valid modules. The fixture writes every relevant
// statement in the bare form; if the parser comes back without
// errors, the unquoted form is accepted.

use libyang::YangStore;

#[test]
fn unquoted_string_arguments_accepted() {
    let mut store = YangStore::new();
    store.add_path("tests/yang");
    store
        .read_with_resolve("unquoted-strings")
        .expect("unquoted-string YANG must parse");
    store.identity_resolve();
    let module = store
        .find_module("unquoted-strings")
        .expect("module registered");
    assert_eq!(module.name, "unquoted-strings");
}
