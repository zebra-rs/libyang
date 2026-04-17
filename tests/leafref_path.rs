// Integration test: leafref type captures its `path "..."` argument
// on TypeNode, and the path survives typedef resolution (a typedef
// whose underlying type is a leafref propagates the path to the
// Entry of a leaf that uses the typedef).

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

fn find_child<'a>(ent: &'a Rc<Entry>, name: &str) -> Option<Rc<Entry>> {
    ent.dir.borrow().iter().find(|e| e.name == name).cloned()
}

#[test]
fn leafref_direct_captures_path() {
    // tests/yang/leafref-sample.yang has a direct `type leafref { path
    // "/items/item/name"; }` leaf.
    let root = load("leafref-sample", "tests/yang");
    let picked = find_child(&root, "picked").expect("picked leaf");
    let t = picked.type_node.as_ref().expect("type_node");
    assert_eq!(t.path.as_deref(), Some("/items/item/name"));
}

#[test]
fn leafref_through_typedef_captures_path() {
    // tests/yang/leafref-sample.yang also has `typedef item-ref {
    // type leafref { path "/items/item/name"; } }` and a leaf using
    // `type item-ref`. The path must survive typedef resolution.
    let root = load("leafref-sample", "tests/yang");
    let picked_td = find_child(&root, "picked-via-typedef").expect("picked-via-typedef leaf");
    let t = picked_td.type_node.as_ref().expect("type_node");
    assert_eq!(t.path.as_deref(), Some("/items/item/name"));
}
