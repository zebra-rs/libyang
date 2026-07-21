// Malformed augments are reported, not printed.
//
// `to_entry` used to write these five problems to stderr and carry on,
// which left a library deciding how an application reports its
// problems, and gave callers no way to react to them. They are now
// collected on the store, so a build can log them, fail on them, or
// ignore them.
//
// The tree behaviour is unchanged and still covered by tests/augment.rs;
// these tests cover the reporting.

use libyang::{AugmentKind, Diagnostic, Entry, YangStore, to_entry};
use std::rc::Rc;

const YANG_DIR: &str = "tests/yang";

/// Build `name`'s tree and return it with whatever was reported.
fn load_with_diagnostics(name: &str) -> (Rc<Entry>, Vec<Diagnostic>) {
    let mut store = YangStore::new();
    store.add_path(YANG_DIR);
    store.read_with_resolve(name).expect("parse / resolve");
    store.identity_resolve();
    let module = store.find_module(name).expect("module found");
    let entry = to_entry(&store, module);
    (entry, store.take_diagnostics())
}

fn find_child(ent: &Rc<Entry>, name: &str) -> Option<Rc<Entry>> {
    ent.dir.borrow().iter().find(|e| e.name == name).cloned()
}

#[test]
fn well_formed_schema_reports_nothing() {
    // The baseline that makes the rest meaningful: a healthy module must
    // not produce noise, or "no diagnostics" would be worthless as a
    // signal.
    let (_, diags) = load_with_diagnostics("augment-target");
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");
}

#[test]
fn top_level_augment_target_must_be_absolute() {
    let (root, diags) = load_with_diagnostics("augment-relative-target");
    assert_eq!(
        diags,
        vec![Diagnostic::AugmentTargetNotAbsolute {
            module: "augment-relative-target".into(),
            target: "art:box".into(),
        }]
    );

    // The node still resolved by name, so the augment applied: the
    // diagnostic reports a malformed target, it does not discard work.
    let box_ = find_child(&root, "box").expect("box container");
    assert!(find_child(&box_, "added").is_some());
}

#[test]
fn uses_augment_target_must_be_descendant() {
    let (_, diags) = load_with_diagnostics("uses-augment-absolute-target");
    assert_eq!(
        diags,
        vec![Diagnostic::AugmentTargetNotDescendant {
            module: "uses-augment-absolute-target".into(),
            target: "/box".into(),
        }]
    );
}

#[test]
fn unresolvable_target_names_the_missing_segment() {
    let (_, diags) = load_with_diagnostics("augment-missing-target");
    match diags.as_slice() {
        [
            Diagnostic::AugmentTargetNotFound {
                kind,
                module,
                target,
                missing,
            },
        ] => {
            assert_eq!(*kind, AugmentKind::Augment);
            assert_eq!(module, "augment-missing-target");
            assert_eq!(target, "/amt:box/amt:absent");
            // The failing segment is the actionable part — it says how
            // far the path got before it broke.
            assert!(
                missing.contains("absent"),
                "missing segment should name the node that did not match: {missing}"
            );
        }
        other => panic!("expected one AugmentTargetNotFound, got {other:?}"),
    }
}

#[test]
fn augment_into_leaf_is_reported() {
    let (_, diags) = load_with_diagnostics("augment-leaf-target");
    match diags.as_slice() {
        [Diagnostic::AugmentIntoLeaf { module, leaf, .. }] => {
            assert_eq!(module, "augment-leaf-target");
            assert_eq!(leaf, "target");
        }
        other => panic!("expected one AugmentIntoLeaf, got {other:?}"),
    }
}

#[test]
fn duplicate_node_is_reported() {
    let (_, diags) = load_with_diagnostics("augment-dup");
    match diags.as_slice() {
        [Diagnostic::AugmentDuplicateNode { module, name, .. }] => {
            assert_eq!(module, "augment-dup");
            assert_eq!(name, "base");
        }
        other => panic!("expected one AugmentDuplicateNode, got {other:?}"),
    }
}

#[test]
fn diagnostics_render_with_module_and_target() {
    // Callers that only print the diagnostic still need to know which
    // module and node are at fault.
    let (_, diags) = load_with_diagnostics("augment-missing-target");
    let text = diags[0].to_string();
    assert!(text.contains("augment-missing-target"), "{text}");
    assert!(text.contains("/amt:box/amt:absent"), "{text}");
}

#[test]
fn take_diagnostics_drains_and_accessor_borrows() {
    let mut store = YangStore::new();
    store.add_path(YANG_DIR);
    store
        .read_with_resolve("augment-dup")
        .expect("parse / resolve");
    store.identity_resolve();
    let module = store.find_module("augment-dup").expect("module found");
    let _ = to_entry(&store, module);

    // Borrowing accessor sees the diagnostic ...
    assert_eq!(store.diagnostics().len(), 1);
    // ... and still does, because reading does not consume.
    assert_eq!(store.diagnostics().len(), 1);

    // Taking drains, so a caller checking one module at a time does not
    // re-report earlier findings.
    assert_eq!(store.take_diagnostics().len(), 1);
    assert!(store.diagnostics().is_empty());
}
