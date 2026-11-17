// Constructs that SONiC's YANG models use and this library did not
// read: docs/sonic-yang-gaps.md lists them by the numbers used here.
//
// 1 to 12 are forms the grammar refused, so each is in the fixture
// `gap-syntax` (or a `gap-ns-*` one) and the first thing these tests
// say is that the fixture parses. 13 to 15 parsed and built a tree that
// was wrong.

use libyang::{Entry, ModuleNode, RangeNode, YangStore, YangType, to_entry};
use std::rc::Rc;

fn store(name: &str) -> YangStore {
    let mut store = YangStore::new();
    store.add_path("tests/yang");
    store
        .read_with_resolve(name)
        .unwrap_or_else(|e| panic!("{name} must parse: {e:?}"));
    store.identity_resolve();
    store
}

fn tree(store: &YangStore, name: &str) -> Rc<Entry> {
    let module = store.find_module(name).expect("module found");
    let root = to_entry(store, module);
    assert_eq!(store.take_diagnostics(), vec![], "{name}");
    root
}

/// The entry at `path` under the fixture's `top` container.
fn top(name: &str) -> Rc<Entry> {
    let store = store("gap-syntax");
    let root = tree(&store, "gap-syntax");
    let mut entry = child(&root, "top");
    for step in name.split('/').filter(|s| !s.is_empty()) {
        entry = child(&entry, step);
    }
    entry
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

fn module(store: &YangStore) -> &ModuleNode {
    store.find_module("gap-syntax").expect("module found")
}

// --- 1: the namespace is any URI ---

#[test]
fn gap01_namespace_is_any_uri() {
    for (name, namespace) in [
        ("gap-syntax", "http://example.com/yang/gap-syntax"),
        (
            "gap-ns-https-query",
            "https://example.com/yang/ns?module=x&v=1#frag",
        ),
        ("gap-ns-urn-quoted", "urn:ietf:params:xml:ns:yang:gap"),
        ("gap-ns-urn-unquoted", "urn:example:gap.ns-1"),
    ] {
        let store = store(name);
        let module = store.find_module(name).expect("module found");
        assert_eq!(module.namespace.as_deref(), Some(namespace), "{name}");
    }
}

// --- 2: pattern, range and length with substatements ---

#[test]
fn gap02_restrictions_take_substatements() {
    let restricted = top("restricted");
    let t = restricted.type_node.as_ref().expect("type");
    assert_eq!(t.pattern.as_deref(), Some("[a-z]+"));
    assert_eq!(
        t.length.as_ref().map(|l| l.to_string()).as_deref(),
        Some("<1..8>")
    );

    // `modifier invert-match` is read. The pattern is then one the
    // value must not match, and `pattern` does not hold it.
    let inverted = top("inverted");
    assert_eq!(inverted.type_node.as_ref().unwrap().pattern, None);

    for (leaf, range) in [("ranged", "<1..4095>"), ("ranged-unquoted", "<1..65535>")] {
        let entry = top(leaf);
        let t = entry.type_node.as_ref().expect("type");
        assert_eq!(t.kind, YangType::Uint16, "{leaf}");
        assert!(matches!(t.range, Some(RangeNode::U16(_))), "{leaf}");
        assert_eq!(
            t.range.as_ref().map(|r| r.to_string()).as_deref(),
            Some(range),
            "{leaf}"
        );
    }
    let entry = top("length-unquoted");
    let t = entry.type_node.as_ref().expect("type");
    assert_eq!(
        t.length.as_ref().map(|l| l.to_string()).as_deref(),
        Some("<0..128>")
    );
}

// --- 3: an unquoted leafref path ---

#[test]
fn gap03_leafref_path_unquoted() {
    let entry = top("ref");
    let t = entry.type_node.as_ref().expect("type");
    assert_eq!(t.kind, YangType::Leafref);
    assert_eq!(t.path.as_deref(), Some("/gs:top/gs:item/gs:name"));
    assert_eq!(
        top("ref-relative")
            .type_node
            .as_ref()
            .unwrap()
            .path
            .as_deref(),
        Some("../item/name")
    );
}

/// The scanner that reads those two arguments still takes a comment
/// for a comment, and an unquoted string that starts with `/` for a
/// path.
#[test]
fn gap03_gap09_comment_before_the_argument() {
    let store = store("gap-arg-comments");
    let root = tree(&store, "gap-arg-comments");
    let pattern = |leaf| {
        child(&root, leaf)
            .type_node
            .as_ref()
            .unwrap()
            .pattern
            .clone()
    };
    assert_eq!(pattern("a").as_deref(), Some("[a-z]+"));
    assert_eq!(pattern("b").as_deref(), Some("[0-9]+"));
    assert_eq!(
        child(&root, "c")
            .type_node
            .as_ref()
            .unwrap()
            .path
            .as_deref(),
        Some("/gc:a")
    );
}

// --- 4: an extension used with no argument ---

#[test]
fn gap04_extension_with_no_argument() {
    let store = store("gap-syntax");
    let top = &module(&store).d.container[0];
    assert_eq!(top.unknown.len(), 1);
    assert_eq!(top.unknown[0].name, "gt:marker");
    assert_eq!(top.unknown[0].argument, "");
}

// --- 5: error-app-tag and reference under must ---

#[test]
fn gap05_must_substatements() {
    let store = store("gap-syntax");
    let checked = module(&store).d.container[0]
        .d
        .leaf
        .iter()
        .find(|l| l.name == "checked")
        .expect("leaf");
    assert_eq!(checked.must.len(), 1);
    assert_eq!(checked.must[0].name, "current() != 'x'");
}

// --- 6: a typedef inside a container or a list ---

#[test]
fn gap06_typedef_inside_a_container_or_list() {
    let entry = top("uses-nested");
    let t = entry
        .type_node
        .as_ref()
        .expect("the nested typedef resolves");
    assert_eq!(t.kind, YangType::Uint8);
    assert_eq!(
        t.range.as_ref().map(|r| r.to_string()).as_deref(),
        Some("<1..9>")
    );

    let index = top("item/index");
    assert_eq!(
        index.type_node.as_ref().expect("type").kind,
        YangType::Uint8
    );

    // They are among the module's typedefs, after its own.
    let store = store("gap-syntax");
    let names: Vec<&str> = module(&store)
        .typedef
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["own", "local-flag", "local-small", "nested", "index-type"]
    );
}

