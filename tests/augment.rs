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

fn count_children(ent: &Rc<Entry>, name: &str) -> usize {
    ent.dir.borrow().iter().filter(|e| e.name == name).count()
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

#[test]
fn uses_augment_injects_into_grouping_subtree() {
    // tests/yang/uses-augment.yang: container top { uses g { augment
    // "box" { leaf added; } } } where grouping g defines box/base.
    // After expansion, box should hold both the grouping's `base` and
    // the uses-augment's `added`.
    let root = load("uses-augment", "tests/yang");
    let top = find_child(&root, "top").expect("top container");
    let box_ = find_child(&top, "box").expect("box from grouping g");
    assert!(
        find_child(&box_, "base").is_some(),
        "grouping's own leaf should survive"
    );
    let added = find_child(&box_, "added").expect("uses-augment leaf present");
    assert!(added.is_leaf(), "augmented node should be a leaf");
}

#[test]
fn augment_adds_action_to_container() {
    // tests/yang/augment-action.yang augments container `box` with an
    // `action ping;`. The action should appear as an action entry.
    let root = load("augment-action", "tests/yang");
    let box_ = find_child(&root, "box").expect("box container");
    let ping = find_child(&box_, "ping").expect("augmented action present");
    assert!(ping.is_action(), "augmented node should be an action");
}

#[test]
fn augment_adds_case_to_choice() {
    // tests/yang/augment-choice.yang augments choice `sel` (in `top`)
    // with `case extra { leaf c; }`. The case's leaf is flattened into
    // `top` and tagged with the choice/case names, alongside the
    // pre-existing `base` case's leaf.
    let root = load("augment-choice", "tests/yang");
    let top = find_child(&root, "top").expect("top container");

    assert!(
        find_child(&top, "b").is_some(),
        "pre-existing case leaf should survive"
    );
    let c = find_child(&top, "c").expect("augmented case leaf present");
    assert!(c.is_leaf(), "augmented node should be a leaf");
    assert_eq!(
        c.choice.borrow().as_deref(),
        Some("sel"),
        "leaf should be tagged with the choice name"
    );
    assert_eq!(
        c.case.borrow().as_deref(),
        Some("extra"),
        "leaf should be tagged with the case name"
    );
}

#[test]
fn augment_rejects_duplicate_node() {
    // tests/yang/augment-dup.yang augments container `box` with a `base`
    // leaf that already exists plus a new `fresh` leaf. The duplicate
    // must be rejected; the new node must be added.
    let root = load("augment-dup", "tests/yang");
    let box_ = find_child(&root, "box").expect("box container");
    assert_eq!(
        count_children(&box_, "base"),
        1,
        "duplicate `base` should not be added a second time"
    );
    assert!(
        find_child(&box_, "fresh").is_some(),
        "non-duplicate `fresh` should still be added"
    );
}

#[test]
fn augment_rejects_leaf_target() {
    // tests/yang/augment-leaf-target.yang targets the leaf `box/target`,
    // which is invalid. Nothing should be injected under the leaf.
    let root = load("augment-leaf-target", "tests/yang");
    let box_ = find_child(&root, "box").expect("box container");
    let target = find_child(&box_, "target").expect("target leaf");
    assert!(target.is_leaf(), "target should be a leaf");
    assert!(
        find_child(&target, "nope").is_none(),
        "nothing should be added under a leaf target"
    );
}

#[test]
fn augment_adds_case_to_empty_choice() {
    // tests/yang/augment-empty-choice.yang augments a choice that has no
    // cases of its own. The choice is recorded on `top`, so the
    // augment's case leaf is still injected and tagged.
    let root = load("augment-empty-choice", "tests/yang");
    let top = find_child(&root, "top").expect("top container");
    let v = find_child(&top, "v").expect("augmented case leaf present");
    assert!(v.is_leaf(), "augmented node should be a leaf");
    assert_eq!(v.choice.borrow().as_deref(), Some("sel"));
    assert_eq!(v.case.borrow().as_deref(), Some("only"));
}
