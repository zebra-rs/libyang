use crate::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, PartialEq, Default)]
pub enum EntryKind {
    #[default]
    LeafEntry,
    DirectoryEntry,
    ChoiceEntry,
    ActionEntry,
}

#[derive(Default, Debug)]
pub struct ListAttr {
    pub min_elements: u64,
    pub max_elements: u64,
    pub ordered_by_user: bool,
}

impl ListAttr {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Entry {
    pub name: String,
    pub kind: EntryKind,
    pub presence: bool,
    pub mandatory: bool,
    pub dir: RefCell<Vec<Rc<Entry>>>,
    pub key: Vec<String>,
    pub extension: HashMap<String, String>,
    pub parent: RefCell<Option<Rc<Entry>>>,
    pub type_node: Option<TypeNode>,
    pub list_attr: Option<ListAttr>,
    pub choice_cases: Option<Vec<Rc<Entry>>>,

    // When this entry was introduced by a `case` inside a `choice`,
    // these record the choice and case names so consumers can reason
    // about mutual exclusion at commit time. Per RFC 7950 §7.9.2,
    // choice/case nodes do not themselves appear in the data tree;
    // only the case's direct children do. We flatten accordingly —
    // the case's children land in the choice's parent `dir`, tagged
    // here with the choice and case names.
    pub choice: RefCell<Option<String>>,
    pub case: RefCell<Option<String>>,

    // Names of `choice` nodes defined directly under this entry. A
    // choice is otherwise invisible in the flattened tree (only its
    // cases' children appear), so recording the names lets an augment
    // target a choice — including one that has no cases yet.
    pub choice_defs: RefCell<Vec<String>>,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_dir(name: String) -> Self {
        Self {
            name,
            kind: EntryKind::DirectoryEntry,
            ..Default::default()
        }
    }

    pub fn new_leaf(name: String) -> Self {
        Self {
            name,
            kind: EntryKind::LeafEntry,
            ..Default::default()
        }
    }

    pub fn new_list(name: String, key: Vec<String>) -> Self {
        Self {
            name,
            kind: EntryKind::DirectoryEntry,
            key,
            ..Default::default()
        }
    }

    pub fn new_choice(name: String) -> Self {
        Self {
            name,
            kind: EntryKind::ChoiceEntry,
            choice_cases: Some(Vec::new()),
            ..Default::default()
        }
    }

    pub fn new_action(name: String) -> Self {
        Self {
            name,
            kind: EntryKind::ActionEntry,
            ..Default::default()
        }
    }

    pub fn has_key(&self) -> bool {
        !self.key.is_empty()
    }

    pub fn is_directory_entry(&self) -> bool {
        self.kind == EntryKind::DirectoryEntry
    }

    pub fn is_leaf_entry(&self) -> bool {
        self.kind == EntryKind::LeafEntry
    }

    pub fn is_container(&self) -> bool {
        self.kind == EntryKind::DirectoryEntry && self.list_attr.is_none()
    }

    pub fn is_list(&self) -> bool {
        self.kind == EntryKind::DirectoryEntry && self.list_attr.is_some()
    }

    pub fn is_leaf(&self) -> bool {
        self.kind == EntryKind::LeafEntry && self.list_attr.is_none()
    }

    pub fn is_leaflist(&self) -> bool {
        self.kind == EntryKind::LeafEntry && self.list_attr.is_some()
    }

