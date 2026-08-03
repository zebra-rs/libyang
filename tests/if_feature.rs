// Integration tests for YANG 1.1 feature / if-feature support.
//
// tests/yang/iffeature-base.yang defines features `iso`, `extra` and
// `dep` (which depends on `iso`), imports `remote` from
// iffeature-features, and guards a leaf, a container, a uses, a choice
// case and an augment with them. A feature is off unless
// `YangStore::enable_feature` marks it supported, so the guarded nodes
// are pruned from the entry tree by default and appear once the right
// features are enabled.

use libyang::{Diagnostic, Entry, YangStore, to_entry};
use std::rc::Rc;

fn load(name: &str, features: &[(&str, &str)]) -> (Rc<Entry>, Vec<Diagnostic>) {
    let mut store = YangStore::new();
    store.add_path("tests/yang");
    for (module, feature) in features {
        store.enable_feature(module, feature);
    }
    store.read_with_resolve(name).expect("parse / resolve");
    store.identity_resolve();
    let module = store.find_module(name).expect("module found");
    let entry = to_entry(&store, module);
    let diagnostics = store.take_diagnostics();
    (entry, diagnostics)
}

fn find_child(ent: &Rc<Entry>, name: &str) -> Option<Rc<Entry>> {
    ent.dir.borrow().iter().find(|e| e.name == name).cloned()
}

fn has_child(ent: &Rc<Entry>, name: &str) -> bool {
    find_child(ent, name).is_some()
}

#[test]
fn all_features_off_prunes_guarded_nodes() {
    let (root, diagnostics) = load("iffeature-base", &[]);
    let routing = find_child(&root, "routing").expect("routing container");

    // Unguarded nodes stay: the plain leaf and the ungated choice
    // case's flattened child.
    assert!(has_child(&routing, "name"));
    assert!(has_child(&routing, "tcp-port"));

    // Every guarded node is pruned.
    for gated in [
        "dhcp",
        "iso-only",
        "both",
        "gated-dep",
        "remote-gated",
        "shared-leaf",
        "tsel",
    ] {
        assert!(
            !has_child(&routing, gated),
            "feature-gated node \"{gated}\" should be pruned by default"
        );
    }

    // The gated augment targets the gated (absent) container; being
    // disabled itself, it must vanish silently rather than report a
    // target-not-found diagnostic.
    assert!(
        diagnostics.is_empty(),
        "no diagnostics expected, got {diagnostics:?}"
    );
}

#[test]
fn enable_feature_exposes_guarded_nodes() {
    let (root, diagnostics) = load("iffeature-base", &[("iffeature-base", "iso")]);
    let routing = find_child(&root, "routing").expect("routing container");

    assert!(has_child(&routing, "dhcp"));
    assert!(has_child(&routing, "shared-leaf"), "uses gated by iso");

    // The gated container is back, complete with the gated augment's
    // contribution.
    let iso_only = find_child(&routing, "iso-only").expect("iso-only container");
    assert!(has_child(&iso_only, "nsap"));
    assert!(has_child(&iso_only, "net"), "augment gated by iso applies");

    // The osi choice case is back and carries its choice/case tags.
    let tsel = find_child(&routing, "tsel").expect("tsel leaf from osi case");
    assert_eq!(tsel.choice.borrow().as_deref(), Some("transport"));
    assert_eq!(tsel.case.borrow().as_deref(), Some("osi"));

    // Still gated by features that remain off.
    assert!(!has_child(&routing, "both"), "extra is still off");
    assert!(!has_child(&routing, "gated-dep"), "dep is still off");
    assert!(!has_child(&routing, "remote-gated"));

    assert!(
        diagnostics.is_empty(),
        "no diagnostics expected, got {diagnostics:?}"
    );
}

#[test]
fn compound_expression_needs_both_features() {
    let (root, _) = load(
        "iffeature-base",
        &[("iffeature-base", "iso"), ("iffeature-base", "extra")],
    );
    let routing = find_child(&root, "routing").expect("routing container");
    assert!(has_child(&routing, "both"), "if-feature \"iso and extra\"");
}

#[test]
fn dependent_feature_requires_its_own_if_feature() {
    // `dep` declares `if-feature iso`, so enabling it alone is not
    // enough (RFC 7950 section 7.20.1).
    let (root, _) = load("iffeature-base", &[("iffeature-base", "dep")]);
    let routing = find_child(&root, "routing").expect("routing container");
    assert!(
        !has_child(&routing, "gated-dep"),
        "dep without iso must stay disabled"
    );

    let (root, _) = load(
        "iffeature-base",
        &[("iffeature-base", "dep"), ("iffeature-base", "iso")],
    );
    let routing = find_child(&root, "routing").expect("routing container");
    assert!(has_child(&routing, "gated-dep"));
}

#[test]
fn prefixed_reference_resolves_to_imported_module() {
    // `remote-gated` is guarded by `ifx:remote`, defined in the
    // imported iffeature-features module — enabling it there turns the
    // leaf on; enabling a same-named feature in the wrong module does
    // not.
    let (root, _) = load("iffeature-base", &[("iffeature-features", "remote")]);
    let routing = find_child(&root, "routing").expect("routing container");
    assert!(has_child(&routing, "remote-gated"));

    let (root, _) = load("iffeature-base", &[("iffeature-base", "remote")]);
    let routing = find_child(&root, "routing").expect("routing container");
    assert!(!has_child(&routing, "remote-gated"));
}

#[test]
fn unknown_feature_reference_is_diagnosed_and_disabled() {
    let (root, diagnostics) = load("iffeature-unknown", &[]);
    let top_box = find_child(&root, "top-box").expect("top-box container");

    assert!(has_child(&top_box, "fine"));
    assert!(!has_child(&top_box, "oops"), "dangling reference disables");
    assert!(
        diagnostics.contains(&Diagnostic::UnknownFeature {
            module: "iffeature-unknown".to_string(),
            feature: "no-such".to_string(),
        }),
        "expected UnknownFeature diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn feature_statements_are_captured_in_the_ast() {
    let mut store = YangStore::new();
    store.add_path("tests/yang");
    store
        .read_with_resolve("iffeature-base")
        .expect("parse / resolve");
    let module = store.find_module("iffeature-base").expect("module found");

    let names: Vec<&str> = module.feature.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["iso", "extra", "dep"]);

    let iso = &module.feature[0];
    assert_eq!(
        iso.description.as_deref(),
        Some("ISO / OSI protocol support")
    );
    assert!(iso.if_feature.is_empty());

    let dep = &module.feature[2];
    assert_eq!(dep.if_feature.len(), 1, "dep depends on iso");
}
