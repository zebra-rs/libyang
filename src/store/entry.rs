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
    entry.clone()
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
    for uses in g.d.uses.iter() {
        group_resolve(top, store, &uses.name, ent.clone());
    }
    for c in g.d.container.iter() {
        container_entry(top, store, c, ent.clone());
    }
    for leaf in g.d.leaf.iter() {
        leaf_entry(top, store, leaf, ent.clone());
    }
    for list in g.d.list.iter() {
        list_entry(top, store, list, ent.clone());
    }
    for leaf_list in g.d.leaf_list.iter() {
        leaf_list_entry(top, store, leaf_list, ent.clone());
    }
    for choice in g.d.choice.iter() {
        choice_entry(top, store, choice, ent.clone());
    }
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

pub fn choice_entry<T>(top: &T, store: &YangStore, c: &ChoiceNode, ent: Rc<Entry>)
where
    T: ModuleCommon,
{
    if let Some(config) = &c.config {
        if !config.config {
            return;
        }
    }
    let mut e = Entry::new_choice(c.name.clone());
    e.mandatory = c.mandatory.as_ref().map(|m| m.mandatory).unwrap_or(false);
    
    // Collect all cases first
    let mut cases = Vec::new();
    
    // Process each case in the choice
    for case in c.cases.iter() {
        let mut case_entry = Entry::new_dir(case.name.clone());
        case_entry.extension.insert("case".to_string(), "true".to_string());
        let case_rc = Rc::new(case_entry);

        // Process data definitions within the case
        for uses in case.d.uses.iter() {
            group_resolve(top, store, &uses.name, case_rc.clone());
        }
        for c in case.d.container.iter() {
            container_entry(top, store, c, case_rc.clone());
        }
        for leaf in case.d.leaf.iter() {
            leaf_entry(top, store, leaf, case_rc.clone());
        }
        for list in case.d.list.iter() {
            list_entry(top, store, list, case_rc.clone());
        }
        for leaf_list in case.d.leaf_list.iter() {
            leaf_list_entry(top, store, leaf_list, case_rc.clone());
        }
        for choice in case.d.choice.iter() {
            choice_entry(top, store, choice, case_rc.clone());
        }

        cases.push(case_rc);
    }
    
    // Set the cases
    e.choice_cases = Some(cases.clone());
    let rc = Rc::new(e);
    
    // Set parent references
    for case_rc in cases {
        case_rc.parent.replace(Some(rc.clone()));
    }

    ent.dir.borrow_mut().push(rc.clone());
    rc.parent.replace(Some(ent.clone()));
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
