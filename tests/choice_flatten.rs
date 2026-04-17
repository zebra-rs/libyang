// Integration test for YANG `choice` case flattening.
//
// Per RFC 7950 §7.9.2, `choice` and `case` nodes do not appear in
// the data tree. libyang's `choice_entry` flattens each case's direct
// children into the choice's parent `dir`, tagging them with
// (choice, case) metadata on the Entry.
//
// This test builds a YangStore from a tiny module with a two-case
// choice inside a container, converts it to an Entry tree, and
// asserts the expected shape.

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
fn choice_cases_flatten_into_parent_dir() {
    // tests/yang/choice-sample.yang has:
    //   container options {
    //     choice option {
    //       case ao { leaf ao-keychain { type string; } }
    //       case md5 { leaf md5-keychain { type string; } }
    //     }
    //   }
    let root = load("choice-sample", "tests/yang");
    let options = find_child(&root, "options").expect("options");

    // After flattening, the case-child leaves should appear directly
    // under `options`, NOT under a nested `option` ChoiceEntry node.
    let ao_keychain = find_child(&options, "ao-keychain").expect("ao-keychain reachable");
    let md5_keychain = find_child(&options, "md5-keychain").expect("md5-keychain reachable");

    // And they should carry choice/case metadata.
    assert_eq!(ao_keychain.choice.borrow().as_deref(), Some("option"));
    assert_eq!(ao_keychain.case.borrow().as_deref(), Some("ao"));
    assert_eq!(md5_keychain.choice.borrow().as_deref(), Some("option"));
    assert_eq!(md5_keychain.case.borrow().as_deref(), Some("md5"));

    // The ChoiceEntry itself must NOT appear as a named path segment.
    assert!(
        find_child(&options, "option").is_none(),
        "choice node 'option' should not appear in parent dir"
    );
}
