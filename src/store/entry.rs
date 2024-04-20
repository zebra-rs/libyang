use crate::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, PartialEq, Default)]
pub enum EntryKind {
    #[default]
    LeafEntry,
    DirectoryEntry,
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
    pub dir: RefCell<Vec<Rc<Entry>>>,
    pub key: Vec<String>,
    pub extension: HashMap<String, String>,
    pub parent: RefCell<Option<Rc<Entry>>>,
    pub type_node: Option<TypeNode>,
    //pub typedef: Option<String>,
    pub list_attr: Option<ListAttr>,
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

impl ModuleNode {
    pub fn identity_resolve(&mut self) {
        for identity in self.identity.iter() {
            if identity.base.is_empty() {
                self.identities.insert(identity.name.clone(), Vec::new());
            } else {
                for base in identity.base.iter() {
                    if let Some((_module, _name)) = path_module(base) {
                        // TODO: println!("B: {} : {}", module, name);
                    } else {
                        for i in self.identity.iter() {
                            if base == &i.name {
                                if let Some(identities) = self.identities.get_mut(base) {
                                    identities.push(identity.name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn prefix_resolve(&self, name: String) -> String {
        for import in self.import.iter() {
            if let Some(prefix) = &import.prefix {
                if name == *prefix {
                    return import.name.clone();
                }
            }
        }
        name
    }

    pub fn group_resolve(&self, store: &YangStore, name: &str, ent: Rc<Entry>) {
        if let Some((m, n)) = path_module(name) {
            let prefix = self.prefix_resolve(m);
            if let Some(m) = store.find_module(&prefix) {
                for g in m.grouping.iter() {
                    if g.name == n {
                        m.group_entry(store, g, ent);
                        return;
                    }
                }
            }
        } else {
            for g in self.grouping.iter() {
                if g.name == name {
                    self.group_entry(store, g, ent);
                    return;
                }
            }
            for include in self.include.iter() {
                if let Some(submodule) = store.find_submodule(&include.name) {
                    for g in submodule.grouping.iter() {
                        if g.name == *name {
                            submodule.group_entry(store, g, ent);
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn group_entry(&self, store: &YangStore, g: &GroupingNode, ent: Rc<Entry>) {
        for uses in g.d.uses.iter() {
            self.group_resolve(store, &uses.name, ent.clone());
        }
        for c in g.d.container.iter() {
            self.container_entry(store, c, ent.clone());
        }
        for leaf in g.d.leaf.iter() {
            self.leaf_entry(store, leaf, ent.clone());
        }
        for list in g.d.list.iter() {
            self.list_entry(store, list, ent.clone());
        }
        for leaf_list in g.d.leaf_list.iter() {
            self.leaf_list_entry(store, leaf_list, ent.clone());
        }
    }

    pub fn container_entry(&self, store: &YangStore, c: &ContainerNode, ent: Rc<Entry>) {
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
            self.group_resolve(store, &uses.name, rc.clone());
        }
        for c in c.d.container.iter() {
            self.container_entry(store, c, rc.clone());
        }
        for leaf in c.d.leaf.iter() {
            self.leaf_entry(store, leaf, rc.clone());
        }
        for list in c.d.list.iter() {
            self.list_entry(store, list, rc.clone());
        }
        for leaf_list in c.d.leaf_list.iter() {
            self.leaf_list_entry(store, leaf_list, rc.clone());
        }

        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }

    fn list_entry(&self, store: &YangStore, l: &ListNode, ent: Rc<Entry>) {
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
            self.group_resolve(store, &uses.name, rc.clone());
        }
        for c in l.d.container.iter() {
            self.container_entry(store, c, rc.clone());
        }
        for leaf in l.d.leaf.iter() {
            self.leaf_entry(store, leaf, rc.clone());
        }
        for list in l.d.list.iter() {
            self.list_entry(store, list, rc.clone());
        }
        for leaf_list in l.d.leaf_list.iter() {
            self.leaf_list_entry(store, leaf_list, rc.clone());
        }

        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }

    fn type_union_resolve(&self, store: &YangStore, type_node: &TypeNode) -> Option<TypeNode> {
        let mut nodes = Vec::<TypeNode>::new();
        for node in type_node.union.iter() {
            if node.kind == YangType::Path {
                if let Some(n) = self.type_path_resolve(store, node) {
                    if n.kind == YangType::String {
                        let mut m = n.clone();
                        m.typedef = Some(node.name.clone());
                        nodes.push(m);
                    }
                    if n.kind == YangType::Union {
                        for m in n.union.iter() {
                            if m.kind == YangType::Path {
                                if let Some(o) = self.type_path_resolve(store, m) {
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

    fn type_path_resolve(&self, store: &YangStore, type_node: &TypeNode) -> Option<TypeNode> {
        if let Some((module, name)) = path_module(&type_node.name) {
            let prefix = self.prefix_resolve(module);
            let module = store.find_module(&prefix);
            if let Some(m) = module {
                for typedef in m.typedef.iter() {
                    if typedef.name == name {
                        if let Some(node) = &typedef.type_node {
                            let mut node = node.clone();
                            node.typedef = Some(type_node.name.clone());
                            if node.kind == YangType::Union {
                                return self.type_union_resolve(store, &node);
                            } else {
                                return Some(node);
                            }
                        }
                    }
                }
            }
        } else {
            for typedef in self.typedef.iter() {
                if typedef.name == type_node.name {
                    if let Some(node) = &typedef.type_node {
                        return Some(node.clone());
                    }
                }
            }
        }
        None
    }

    fn type_resolve(&self, store: &YangStore, type_node: &TypeNode, ent: &mut Entry) {
        if type_node.kind == YangType::Path {
            if let Some(node) = self.type_path_resolve(store, type_node) {
                ent.type_node = Some(node);
            }
        } else if type_node.kind == YangType::Identityref {
            if let Some(base) = &type_node.base {
                if let Some((module, name)) = path_module(&base) {
                    let prefix = self.prefix_resolve(module);
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
            for node in type_node.union.iter() {
                if node.kind == YangType::Path {
                    println!("X: {} ({})", node.name, self.name);
                    if let Some(node) = self.type_path_resolve(store, node) {
                        println!("X: found {}", node.name);
                    }
                }
            }
        } else {
            ent.type_node = Some(type_node.clone());
        }
    }

    fn leaf_entry(&self, store: &YangStore, leaf: &LeafNode, ent: Rc<Entry>) {
        if let Some(config) = &leaf.config {
            if !config.config {
                return;
            }
        }
        let mut e = Entry::new_leaf(leaf.name.to_owned());
        for u in leaf.unknown.iter() {
            e.extension.insert(u.name.clone(), u.argument.clone());
        }
        if let Some(t) = leaf.type_stmt.as_ref() {
            self.type_resolve(store, t, &mut e);
        }
        let rc = Rc::new(e);
        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }

    fn leaf_list_entry(&self, store: &YangStore, leaf: &LeafListNode, ent: Rc<Entry>) {
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
            self.type_resolve(store, t, &mut e);
        }
        let list_attr = ListAttr::new();
        e.list_attr = Some(list_attr);

        let rc = Rc::new(e);
        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }
}

impl SubmoduleNode {
    pub fn identity_resolve(&mut self) {
        for identity in self.identity.iter() {
            if identity.base.is_empty() {
                self.identities.insert(identity.name.clone(), Vec::new());
            } else {
                for base in identity.base.iter() {
                    if let Some((_module, _name)) = path_module(base) {
                        // TODO: println!("B: {} : {}", module, name);
                    } else {
                        for i in self.identity.iter() {
                            if base == &i.name {
                                if let Some(identities) = self.identities.get_mut(base) {
                                    identities.push(identity.name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn prefix_resolve(&self, name: String) -> String {
        for import in self.import.iter() {
            if let Some(prefix) = &import.prefix {
                if name == *prefix {
                    return import.name.clone();
                }
            }
        }
        name
    }

    pub fn group_resolve(&self, store: &YangStore, name: &str, ent: Rc<Entry>) {
        if let Some((m, n)) = path_module(name) {
            let prefix = self.prefix_resolve(m);
            if let Some(m) = store.find_module(&prefix) {
                for g in m.grouping.iter() {
                    if g.name == n {
                        m.group_entry(store, g, ent);
                        return;
                    }
                }
            }
        } else {
            for g in self.grouping.iter() {
                if g.name == name {
                    self.group_entry(store, g, ent);
                    return;
                }
            }
            for include in self.include.iter() {
                if let Some(submodule) = store.find_submodule(&include.name) {
                    for g in submodule.grouping.iter() {
                        if g.name == *name {
                            submodule.group_entry(store, g, ent);
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn group_entry(&self, store: &YangStore, g: &GroupingNode, ent: Rc<Entry>) {
        for uses in g.d.uses.iter() {
            self.group_resolve(store, &uses.name, ent.clone());
        }
        for c in g.d.container.iter() {
            self.container_entry(store, c, ent.clone());
        }
        for leaf in g.d.leaf.iter() {
            self.leaf_entry(store, leaf, ent.clone());
        }
        for list in g.d.list.iter() {
            self.list_entry(store, list, ent.clone());
        }
        for leaf_list in g.d.leaf_list.iter() {
            self.leaf_list_entry(store, leaf_list, ent.clone());
        }
    }

    pub fn container_entry(&self, store: &YangStore, c: &ContainerNode, ent: Rc<Entry>) {
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
            self.group_resolve(store, &uses.name, rc.clone());
        }
        for c in c.d.container.iter() {
            self.container_entry(store, c, rc.clone());
        }
        for leaf in c.d.leaf.iter() {
            self.leaf_entry(store, leaf, rc.clone());
        }
        for list in c.d.list.iter() {
            self.list_entry(store, list, rc.clone());
        }
        for leaf_list in c.d.leaf_list.iter() {
            self.leaf_list_entry(store, leaf_list, rc.clone());
        }

        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }

    fn list_entry(&self, store: &YangStore, l: &ListNode, ent: Rc<Entry>) {
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
            self.group_resolve(store, &uses.name, rc.clone());
        }
        for c in l.d.container.iter() {
            self.container_entry(store, c, rc.clone());
        }
        for leaf in l.d.leaf.iter() {
            self.leaf_entry(store, leaf, rc.clone());
        }
        for list in l.d.list.iter() {
            self.list_entry(store, list, rc.clone());
        }
        for leaf_list in l.d.leaf_list.iter() {
            self.leaf_list_entry(store, leaf_list, rc.clone());
        }

        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }

    fn type_union_resolve(&self, store: &YangStore, type_node: &TypeNode) -> Option<TypeNode> {
        let mut nodes = Vec::<TypeNode>::new();
        for node in type_node.union.iter() {
            if node.kind == YangType::Path {
                if let Some(node) = self.type_path_resolve(store, node) {
                    if node.kind == YangType::Union {
                        for n in node.union.iter() {
                            if n.kind == YangType::Path {
                                if let Some(m) = self.type_path_resolve(store, n) {
                                    if m.kind == YangType::String {
                                        let mut m = m.clone();
                                        m.typedef = Some(n.name.clone());
                                        nodes.push(m);
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

    fn type_path_resolve(&self, store: &YangStore, type_node: &TypeNode) -> Option<TypeNode> {
        if let Some((module, name)) = path_module(&type_node.name) {
            let prefix = self.prefix_resolve(module);
            let module = store.find_module(&prefix);
            if let Some(m) = module {
                for typedef in m.typedef.iter() {
                    if typedef.name == name {
                        if let Some(node) = &typedef.type_node {
                            let mut node = node.clone();
                            node.typedef = Some(type_node.name.clone());
                            if node.kind == YangType::Union {
                                return self.type_union_resolve(store, &node);
                            } else {
                                return Some(node);
                            }
                        }
                    }
                }
            }
        } else {
            for typedef in self.typedef.iter() {
                if typedef.name == type_node.name {
                    if let Some(node) = &typedef.type_node {
                        return Some(node.clone());
                    }
                }
            }
        }
        None
    }

    fn type_resolve(&self, store: &YangStore, type_node: &TypeNode, ent: &mut Entry) {
        if type_node.kind == YangType::Path {
            if let Some(node) = self.type_path_resolve(store, type_node) {
                ent.type_node = Some(node);
            }
        } else if type_node.kind == YangType::Identityref {
            if let Some(base) = &type_node.base {
                if let Some((module, name)) = path_module(&base) {
                    let prefix = self.prefix_resolve(module);
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
            ent.type_node = self.type_union_resolve(store, type_node);
        } else {
            ent.type_node = Some(type_node.clone());
        }
    }

    fn leaf_entry(&self, store: &YangStore, leaf: &LeafNode, ent: Rc<Entry>) {
        if let Some(config) = &leaf.config {
            if !config.config {
                return;
            }
        }
        let mut e = Entry::new_leaf(leaf.name.to_owned());
        for u in leaf.unknown.iter() {
            e.extension.insert(u.name.clone(), u.argument.clone());
        }
        if let Some(t) = leaf.type_stmt.as_ref() {
            self.type_resolve(store, t, &mut e);
        }
        let rc = Rc::new(e);
        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }

    fn leaf_list_entry(&self, store: &YangStore, leaf: &LeafListNode, ent: Rc<Entry>) {
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
            self.type_resolve(store, t, &mut e);
        }
        let list_attr = ListAttr::new();
        e.list_attr = Some(list_attr);

        let rc = Rc::new(e);
        ent.dir.borrow_mut().push(rc.clone());
        rc.parent.replace(Some(ent.clone()));
    }
}

pub fn to_entry(store: &YangStore, module: &ModuleNode) -> Rc<Entry> {
    let entry = Rc::new(Entry::new_dir(String::from("")));
    for c in module.d.container.iter() {
        module.container_entry(store, c, entry.clone());
    }
    for leaf in module.d.leaf.iter() {
        module.leaf_entry(store, leaf, entry.clone());
    }
    for list in module.d.list.iter() {
        module.list_entry(store, list, entry.clone());
    }
    for leaf_list in module.d.leaf_list.iter() {
        module.leaf_list_entry(store, leaf_list, entry.clone());
    }
    entry.clone()
}
