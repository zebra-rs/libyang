// Integration test: inline union arms with a recognized built-in
// kind (e.g. `type uint32;` written directly inside the union, not
// via a typedef reference) must survive YANG store resolution so
// the matcher can dispatch on them.
//
// Regression: a leaf declared as
//     type union { type uint32; type inet:ipv4-address; }
// previously kept only the path arm (ipv4-address) because
// type_resolve's Union branch only pushed arms whose kind was Path.

use libyang::{Entry, YangStore, YangType, to_entry};
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
fn inline_union_keeps_builtin_and_typedef_arms() {
    let root = load("union-sample", "tests/yang");
    let leaf = find_child(&root, "area-id").expect("area-id leaf");
    let t = leaf.type_node.as_ref().expect("type_node");
    assert_eq!(t.kind, YangType::Union);
    let kinds: Vec<YangType> = t.union.iter().map(|n| n.kind).collect();
    assert!(
        kinds.contains(&YangType::Uint32),
        "uint32 arm must survive; got {kinds:?}"
    );
    assert!(
        kinds.contains(&YangType::String),
        "ipv4-addr (resolves to string) arm must survive; got {kinds:?}"
    );
}

#[test]
fn inline_union_keeps_all_builtin_arms() {
    let root = load("union-sample", "tests/yang");
    let leaf = find_child(&root, "scalar-or-string").expect("scalar-or-string leaf");
    let t = leaf.type_node.as_ref().expect("type_node");
    assert_eq!(t.kind, YangType::Union);
    let kinds: Vec<YangType> = t.union.iter().map(|n| n.kind).collect();
    assert!(
        kinds.contains(&YangType::Uint32),
        "uint32 arm must survive; got {kinds:?}"
    );
    assert!(
        kinds.contains(&YangType::String),
        "string arm must survive; got {kinds:?}"
    );
}
