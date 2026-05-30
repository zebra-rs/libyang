// Integration test for YANG 1.1 `augment`.
//
// A tiny base module defines `container root { list item { key name;
// leaf name { type string; } } }`. A sibling module augments
// /base:root/base:item with a new leaf `badge`. After to_entry, the
// badge leaf should appear under the item list's dir.

use libyang::{AugmentNode, Entry, YangStore, to_entry};
use std::rc::Rc;

fn load(name: &str, yang_dir: &str) -> Rc<Entry> {
    let mut store = YangStore::new();
    store.add_path(yang_dir);
    store.read_with_resolve(name).expect("parse / resolve");
    store.identity_resolve();
    let module = store.find_module(name).expect("module found");
    to_entry(&store, module)
}

// Parse a module and return a clone of its top-level augment list so
// tests can assert on the substatements the AST builder captured.
fn augments_of(name: &str, yang_dir: &str) -> Vec<AugmentNode> {
    let mut store = YangStore::new();
    store.add_path(yang_dir);
    store.read_with_resolve(name).expect("parse / resolve");
    store
        .find_module(name)
        .expect("module found")
        .augment
        .clone()
}

fn find_child(ent: &Rc<Entry>, name: &str) -> Option<Rc<Entry>> {
    ent.dir.borrow().iter().find(|e| e.name == name).cloned()
}

#[test]
fn augment_injects_leaf_into_cross_module_target() {
    // tests/yang/augment-target.yang — base module.
    // tests/yang/augment-consumer.yang — consumer that imports
    // augment-target and augments /base:root/base:item with `badge`.
    //
    // We load the consumer as the root module; apply_augment walks
    // target's tree, which for cross-module targets requires that
    // the augment target lives in the root module's own subtree.
    // For this test we therefore load the TARGET module as root and
    // rely on the consumer being present in the store (it's
    // imported reversely via a marker) — the cleanest realistic
    // shape for zebra-rs is that augments live on the imports of
    // the root module.
    let root = load("augment-target", "tests/yang");

    // Walk: root/root/item
    let root_container = find_child(&root, "root").expect("root container");
    let item = find_child(&root_container, "item").expect("item list");
    let badge = find_child(&item, "badge").expect("augmented badge leaf");
    assert!(
        badge.is_leaf(),
        "augmented node should be a leaf, got {:?}",
        badge.kind
    );
}

#[test]
fn augment_captures_when_status_action_and_case() {
    // tests/yang/augment-substmts.yang exercises the substatements the
    // AST builder used to silently drop: when, status, action (target
    // is a container) and an explicit case (target is a choice).
    let augments = augments_of("augment-substmts", "tests/yang");
    assert_eq!(augments.len(), 2, "two augment statements expected");

    let top = &augments[0];
    assert_eq!(top.target, "/sub:top");
    assert_eq!(
        top.when.as_ref().map(|w| w.name.as_str()),
        Some("true()"),
        "when condition captured"
    );
    assert!(top.status.is_some(), "status captured");
    assert_eq!(top.action.len(), 1, "action captured");
    assert_eq!(top.action[0].name, "reset");
    assert_eq!(top.d.leaf.len(), 1, "data-def leaf captured");
    assert_eq!(top.d.leaf[0].name, "added");

    let sel = &augments[1];
    assert_eq!(sel.target, "/sub:top/sub:sel");
    assert_eq!(sel.cases.len(), 1, "case captured");
    assert_eq!(sel.cases[0].name, "extra");
}

#[test]
fn augment_applies_within_same_module() {
    // The augmenting module's own prefix maps to its own tree, so the
    // same-module augment in augment-apply.yang resolves and injects
    // `added` under `top`.
    let root = load("augment-apply", "tests/yang");
    let top = find_child(&root, "top").expect("top container");
    let added = find_child(&top, "added").expect("augmented leaf present");
    assert!(added.is_leaf(), "augmented node should be a leaf");
}
