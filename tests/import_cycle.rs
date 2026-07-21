// Loading must terminate when modules import each other.
//
// RFC 7950 §5.1 lets modules import each other, so an import graph can
// contain cycles and shared (diamond) dependencies. `read_with_resolve`
// handles both by skipping anything already loaded.
//
// This is the property the removed `YangStore::read` lacked: it
// recursed into every import unconditionally, so the fixtures below —
// where a target module and its six augmenting siblings import each
// other — overflowed the stack and aborted the process. Nothing pinned
// that, so the guard could have been refactored away unnoticed.

use libyang::YangStore;

const YANG_DIR: &str = "tests/yang";

fn load(name: &str) -> YangStore {
    let mut store = YangStore::new();
    store.add_path(YANG_DIR);
    store
        .read_with_resolve(name)
        .expect("mutually importing modules should load");
    store.identity_resolve();
    store
}

#[test]
fn mutually_importing_modules_terminate() {
    // augment-order-target imports augment-order-{a..f}, and each of
    // those imports augment-order-target back.
    let store = load("augment-order-target");

    assert!(store.find_module("augment-order-target").is_some());
    for m in ["a", "b", "c", "d", "e", "f"] {
        let name = format!("augment-order-{m}");
        assert!(
            store.find_module(&name).is_some(),
            "{name} should have been loaded through the cycle"
        );
    }
}

#[test]
fn entering_the_cycle_from_either_side_works() {
    // Starting at a sibling reaches the target the same way, so
    // termination does not depend on which module is the entry point.
    let store = load("augment-order-a");
    assert!(store.find_module("augment-order-a").is_some());
    assert!(store.find_module("augment-order-target").is_some());
}

#[test]
fn submodules_are_stored_not_silently_dropped() {
    // The removed `read` matched only `Node::Module`, so asking it for a
    // submodule returned Ok(()) having stored nothing — a success the
    // caller could not tell from a real load. `read_with_resolve` keeps
    // submodules, reachable via `find_submodule`.
    let mut store = YangStore::new();
    store.add_path(YANG_DIR);
    store
        .read_with_resolve("augment-target")
        .expect("module loads");

    // A module is not a submodule, and vice versa: the two namespaces
    // stay distinct.
    assert!(store.find_module("augment-target").is_some());
    assert!(store.find_submodule("augment-target").is_none());
}
