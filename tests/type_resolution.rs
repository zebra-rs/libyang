// What a type that names a typedef resolves to, and what a `range` or
// `length` may hold. Each of these was wrong after typedefs of typedefs
// were first followed (docs/sonic-yang-gaps.md, 2, 8, 14 and 15).

use libyang::{Entry, RangeNode, TypeNode, YangError, YangStore, YangType, to_entry};
use std::error::Error;
use std::rc::Rc;

fn root(name: &str) -> Rc<Entry> {
    let mut store = YangStore::new();
    store.add_path("tests/yang");
    store
        .read_with_resolve(name)
        .unwrap_or_else(|e| panic!("{name} must parse: {e:?}"));
    store.identity_resolve();
    let module = store.find_module(name).expect("module found");
    let root = to_entry(&store, module);
    assert_eq!(store.take_diagnostics(), vec![], "{name}");
    root
}

fn child(entry: &Rc<Entry>, name: &str) -> Rc<Entry> {
    entry
        .dir
        .borrow()
        .iter()
        .find(|e| e.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("no `{name}` under `{}`", entry.name))
}

fn type_of(root: &Rc<Entry>, leaf: &str) -> TypeNode {
    child(root, leaf)
        .type_node
        .clone()
        .unwrap_or_else(|| panic!("`{leaf}` has a type"))
}

fn range(t: &TypeNode) -> Option<String> {
    t.range.as_ref().map(|r| r.to_string())
}

fn length(t: &TypeNode) -> Option<String> {
    t.length.as_ref().map(|l| l.to_string())
}

fn enums(t: &TypeNode) -> Vec<&str> {
    t.enum_stmt.iter().map(|e| e.name.as_str()).collect()
}

// --- Restrictions on a reference to a typedef ---

#[test]
fn a_range_on_the_reference_replaces_the_typedefs() {
    let root = root("typedef-restrict");

    let t = type_of(&root, "small");
    assert_eq!(t.kind, YangType::Uint8);
    assert_eq!(t.typedef.as_deref(), Some("percent"));
    assert_eq!(range(&t).as_deref(), Some("<1..10>"));
    // Read as a range of the type the typedef stands for.
    assert!(matches!(t.range, Some(RangeNode::U8(_))), "{:?}", t.range);

    // The typedef has no range of its own.
    let t = type_of(&root, "ranged");
    assert_eq!(range(&t).as_deref(), Some("<1..10>"));
    assert!(matches!(t.range, Some(RangeNode::U8(_))), "{:?}", t.range);

    // No restriction on the reference: the typedef's.
    let t = type_of(&root, "unrestricted");
    assert_eq!(range(&t).as_deref(), Some("<0..100>"));

    // A leaf-list.
    let t = type_of(&root, "ports");
    assert_eq!(range(&t).as_deref(), Some("<5..6>"));
}

#[test]
fn a_pattern_and_a_length_on_the_reference() {
    let root = root("typedef-restrict");

    let t = type_of(&root, "short-name");
    assert_eq!(t.kind, YangType::String);
    assert_eq!(t.pattern.as_deref(), Some("[a-z]+"));
    assert_eq!(length(&t).as_deref(), Some("<1..8>"));

    // Only a pattern is added: the length is the typedef's.
    let t = type_of(&root, "vrf");
    assert_eq!(t.pattern.as_deref(), Some("Vrf[a-zA-Z0-9_-]+"));
    assert_eq!(length(&t).as_deref(), Some("<1..64>"));
    assert_eq!(t.typedef.as_deref(), Some("tr:name"));
}

#[test]
fn restrictions_on_the_arms_of_a_union() {
    let root = root("typedef-restrict");
    let t = type_of(&root, "either");
    assert_eq!(t.kind, YangType::Union);
    assert_eq!(t.union.len(), 2);
    assert_eq!(t.union[0].kind, YangType::Uint8);
    assert_eq!(range(&t.union[0]).as_deref(), Some("<1..10>"));
    assert_eq!(t.union[1].kind, YangType::String);
    assert_eq!(length(&t.union[1]).as_deref(), Some("<1..8>"));
}

