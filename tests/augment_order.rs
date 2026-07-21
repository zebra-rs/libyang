// Regression test: `to_entry` must produce the same child order on
// every run.
//
// Augmented nodes are appended to their target's `dir` as each
// augment is applied, so the child order is decided by the order
// `to_entry` walks the loaded modules. `YangStore::modules` is a
// `BTreeMap` to make that walk deterministic. It was a `HashMap`, and
// because Rust seeds each `HashMap` instance separately the order
// differed not just between processes but between two stores in the
// same process — which is what these tests exercise, by loading the
// same files through several fresh stores and comparing.
//
// tests/yang/augment-order-target.yang defines `container root` and
// bootstrap-imports six sibling modules (augment-order-{a..f}), each
// augmenting `/base:root` with one leaf. Six siblings give 720
// possible orders, so repeated loads agreeing by chance is not a
// plausible explanation for a pass.

use libyang::{Entry, YangStore, to_entry};
use std::rc::Rc;

const YANG_DIR: &str = "tests/yang";
const TARGET: &str = "augment-order-target";

/// Build the tree from a *fresh* store, so each call gets its own
/// `HashMap` instance (and thus its own iteration order).
fn load_root() -> Rc<Entry> {
    let mut store = YangStore::new();
    store.add_path(YANG_DIR);
    store.read_with_resolve(TARGET).expect("parse / resolve");
    store.identity_resolve();
    let module = store.find_module(TARGET).expect("module found");
    let entry = to_entry(&store, module);
    entry
        .dir
        .borrow()
        .iter()
        .find(|e| e.name == "root")
        .cloned()
        .expect("container root")
}

fn child_names(ent: &Rc<Entry>) -> Vec<String> {
    ent.dir.borrow().iter().map(|e| e.name.clone()).collect()
}

#[test]
fn augment_child_order_is_stable_across_loads() {
    let first = child_names(&load_root());

    // All six augments landed, so the order really is observable.
    for m in ["a", "b", "c", "d", "e", "f"] {
        let leaf = format!("leaf-{m}");
        assert!(
            first.contains(&leaf),
            "augment from augment-order-{m} missing; got {first:?}"
        );
    }

    for i in 1..8 {
        let next = child_names(&load_root());
        assert_eq!(
            first, next,
            "augmented child order changed between load 0 and load {i}: \
             to_entry must not depend on the hash seeding of the store's \
             module map"
        );
    }
}

#[test]
fn augment_child_order_is_sorted_by_module_name() {
    // The chosen deterministic order is the augmenting modules' names,
    // sorted. Pin it so the guarantee is a documented contract rather
    // than an accident of the implementation.
    let names = child_names(&load_root());
    let augmented: Vec<&String> = names.iter().filter(|n| n.starts_with("leaf-")).collect();
    let want = ["leaf-a", "leaf-b", "leaf-c", "leaf-d", "leaf-e", "leaf-f"];
    assert_eq!(
        augmented.len(),
        want.len(),
        "expected one leaf per augmenting module; got {augmented:?}"
    );
    for (got, want) in augmented.iter().zip(want.iter()) {
        assert_eq!(got.as_str(), *want, "augment order: {augmented:?}");
    }
}