// --- 7, 10: a quoted default is an ordinary string ---

#[test]
fn gap07_gap10_quoted_defaults() {
    // An empty default, one over several lines with an escaped quote in
    // it, and an unquoted one beside them: the fixture parses, and the
    // leaves after them are read.
    for leaf in ["empty-default", "banner", "plain-default", "percent"] {
        top(leaf);
    }
    assert_eq!(
        top("plain-default").type_node.as_ref().unwrap().kind,
        YangType::Uint8
    );
}

// --- 8: a decimal64 range ---

#[test]
fn gap08_decimal_range() {
    let entry = top("percent");
    assert_eq!(
        entry.type_node.as_ref().expect("type").kind,
        YangType::Decimal64
    );
}

// --- 9: an unquoted pattern ---

#[test]
fn gap09_pattern_unquoted() {
    assert_eq!(
        top("digit").type_node.as_ref().unwrap().pattern.as_deref(),
        Some("[0-7]")
    );
}

// --- 11: a single-quoted string has no escapes ---

#[test]
fn gap11_backslash_in_a_single_quoted_string() {
    assert_eq!(
        top("no-newline")
            .type_node
            .as_ref()
            .unwrap()
            .pattern
            .as_deref(),
        Some(r"[^\n\r]+")
    );
}

// --- 12: single- and double-quoted strings joined ---

#[test]
fn gap12_mixed_quotes_joined() {
    assert_eq!(
        top("joined").type_node.as_ref().unwrap().pattern.as_deref(),
        Some(r"[^ @,\\']*")
    );
}

// --- 13: every key of a list ---

#[test]
fn gap13_a_list_has_all_its_keys() {
    assert_eq!(top("item").key, ["name", "index"]);
    assert_eq!(top("triple").key, ["a", "b", "c"]);
    assert_eq!(top("single").key, ["name"]);
}

// --- 14: a typedef named with the module's own prefix ---

#[test]
fn gap14_typedef_by_own_prefix() {
    let entry = top("uses-own");
    let t = entry
        .type_node
        .as_ref()
        .expect("gs:own resolves in gap-syntax");
    assert_eq!(t.kind, YangType::String);
    assert_eq!(t.typedef.as_deref(), Some("gs:own"));
}

// --- 15: a typedef of a typedef ---

#[test]
fn gap15_typedef_of_a_typedef() {
    // local-flag -> gt:flag -> boolean
    let entry = top("uses-local-flag");
    let t = entry.type_node.as_ref().expect("type");
    assert_eq!(t.kind, YangType::Boolean);
    // Known by the last typedef before the built-in type.
    assert_eq!(t.typedef.as_deref(), Some("gt:flag"));

    // gs:local-small -> gt:small -> gt:number -> uint8, and `small`
    // adds a range on the way. `gt:number` inside gap-types is that
    // module's own prefix.
    let entry = top("uses-local-small");
    let t = entry.type_node.as_ref().expect("type");
    assert_eq!(t.kind, YangType::Uint8);
    assert_eq!(t.typedef.as_deref(), Some("gt:number"));
    assert_eq!(
        t.range.as_ref().map(|r| r.to_string()).as_deref(),
        Some("<1..10>")
    );
    // Read as a range of the type the chain ends at.
    assert!(matches!(t.range, Some(RangeNode::U8(_))), "{:?}", t.range);
}

/// The arms of a union are looked up in the module that defines the
/// union. `number` has no prefix in gap-types and gap-syntax has no
/// typedef of that name: read in the using module, the arm was lost.
#[test]
fn gap15_union_arms_resolve_in_the_defining_module() {
    let entry = top("uses-union");
    let t = entry.type_node.as_ref().expect("type");
    assert_eq!(t.kind, YangType::Union);
    let kinds: Vec<YangType> = t.union.iter().map(|arm| arm.kind).collect();
    assert_eq!(kinds, [YangType::Uint8, YangType::String]);
}
