// Integration test for YANG 1.1 `augment`.
//
// A tiny base module defines `container root { list item { key name;
// leaf name { type string; } } }`. A sibling module augments
// /base:root/base:item with a new leaf `badge`. After to_entry, the
// badge leaf should appear under the item list's dir.

use libyang::{Entry, YangStore, to_entry};
use std::rc::Rc;

fn load(name: &str, yang_dir: &str) -> Rc<Entry> {
    let mut store = YangStore::new();
    store.add_path(yang_dir);
    store.read_with_resolve(name).expect("parse / resolve");
    store.identity_resolve();
    let module = store.find_module(name).expect("module found");
    to_entry(&store, module)
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