    pub fn is_empty_leaf(&self) -> bool {
        if self.kind == EntryKind::LeafEntry {
            if let Some(n) = self.type_node.as_ref() {
                if n.kind == YangType::Empty {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_choice(&self) -> bool {
        self.kind == EntryKind::ChoiceEntry
    }

    pub fn is_action(&self) -> bool {
        self.kind == EntryKind::ActionEntry
    }
}

pub fn path_split(path: String) -> (String, String) {
    let paths: Vec<_> = path.split(':').collect();
    if paths.len() > 1 {
        (String::from(paths[0]), String::from(paths[1]))
    } else {
        (String::from(""), String::from(""))
    }
}

pub fn path_module(path: &str) -> Option<(String, String)> {
    let paths: Vec<_> = path.split(':').collect();
    if paths.len() == 2 {
        Some((paths[0].to_string(), paths[1].to_string()))
    } else {
        None
    }
}

pub fn to_entry(store: &YangStore, module: &ModuleNode) -> Rc<Entry> {
    let entry = Rc::new(Entry::new_dir(module.name.clone()));
    for c in module.d.container.iter() {
        container_entry(module, store, c, entry.clone());
    }
    for leaf in module.d.leaf.iter() {
        leaf_entry(module, store, leaf, entry.clone());
    }
    for list in module.d.list.iter() {
        list_entry(module, store, list, entry.clone());
    }
    for leaf_list in module.d.leaf_list.iter() {
        leaf_list_entry(module, store, leaf_list, entry.clone());
    }
    for choice in module.d.choice.iter() {
        choice_entry(module, store, choice, entry.clone());
    }

    // Apply YANG 1.1 §7.17 augment statements. Each augment may live
    // in the root module itself or in any loaded module — typically
    // a sibling module that imports the target. Walk all modules
    // once the primary tree is built so augment targets are
    // resolvable.
    for aug in module.augment.iter() {
        apply_augment(module, store, entry.clone(), aug);
    }
    for (name, m) in store.modules.iter() {
        if name == &module.name {
            continue;
        }
        for aug in m.augment.iter() {
            apply_augment(m, store, entry.clone(), aug);
        }
    }

    entry.clone()
}

/// Resolve the name of the module whose schema tree `target` roots
/// in, as seen from the augmenting module `top`. The leading segment
/// of an absolute schema-node-identifier carries the prefix that
/// binds the rest of the path to a module (RFC 7950 §6.5): the
/// augmenting module's own prefix maps to its own name, an imported
/// prefix maps to the imported module's name, and a bare (prefixless)
/// first segment defaults to the augmenting module itself.
fn augment_target_module<T>(top: &T, target: &str) -> String
where
    T: ModuleCommon,
{
    let first = target.split('/').find(|s| !s.is_empty()).unwrap_or("");
    match first.split_once(':') {
        Some((prefix, _)) => {
            if Some(prefix) == top.get_prefix() {
                top.get_name().to_string()
            } else {
                // Maps an imported prefix to its module name, or
                // returns the prefix unchanged when it binds to
                // nothing — in which case the caller's module-name
                // comparison fails and the augment is skipped.
                prefix_resolve(top, prefix.to_string())
            }
        }
        None => top.get_name().to_string(),
    }
}

/// Walk `target` from `root`, matching each `[prefix:]name` segment
/// by its bare name against the Entry tree (the tree carries no
/// per-node namespace, and names are effectively unique within a
/// container). Returns the resolved entry, or the first segment that
/// did not match so callers can report where the path broke.
fn resolve_target(root: Rc<Entry>, target: &str) -> Result<Rc<Entry>, String> {
    let mut current = root;
    for seg in target.split('/').filter(|s| !s.is_empty()) {
        let name = seg.rsplit(':').next().unwrap_or(seg);
        let next = current
            .dir
            .borrow()
            .iter()
            .find(|e| e.name == name)
            .cloned();
        match next {
            Some(e) => current = e,
            None => return Err(seg.to_string()),
        }
    }
    Ok(current)
}

/// Apply a top-level `augment` against the tree rooted at `root`.
///
/// `to_entry` applies every loaded module's augments against the tree
/// currently being built, so first check that this augment actually
/// targets this tree's module (`root.name`) before touching it; that
/// both prevents an augment from bleeding into an unrelated module
/// that happens to share a node name and keeps the not-found
/// diagnostic meaningful.
fn apply_augment<T>(top: &T, store: &YangStore, root: Rc<Entry>, aug: &AugmentNode)
where
    T: ModuleCommon,
{
    if augment_target_module(top, &aug.target) != root.name {
        return;
    }
    // RFC 7950 §7.17: a top-level augment's target is an
    // absolute-schema-nodeid (leading '/').
    if !aug.target.starts_with('/') {
        eprintln!(
            "augment: top-level target \"{}\" must use the absolute form (leading '/')",
            aug.target
        );
    }
    resolve_and_inject(top, store, root, aug, "augment");
}

/// Expand a `uses` into `ent`: instantiate the referenced grouping,
/// then apply any uses-substatement augments. Every site that expands
/// a `uses` (module/container/list bodies, choice cases, rpc/action
/// input and output) goes through here so uses-augments are applied
/// consistently.
fn uses_entry<T>(top: &T, store: &YangStore, uses: &UsesNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    group_resolve(top, store, &uses.name, ent.clone());
    for aug in uses.augment.iter() {
        apply_uses_augment(top, store, ent.clone(), aug);
    }
}

/// Apply a `uses`-substatement augment (RFC 7950 §7.17, descendant
/// form). The target path is relative to `ent`, the point where the
/// grouping was just expanded, so resolve from there directly. Unlike
/// a top-level augment there is no cross-module targeting to gate on —
/// the augment only ever reaches nodes the grouping contributed.
fn apply_uses_augment<T>(top: &T, store: &YangStore, ent: Rc<Entry>, aug: &AugmentNode)
where
    T: ModuleCommon,
{
    // RFC 7950 §7.17: a uses-substatement augment's target is a
    // descendant-schema-nodeid (no leading '/').
    if aug.target.starts_with('/') {
        eprintln!(
            "uses augment: target \"{}\" must use the descendant form (no leading '/')",
            aug.target
        );
    }
    resolve_and_inject(top, store, ent, aug, "uses augment");
}

/// Resolve `aug.target` from `root` and inject the augment body.
/// Shared by top-level and uses augments (`kind` only labels the
/// diagnostic). If the path resolves to a data node, inject there; if
/// the final segment instead names a choice (choices are flattened and
/// not addressable as entries), add the augment's cases to that choice.
fn resolve_and_inject<T>(top: &T, store: &YangStore, root: Rc<Entry>, aug: &AugmentNode, kind: &str)
where
    T: ModuleCommon,
{
    match resolve_target(root.clone(), &aug.target) {
        Ok(current) => inject_augment_body(top, store, current, aug),
        Err(seg) => {
            if !augment_into_choice(top, store, root, aug) {
                eprintln!(
                    "{kind}: in module {}, target \"{}\" not found (no node matching \"{}\")",
                    top.get_name(),
                    aug.target,
                    seg
                );
            }
        }
    }
}

/// Inject an augment body into an already-resolved data node: its
/// data-def children (RFC 7950 §7.17) and any `action` substatements
/// (valid when the target is a container or list). Explicit `case`
/// substatements are handled separately by `augment_into_choice` since
/// they only apply to choice targets, which are not data nodes.
fn inject_augment_body<T>(top: &T, store: &YangStore, current: Rc<Entry>, aug: &AugmentNode)
where
    T: ModuleCommon,
{
    // RFC 7950 §7.17: data nodes and actions may only be added to a
    // container/list/choice/case/input/output/notification — never to a
    // leaf or leaf-list. Resolution lands on a leaf only when the
    // augment is malformed.
    if current.is_leaf_entry() {
        eprintln!(
            "augment: cannot add nodes to leaf target \"{}\" ({})",
            current.name, aug.target
        );
        return;
    }

    // Snapshot existing child names so duplicates the augment introduces
    // can be rejected afterwards (RFC 7950 §7.17: an augment MUST NOT add
    // a node with the same name as one already present in the target).
    let existing: Vec<String> = current
        .dir
        .borrow()
        .iter()
        .map(|e| e.name.clone())
        .collect();
    let before_len = existing.len();

    datadef_entry(top, store, &aug.d, current.clone());
    for a in aug.action.iter() {
        action_entry(top, store, a, current.clone());
    }

    let mut dir = current.dir.borrow_mut();
    let mut i = before_len;
    while i < dir.len() {
        if existing.contains(&dir[i].name) {
            eprintln!(
                "augment: node \"{}\" already exists in target \"{}\"; not added",
                dir[i].name, current.name
            );
            dir.remove(i);
        } else {
            i += 1;
        }
    }
}

/// Handle an augment whose target is a choice. A choice node is not an
/// addressable entry — `choice_entry` flattens each case's children
/// into the choice's parent, tagged with the choice/case names — so
/// resolve the parent (every segment but the last) and treat the final
/// segment as the choice name. The choice is recognised via the
/// parent's recorded `choice_defs`, which lists every choice defined
/// there whether or not it has cases. Returns true if it handled the
/// augment.
fn augment_into_choice<T>(top: &T, store: &YangStore, root: Rc<Entry>, aug: &AugmentNode) -> bool
where
    T: ModuleCommon,
{
    let segs: Vec<&str> = aug.target.split('/').filter(|s| !s.is_empty()).collect();
    let Some((last, parents)) = segs.split_last() else {
        return false;
    };
    let choice_name = last.rsplit(':').next().unwrap_or(last);

    let mut parent = root;
    for seg in parents {
        let name = seg.rsplit(':').next().unwrap_or(seg);
        let next = parent.dir.borrow().iter().find(|e| e.name == name).cloned();
        match next {
            Some(e) => parent = e,
            None => return false,
        }
    }

    let is_choice = parent.choice_defs.borrow().iter().any(|n| n == choice_name);
    if !is_choice {
        return false;
    }

    // Explicit `case` substatements.
    for case in aug.cases.iter() {
        inject_case(top, store, parent.clone(), choice_name, &case.name, &case.d);
    }
    // Shorthand cases: each direct data node forms its own case named
    // after the node (RFC 7950 §7.9.2).
    for c in aug.d.container.iter() {
        inject_case_node(parent.clone(), choice_name, &c.name, |e| {
            container_entry(top, store, c, e)
        });
    }
    for leaf in aug.d.leaf.iter() {
        inject_case_node(parent.clone(), choice_name, &leaf.name, |e| {
            leaf_entry(top, store, leaf, e)
        });
    }
    for list in aug.d.list.iter() {
        inject_case_node(parent.clone(), choice_name, &list.name, |e| {
            list_entry(top, store, list, e)
        });
    }
    for leaf_list in aug.d.leaf_list.iter() {
        inject_case_node(parent.clone(), choice_name, &leaf_list.name, |e| {
            leaf_list_entry(top, store, leaf_list, e)
        });
    }
    true
}

/// Inject an explicit `case`'s data-def children into `ent` (the
/// choice's parent) and tag the newly added entries with the choice
/// and case names — the flattened representation used throughout for
/// choice/case membership.
fn inject_case<T>(
    top: &T,
    store: &YangStore,
    ent: Rc<Entry>,
    choice_name: &str,
    case_name: &str,
    d: &DatadefNode,
) where
    T: ModuleCommon,
{
    let before = ent.dir.borrow().len();
    datadef_entry(top, store, d, ent.clone());
    tag_case_children(&ent, before, choice_name, case_name);
}

/// Inject a single shorthand-case node via `build` and tag the entries
/// it added with the choice name and the node's own name (the implicit
/// case name for a shorthand case).
fn inject_case_node<F>(ent: Rc<Entry>, choice_name: &str, case_name: &str, build: F)
where
    F: FnOnce(Rc<Entry>),
{
    let before = ent.dir.borrow().len();
    build(ent.clone());
    tag_case_children(&ent, before, choice_name, case_name);
}

/// Tag entries appended to `ent.dir` at or after index `before` with
/// the given choice/case names, leaving any already tagged by a nested
/// choice recursion untouched (innermost wins).
fn tag_case_children(ent: &Rc<Entry>, before: usize, choice_name: &str, case_name: &str) {
    let dir = ent.dir.borrow();
    for child in dir[before..].iter() {
        if child.choice.borrow().is_none() {
            *child.choice.borrow_mut() = Some(choice_name.to_string());
            *child.case.borrow_mut() = Some(case_name.to_string());
        }
    }
}

/// Process a DatadefNode's children (uses, container, leaf, list,
/// leaf-list, choice) into `ent.dir`. Shared between groupings and
/// augments.
pub fn datadef_entry<T>(top: &T, store: &YangStore, d: &DatadefNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    for uses in d.uses.iter() {
        uses_entry(top, store, uses, ent.clone());
    }
    for c in d.container.iter() {
        container_entry(top, store, c, ent.clone());
    }
    for leaf in d.leaf.iter() {
        leaf_entry(top, store, leaf, ent.clone());
    }
    for list in d.list.iter() {
        list_entry(top, store, list, ent.clone());
    }
    for leaf_list in d.leaf_list.iter() {
        leaf_list_entry(top, store, leaf_list, ent.clone());
    }
    for choice in d.choice.iter() {
        choice_entry(top, store, choice, ent.clone());
    }
}

pub trait ModuleCommon {
    fn get_name(&self) -> &str;
    fn get_prefix(&self) -> Option<&str>;
    fn get_identity(&self) -> &Vec<IdentityNode>;
    fn get_identities_mut(&mut self) -> &mut HashMap<String, Vec<String>>;
    fn get_include(&self) -> &Vec<IncludeNode>;
    fn get_import(&self) -> &Vec<ImportNode>;
    fn get_typedef(&self) -> &Vec<TypedefNode>;
    fn get_grouping(&self) -> &Vec<GroupingNode>;
    fn get_d(&self) -> &DatadefNode;
}

pub fn group_entry<T>(top: &T, store: &YangStore, g: &GroupingNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    datadef_entry(top, store, &g.d, ent);
}

pub fn group_resolve<T>(top: &T, store: &YangStore, name: &str, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    if let Some((m, n)) = path_module(name) {
        let prefix = prefix_resolve(top, m);
        if let Some(m) = store.find_module(&prefix) {
            for g in m.grouping.iter() {
                if g.name == n {
                    group_entry(m, store, g, ent);
                    return;
                }
            }
        }
    } else {
        for g in top.get_grouping().iter() {
            if g.name == name {
                group_entry(top, store, g, ent);
                return;
            }
        }
        for include in top.get_include().iter() {
            if let Some(submodule) = store.find_submodule(&include.name) {
                for g in submodule.grouping.iter() {
                    if g.name == *name {
                        group_entry(submodule, store, g, ent);
                        return;
                    }
                }
            }
        }
    }
}

pub fn identity_resolve<T>(top: &mut T)
where
    T: ModuleCommon,
{
    // Iterate over node.identity and collect the necessary information
    let mut identity_bases: HashMap<String, Vec<String>> = HashMap::new();
    {
        let identity = top.get_identity();
        for identity in identity.iter() {
            if !identity.base.is_empty() {
                for base in &identity.base {
                    if let Some((_module, _name)) = path_module(base) {
                        // TODO: Resolve only local identity reference
                    } else {
                        for i in top.get_identity().iter() {
                            if base == &i.name {
                                let items = identity_bases.entry(base.clone()).or_default();
                                items.push(identity.name.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    for (key, value) in identity_bases.iter() {
        top.get_identities_mut().insert(key.clone(), value.to_vec());
    }
}

fn prefix_resolve<T>(node: &T, name: String) -> String
where
    T: ModuleCommon,
{
    for import in node.get_import().iter() {
        if let Some(prefix) = &import.prefix {
            if name == *prefix {
                return import.name.clone();
            }
        }
    }
    name
}

fn type_union_resolve<T>(top: &T, store: &YangStore, type_node: &TypeNode) -> Option<TypeNode>
where
    T: ModuleCommon,
{
    let mut nodes = Vec::<TypeNode>::new();
    for node in type_node.union.iter() {
        if node.kind == YangType::Path {
            if let Some(n) = type_path_resolve(top, store, node) {
                if n.kind == YangType::String {
                    let mut m = n.clone();
                    m.typedef = Some(node.name.clone());
                    nodes.push(m);
                }
                if n.kind == YangType::Union {
                    for m in n.union.iter() {
                        if m.kind == YangType::Path {
                            if let Some(o) = type_path_resolve(top, store, m) {
                                if o.kind == YangType::String {
                                    let mut o = o.clone();
                                    o.typedef = Some(m.name.clone());
                                    nodes.push(o);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Inline union arm with a recognized kind (e.g.
            // `type string { pattern '...'; }` written directly
            // inside the union, not via a typedef reference). Keep
            // it so the matcher can dispatch on it; the previous
            // drop-on-the-floor behavior was the reason inline
            // pattern-restricted string arms in unions never engaged.
            nodes.push(node.clone());
        }
    }
    let mut type_node = type_node.clone();
    type_node.union = nodes;
    Some(type_node)
}

fn type_path_resolve<T>(top: &T, store: &YangStore, type_node: &TypeNode) -> Option<TypeNode>
where
    T: ModuleCommon,
{
    if let Some((module, name)) = path_module(&type_node.name) {
        let prefix = prefix_resolve(top, module);
        let module = store.find_module(&prefix);
        if let Some(m) = module {
            for typedef in m.typedef.iter() {
                if typedef.name == name {
                    if let Some(node) = &typedef.type_node {
                        let mut node = node.clone();
                        node.typedef = Some(type_node.name.clone());
                        if node.kind == YangType::Union {
                            return type_union_resolve(top, store, &node);
                        } else {
                            return Some(node);
                        }
                    }
                }
            }
        }
    } else {
        for typedef in top.get_typedef().iter() {
            if typedef.name == type_node.name {
                if let Some(node) = &typedef.type_node {
                    let mut node = node.clone();
                    node.typedef = Some(type_node.name.clone());
                    if node.kind == YangType::Union {
                        // A local typedef whose underlying type is a
                        // union: resolve its Path arms the same way
                        // the prefixed branch does, otherwise a leaf
                        // like `type peer-id-or-all` reaches the
                        // matcher with every arm still `kind = Path`
                        // and nothing dispatches.
                        return type_union_resolve(top, store, &node);
                    }
                    return Some(node);
                }
            }
        }
    }
    None
}

fn type_resolve<T>(top: &T, store: &YangStore, type_node: &TypeNode, ent: &mut Entry)
where
    T: ModuleCommon,
{
    if type_node.kind == YangType::Path {
        if let Some(node) = type_path_resolve(top, store, type_node) {
            ent.type_node = Some(node);
        }
    } else if type_node.kind == YangType::Identityref {
        if let Some(base) = &type_node.base {
            if let Some((module, name)) = path_module(base) {
                let prefix = prefix_resolve(top, module);
                let module = store.find_module(&prefix);
                if let Some(m) = module {
                    let mut node = type_node.clone();
                    node.kind = YangType::Enumeration;
                    if let Some(identities) = m.identities.get(&name) {
                        for i in identities.iter() {
                            node.enum_stmt.push(EnumNode { name: i.clone() });
                        }
                    }
                    ent.type_node = Some(node);
                    return;
                } else {
                    // println!("XXX: module not found {}", name);
                }
            } else {
                // println!("XXX: self {}", base);
            }
        }
        ent.type_node = Some(type_node.clone());
    } else if type_node.kind == YangType::Union {
        let mut union_node = TypeNode::new(type_node.name.clone(), YangType::Union);
        for node in type_node.union.iter() {
            if node.kind == YangType::Path {
                if let Some(node) = type_path_resolve(top, store, node) {
                    union_node.union.push(node);
                }
            } else {
                // Inline arm with a recognized kind (e.g. `type uint32;`
                // or `type string { pattern '...'; }` written directly
                // inside the union, not via a typedef reference). Keep
                // it so the matcher can dispatch on it; otherwise inline
                // scalar / patterned-string arms silently disappear and
                // a union like `union { uint32; inet:ipv4-address; }`
                // only matches the ipv4-address arm.
                union_node.union.push(node.clone());
            }
        }
        ent.type_node = Some(union_node);
    } else {
        ent.type_node = Some(type_node.clone());
    }
}

pub fn action_entry<T>(top: &T, store: &YangStore, a: &ActionNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    let e = Entry::new_action(a.name.clone());
    let rc = Rc::new(e);

    // Process input parameters if present
    if let Some(input) = &a.input {
        let mut input_entry = Entry::new_dir("input".to_string());
        input_entry
            .extension
            .insert("input".to_string(), "true".to_string());
        let input_rc = Rc::new(input_entry);

        // Process input data definitions
        for uses in input.d.uses.iter() {
            uses_entry(top, store, uses, input_rc.clone());
        }
        for c in input.d.container.iter() {
            container_entry(top, store, c, input_rc.clone());
        }
        for leaf in input.d.leaf.iter() {
            leaf_entry(top, store, leaf, input_rc.clone());
        }
        for list in input.d.list.iter() {
            list_entry(top, store, list, input_rc.clone());
        }
        for leaf_list in input.d.leaf_list.iter() {
            leaf_list_entry(top, store, leaf_list, input_rc.clone());
        }
        for choice in input.d.choice.iter() {
            choice_entry(top, store, choice, input_rc.clone());
        }

        rc.dir.borrow_mut().push(input_rc.clone());
        input_rc.parent.replace(Some(rc.clone()));
    }

    // Process output parameters if present
    if let Some(output) = &a.output {
        let mut output_entry = Entry::new_dir("output".to_string());
        output_entry
            .extension
            .insert("output".to_string(), "true".to_string());
        let output_rc = Rc::new(output_entry);

        // Process output data definitions
        for uses in output.d.uses.iter() {
            uses_entry(top, store, uses, output_rc.clone());
        }
        for c in output.d.container.iter() {
            container_entry(top, store, c, output_rc.clone());
        }
        for leaf in output.d.leaf.iter() {
            leaf_entry(top, store, leaf, output_rc.clone());
        }
        for list in output.d.list.iter() {
            list_entry(top, store, list, output_rc.clone());
        }
        for leaf_list in output.d.leaf_list.iter() {
            leaf_list_entry(top, store, leaf_list, output_rc.clone());
        }
        for choice in output.d.choice.iter() {
            choice_entry(top, store, choice, output_rc.clone());
        }

        rc.dir.borrow_mut().push(output_rc.clone());
        output_rc.parent.replace(Some(rc.clone()));
    }

    ent.dir.borrow_mut().push(rc.clone());
    rc.parent.replace(Some(ent.clone()));
}

pub fn choice_entry<T>(top: &T, store: &YangStore, c: &ChoiceNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    if let Some(config) = &c.config {
        if !config.config {
            return;
        }
    }

    // Record the choice name on its parent so it stays addressable for
    // augments even before (or without) any case contributing children.
    ent.choice_defs.borrow_mut().push(c.name.clone());

    // Per RFC 7950 §7.9.2, neither the `choice` node nor its `case`
    // nodes appear in the data tree — only the case's direct data
    // children do. We flatten each case's children into the choice's
    // parent `ent.dir` and tag each added entry with (choice, case)
    // metadata so consumers can enforce mutual exclusion later. The
    // same flattening is reused by `augment_into_choice`.
    for case in c.cases.iter() {
        inject_case(top, store, ent.clone(), &c.name, &case.name, &case.d);
    }
}

pub fn container_entry<T>(top: &T, store: &YangStore, c: &ContainerNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    if let Some(config) = &c.config {
        if !config.config {
            return;
        }
    }
    let mut e = Entry::new_dir(c.name.clone());
    for u in c.unknown.iter() {
        e.extension.insert(u.name.clone(), u.argument.clone());
    }
    e.presence = c.presence.is_some();
    let rc = Rc::new(e);

    for uses in c.d.uses.iter() {
        uses_entry(top, store, uses, rc.clone());
    }
    for c in c.d.container.iter() {
        container_entry(top, store, c, rc.clone());
    }
    for leaf in c.d.leaf.iter() {
        leaf_entry(top, store, leaf, rc.clone());
    }
    for list in c.d.list.iter() {
        list_entry(top, store, list, rc.clone());
    }
    for leaf_list in c.d.leaf_list.iter() {
        leaf_list_entry(top, store, leaf_list, rc.clone());
    }
    for choice in c.d.choice.iter() {
        choice_entry(top, store, choice, rc.clone());
    }
    for action in c.action.iter() {
        action_entry(top, store, action, rc.clone());
    }

    ent.dir.borrow_mut().push(rc.clone());
    rc.parent.replace(Some(ent.clone()));
}

fn list_entry<T>(top: &T, store: &YangStore, l: &ListNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    if let Some(config) = &l.config {
        if !config.config {
            return;
        }
    }
    let mut e = Entry::new_list(l.name.clone(), l.key.keys.clone());
    for u in l.unknown.iter() {
        if u.name == "ext:presence" {
            e.presence = true;
        }
        e.extension.insert(u.name.clone(), u.argument.clone());
    }
    let list_attr = ListAttr::new();
    e.list_attr = Some(list_attr);
    let rc = Rc::new(e);

    for uses in l.d.uses.iter() {
        uses_entry(top, store, uses, rc.clone());
    }
    for c in l.d.container.iter() {
        container_entry(top, store, c, rc.clone());
    }
    for leaf in l.d.leaf.iter() {
        leaf_entry(top, store, leaf, rc.clone());
    }
    for list in l.d.list.iter() {
        list_entry(top, store, list, rc.clone());
    }
    for leaf_list in l.d.leaf_list.iter() {
        leaf_list_entry(top, store, leaf_list, rc.clone());
    }
    for choice in l.d.choice.iter() {
        choice_entry(top, store, choice, rc.clone());
    }

    ent.dir.borrow_mut().push(rc.clone());
    rc.parent.replace(Some(ent.clone()));
}

fn leaf_entry<T>(top: &T, store: &YangStore, leaf: &LeafNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    if let Some(config) = &leaf.config {
        if !config.config {
            return;
        }
    }
    let mut e = Entry::new_leaf(leaf.name.to_owned());
    e.mandatory = leaf.is_mandatory();
    for u in leaf.unknown.iter() {
        e.extension.insert(u.name.clone(), u.argument.clone());
    }
    if let Some(t) = leaf.type_stmt.as_ref() {
        type_resolve(top, store, t, &mut e);
    }
    let rc = Rc::new(e);
    ent.dir.borrow_mut().push(rc.clone());
    rc.parent.replace(Some(ent.clone()));
}

fn leaf_list_entry<T>(top: &T, store: &YangStore, leaf: &LeafListNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    if let Some(config) = &leaf.config {
        if !config.config {
            return;
        }
    }
    let mut e = Entry::new_leaf(leaf.name.clone());
    for u in leaf.unknown.iter() {
        e.extension.insert(u.name.clone(), u.argument.clone());
    }
    if let Some(t) = leaf.type_stmt.as_ref() {
        type_resolve(top, store, t, &mut e);
    }
    let list_attr = ListAttr::new();
    e.list_attr = Some(list_attr);

    let rc = Rc::new(e);
    ent.dir.borrow_mut().push(rc.clone());
    rc.parent.replace(Some(ent.clone()));
}

impl ModuleCommon for ModuleNode {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    fn get_identity(&self) -> &Vec<IdentityNode> {
        &self.identity
    }

    fn get_identities_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.identities
    }

    fn get_include(&self) -> &Vec<IncludeNode> {
        &self.include
    }

    fn get_import(&self) -> &Vec<ImportNode> {
        &self.import
    }

    fn get_typedef(&self) -> &Vec<TypedefNode> {
        &self.typedef
    }

    fn get_grouping(&self) -> &Vec<GroupingNode> {
        &self.grouping
    }

    fn get_d(&self) -> &DatadefNode {
        &self.d
    }
}

impl ModuleCommon for SubmoduleNode {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_prefix(&self) -> Option<&str> {
        // A submodule has no prefix of its own; per RFC 7950 §7.2.2 it
        // shares the prefix of the module it belongs to.
        self.belongs_to.as_ref().and_then(|b| b.prefix.as_deref())
    }

    fn get_identity(&self) -> &Vec<IdentityNode> {
        &self.identity
    }

    fn get_identities_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.identities
    }

    fn get_include(&self) -> &Vec<IncludeNode> {
        &self.include
    }

    fn get_import(&self) -> &Vec<ImportNode> {
        &self.import
    }

    fn get_typedef(&self) -> &Vec<TypedefNode> {
        &self.typedef
    }

    fn get_grouping(&self) -> &Vec<GroupingNode> {
        &self.grouping
    }

    fn get_d(&self) -> &DatadefNode {
        &self.d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> Rc<Entry> {
        Rc::new(Entry::new_dir(name.to_string()))
    }

    #[test]
    fn resolve_target_walks_prefixed_path_by_bare_name() {
        let root = dir("m");
        let top = dir("top");
        let item = dir("item");
        top.dir.borrow_mut().push(item);
        root.dir.borrow_mut().push(top);

        // Each `prefix:name` segment is matched on its bare name.
        let found = resolve_target(root, "/a:top/a:item").expect("resolved");
        assert_eq!(found.name, "item");
    }

    #[test]
    fn resolve_target_reports_first_missing_segment() {
        let root = dir("m");
        root.dir.borrow_mut().push(dir("top"));

        let err = resolve_target(root, "/a:top/a:missing").unwrap_err();
        assert_eq!(err, "a:missing");
    }
}
