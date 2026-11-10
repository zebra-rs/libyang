// The `ordered-by` statement (RFC 7950 §7.7.7) reaches the entry tree.
//
// The grammar always accepted it, but the AST conversion dropped it, so
// `ListAttr::ordered_by_user` was false on every list and leaf-list and
// a consumer could not tell a user-ordered list from a system-ordered
// one. `ordered-by user` now sets it; `ordered-by system` and no
// statement at all (system is the default) leave it unset.

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

fn ordered_by_user(top: &Rc<Entry>, name: &str) -> bool {
    find_child(top, name)
        .unwrap_or_else(|| panic!("{name} present"))
        .list_attr
        .as_ref()
        .unwrap_or_else(|| panic!("{name} is a list or leaf-list"))
        .ordered_by_user
}

#[test]
fn ordered_by_user_reaches_the_entry() {
    let root = load("ordered-by-sample", "tests/yang");
    let top = find_child(&root, "top").expect("top container");

    for name in ["by-user", "by-user-quoted", "ll-by-user"] {
        assert!(ordered_by_user(&top, name), "{name}: ordered-by user");
    }
    for name in ["by-system", "unset", "ll-unset"] {
        assert!(!ordered_by_user(&top, name), "{name}: system ordered");
    }
}