#[test]
fn enum_statements_on_the_reference_name_the_values_left() {
    let root = root("typedef-restrict");

    // `warm` is `color` with `red` alone.
    let t = type_of(&root, "warm-color");
    assert_eq!(t.kind, YangType::Enumeration);
    assert_eq!(enums(&t), ["red"]);
    assert_eq!(t.typedef.as_deref(), Some("color"));

    // On the leaf itself, in the order written there.
    let t = type_of(&root, "two-colors");
    assert_eq!(enums(&t), ["blue", "red"]);
}

#[test]
fn a_range_bound_above_i64_on_a_typedef() {
    let root = root("typedef-restrict");
    let t = type_of(&root, "big");
    assert_eq!(t.kind, YangType::Uint64);
    assert_eq!(range(&t).as_deref(), Some("<10000000000000000000..max>"));
}

#[test]
fn a_decimal_range_on_a_typedef_is_not_kept() {
    let root = root("typedef-restrict");
    let t = type_of(&root, "part");
    assert_eq!(t.kind, YangType::Decimal64);
    assert_eq!(t.range, None);
}

// --- An inverted pattern ---

/// `pattern` is one the value must match. A pattern with `modifier
/// invert-match` is one it must not match, and is not there.
#[test]
fn an_inverted_pattern_is_not_a_pattern_to_match() {
    let root = root("typedef-restrict");
    let t = type_of(&root, "not-reserved");
    assert_eq!(t.pattern.as_deref(), Some("[a-z]+"));
}

// --- Typedefs defined by themselves ---

#[test]
fn a_typedef_defined_by_itself_has_no_type() {
    let root = root("typedef-cycle");

    // The arm that leads back is not there; the others are.
    let kinds = |leaf| -> Vec<YangType> {
        let t = type_of(&root, leaf);
        assert_eq!(t.kind, YangType::Union, "{leaf}");
        t.union.iter().map(|arm| arm.kind).collect()
    };
    assert_eq!(kinds("through-union"), [YangType::String]);
    assert_eq!(kinds("two-unions"), [YangType::Boolean, YangType::Uint8]);
    assert_eq!(kinds("twice"), [YangType::String]);

    assert_eq!(child(&root, "itself").type_node, None);
}

// --- An identityref by the module's own prefix ---

#[test]
fn an_identityref_by_own_prefix_is_an_identityref() {
    let root = root("identityref-own-prefix");

    let prefixed = type_of(&root, "prefixed");
    assert_eq!(prefixed.kind, YangType::Identityref);
    assert_eq!(prefixed.base.as_deref(), Some("iop:base-id"));
    assert_eq!(prefixed.enum_stmt, vec![]);

    // As the same type written with no prefix.
    let plain = type_of(&root, "plain");
    assert_eq!(plain.kind, YangType::Identityref);
    assert_eq!(plain.base.as_deref(), Some("base-id"));
}

// --- The bounds of a range and of a length ---

/// The error loading `name`, which must be a syntax error at `line`
/// that quotes `bound`.
fn bound_error(name: &str, line: u32, bound: &str) {
    let mut store = YangStore::new();
    store.add_path("tests/yang");
    let err = store
        .read_with_resolve(name)
        .expect_err("the module must not load");
    match &err {
        YangError::ParseError { path, .. } => {
            assert!(path.ends_with(format!("{name}.yang")), "{path:?}")
        }
        other => panic!("expected ParseError, got {other:?}"),
    }
    let detail = format!("{:?}", err.source().expect("a source"));
    assert!(detail.contains(&format!("`{bound}` is not")), "{detail}");
    assert!(detail.contains(&format!("start_line: {line}")), "{detail}");
}

/// A bound may have a fraction, for a `decimal64`. On an integer type
/// it is refused when the file is parsed.
#[test]
fn a_fraction_in_an_integer_range_is_a_parse_error() {
    bound_error("bound-range-fraction", 7, "1.5");
}

#[test]
fn a_fraction_in_a_length_is_a_parse_error() {
    bound_error("bound-length-fraction", 7, "4.5");
}

/// A bound the type cannot hold, in a type that is an arm of a union.
#[test]
fn a_range_bound_outside_the_type_is_a_parse_error() {
    bound_error("bound-range-too-large", 9, "300");
}
