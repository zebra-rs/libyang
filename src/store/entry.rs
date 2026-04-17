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

/// Walk `aug.target` from `root` and inject `aug.d`'s children at
/// the resolved node. Silently no-ops if the target cannot be found
/// (malformed augment, missing prefix binding, etc.).
fn apply_augment<T>(top: &T, store: &YangStore, root: Rc<Entry>, aug: &AugmentNode)
where
    T: ModuleCommon,
{
    let mut current = root;
    for seg in aug.target.split('/').filter(|s| !s.is_empty()) {
        // YANG schema-node identifier segments are `[prefix:]name`.
        // We match on the bare name against the Entry tree — the
        // tree doesn't carry per-node namespace metadata, and names
        // are effectively globally unique within their container.
        let name = seg.rsplit(':').next().unwrap_or(seg);
        let next = current
            .dir
            .borrow()
            .iter()
            .find(|e| e.name == name)
            .cloned();
        match next {
            Some(e) => current = e,
            None => return,
        }
    }
    datadef_entry(top, store, &aug.d, current);
}

/// Process a DatadefNode's children (uses, container, leaf, list,
/// leaf-list, choice) into `ent.dir`. Shared between groupings and
/// augments.
pub fn datadef_entry<T>(top: &T, store: &YangStore, d: &DatadefNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    for uses in d.uses.iter() {
        group_resolve(top, store, &uses.name, ent.clone());
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
                    return Some(node.clone());
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
            group_resolve(top, store, &uses.name, input_rc.clone());
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
            group_resolve(top, store, &uses.name, output_rc.clone());
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

    // Per RFC 7950 §7.9.2, neither the `choice` node nor its `case`
    // nodes appear in the data tree — only the case's direct data
    // children do. We flatten each case's children into the choice's
    // parent `ent.dir` and tag each added entry with (choice, case)
    // metadata so consumers can enforce mutual exclusion later.
    let choice_name = c.name.clone();

    for case in c.cases.iter() {
        let case_name = case.name.clone();
        let len_before = ent.dir.borrow().len();

        for uses in case.d.uses.iter() {
            group_resolve(top, store, &uses.name, ent.clone());
        }
        for cnode in case.d.container.iter() {
            container_entry(top, store, cnode, ent.clone());
        }
        for leaf in case.d.leaf.iter() {
            leaf_entry(top, store, leaf, ent.clone());
        }
        for list in case.d.list.iter() {
            list_entry(top, store, list, ent.clone());
        }
        for leaf_list in case.d.leaf_list.iter() {
            leaf_list_entry(top, store, leaf_list, ent.clone());
        }
        for nested in case.d.choice.iter() {
            choice_entry(top, store, nested, ent.clone());
        }

        // Tag the newly added entries. Skip any already tagged by a
        // nested `choice_entry` recursion — the innermost choice/case
        // wins (outer-nesting metadata is not currently represented).
        let dir = ent.dir.borrow();
        for child in dir[len_before..].iter() {
            if child.choice.borrow().is_none() {
                *child.choice.borrow_mut() = Some(choice_name.clone());
                *child.case.borrow_mut() = Some(case_name.clone());
            }
        }
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
        group_resolve(top, store, &uses.name, rc.clone());
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
        group_resolve(top, store, &uses.name, rc.clone());
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
