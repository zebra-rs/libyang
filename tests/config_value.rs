// Regression test for the `config` statement value.
//
// to_entry drops `config false` nodes (state data) from the config tree
// and keeps `config true` / config-unset nodes. Before the parol 4.x work
// fixed it, config() read the `config` keyword token instead of its
// argument, so every explicit `config` parsed as false — which meant
// `config true` nodes were wrongly dropped. This test guards that.

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
fn config_true_kept_false_dropped() {
    let root = load("config-sample", "tests/yang");
    let top = find_child(&root, "top").expect("top container");

    assert!(
        find_child(&top, "cfg-true").is_some(),
        "`config true;` leaf must be kept in the config tree"
    );
    assert!(
        find_child(&top, "cfg-none").is_some(),
        "config-unset leaf must be kept in the config tree"
    );
    assert!(
        find_child(&top, "cfg-false").is_none(),
        "`config false;` leaf is state data and must be excluded"
    );
}
