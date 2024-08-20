use crate::yang_grammar::YangGrammar;
use crate::yang_grammar_trait::*;
use crate::*;
use std::str::FromStr;

pub fn yang(y: YangGrammar) -> Result<Node, YangError> {
    match y.yang {
        Some(y) => match y {
            Yang::ModuleStmt(m) => {
                let name = identifier_arg_str(&m.module_stmt.identifier_arg_str);
                let mut node = ModuleNode::new(name);
                module(&mut node, m);
                Ok(Node::Module(Box::new(node)))
            }
            Yang::SubmoduleStmt(m) => {
                let name = identifier_arg_str(&m.submodule_stmt.identifier_arg_str);
                let mut node = SubmoduleNode::new(name);
                submodule(&mut node, m);
                Ok(Node::Submodule(Box::new(node)))
            }
        },
        None => Err(YangError::ParseError),
    }
}

fn module(node: &mut ModuleNode, m: YangModuleStmt) {
    for m in m.module_stmt.module_stmt_list.iter() {
        match &*m.module_header_stmts {
            ModuleHeaderStmts::YangVersionStmt(m) => {
                node.yang_version(m);
            }
            ModuleHeaderStmts::NamespaceStmt(m) => {
                node.namespace(m);
            }
            ModuleHeaderStmts::PrefixStmt(m) => {
                node.prefix(m);
            }
        }
    }
    for m in m.module_stmt.module_stmt_list0.iter() {
        match &*m.linkage_stmts {
            LinkageStmts::ImportStmt(m) => {
                let n = import(m);
                node.import.push(n);
            }
            LinkageStmts::IncludeStmt(m) => {
                let n = include(m);
                node.include.push(n);
            }
        }
    }
    for m in m.module_stmt.module_stmt_list1.iter() {
        match &*m.meta_stmts {
            MetaStmts::OrganizationStmt(m) => {
                node.organization(m);
            }
            MetaStmts::ContactStmt(m) => {
                node.contact(m);
            }
            MetaStmts::DescriptionStmt(m) => {
                node.description(m);
            }
            MetaStmts::ReferenceStmt(m) => {
                node.reference(m);
            }
        }
    }
    for m in m.module_stmt.module_stmt_list2.iter() {
        let n = revision(&m.revision_stmt);
        node.revision.push(n);
    }
    for m in m.module_stmt.module_stmt_list3.iter() {
        match &*m.body_stmts {
            BodyStmts::ExtensionStmt(m) => {
                let n = extension(&m.extension_stmt);
                node.extension.push(n);
            }
            BodyStmts::FeatureStmt(_m) => {}
            BodyStmts::IdentityStmt(m) => {
                let n = identity(m);
                node.identity.push(n);
            }
            BodyStmts::TypedefStmt(m) => {
                let n = typedef(&m.typedef_stmt);
                node.typedef.push(n);
            }
            BodyStmts::GroupingStmt(m) => {
                let n = grouping(&m.grouping_stmt);
                node.grouping.push(n);
            }
            BodyStmts::DataDefStmt(m) => {
                datadef(&mut node.d, &m.data_def_stmt);
            }
            BodyStmts::AugmentStmt(_m) => {}
            BodyStmts::RpcStmt(_m) => {}
            BodyStmts::NotificationStmt(_m) => {}
            BodyStmts::DeviationStmt(_m) => {}
            BodyStmts::UnknownStmt(m) => {
                let n = unknown(&m.unknown_stmt);
                node.unknown.push(n);
            }
        }
    }
}

fn submodule(node: &mut SubmoduleNode, m: YangSubmoduleStmt) {
    for m in m.submodule_stmt.submodule_stmt_list.iter() {
        match &*m.submodule_header_stmts {
            SubmoduleHeaderStmts::YangVersionStmt(m) => {
                node.yang_version(m);
            }
            SubmoduleHeaderStmts::BelongsToStmt(m) => {
                node.belongs_to(m);
            }
        }
    }
    for m in m.submodule_stmt.submodule_stmt_list0.iter() {
        match &*m.linkage_stmts {
            LinkageStmts::ImportStmt(m) => {
                let import = import(m);
                node.import.push(import);
            }
            LinkageStmts::IncludeStmt(m) => {
                let include = include(m);
                node.include.push(include);
            }
        }
    }
    for m in m.submodule_stmt.submodule_stmt_list1.iter() {
        match &*m.meta_stmts {
            MetaStmts::OrganizationStmt(m) => {
                node.organization(m);
            }
            MetaStmts::ContactStmt(m) => {
                node.contact(m);
            }
            MetaStmts::DescriptionStmt(m) => {
                node.description(m);
            }
            MetaStmts::ReferenceStmt(m) => {
                node.reference(m);
            }
        }
    }
    for m in m.submodule_stmt.submodule_stmt_list2.iter() {
        let n = revision(&m.revision_stmt);
        node.revision.push(n);
    }
    for m in m.submodule_stmt.submodule_stmt_list3.iter() {
        match &*m.body_stmts {
            BodyStmts::ExtensionStmt(_m) => {}
            BodyStmts::FeatureStmt(_m) => {}
            BodyStmts::IdentityStmt(m) => {
                let n = identity(m);
                node.identity.push(n);
            }
            BodyStmts::TypedefStmt(m) => {
                let n = typedef(&m.typedef_stmt);
                node.typedef.push(n);
            }
            BodyStmts::GroupingStmt(m) => {
                let n = grouping(&m.grouping_stmt);
                node.grouping.push(n);
            }
            BodyStmts::DataDefStmt(m) => {
                datadef(&mut node.d, &m.data_def_stmt);
            }
            BodyStmts::AugmentStmt(_m) => {}
            BodyStmts::RpcStmt(_m) => {}
            BodyStmts::NotificationStmt(_m) => {}
            BodyStmts::DeviationStmt(_m) => {}
            BodyStmts::UnknownStmt(m) => {
                let n = unknown(&m.unknown_stmt);
                node.unknown.push(n);
            }
        }
    }
}

impl ModuleNode {
    fn yang_version(&mut self, m: &ModuleHeaderStmtsYangVersionStmt) {
        self.version = Some(yang_version(&m.yang_version_stmt));
    }

    fn namespace(&mut self, m: &ModuleHeaderStmtsNamespaceStmt) {
        self.namespace = Some(namespace(m));
    }

    fn prefix(&mut self, m: &ModuleHeaderStmtsPrefixStmt) {
        self.prefix = Some(prefix(m));
    }

    fn organization(&mut self, m: &MetaStmtsOrganizationStmt) {
        self.organization = Some(ystring(&m.organization_stmt.ystring));
    }

    fn contact(&mut self, m: &MetaStmtsContactStmt) {
        self.contact = Some(ystring(&m.contact_stmt.ystring));
    }

    fn description(&mut self, m: &MetaStmtsDescriptionStmt) {
        self.description = Some(ystring(&m.description_stmt.ystring));
    }

    fn reference(&mut self, m: &MetaStmtsReferenceStmt) {
        self.reference = Some(ystring(&m.reference_stmt.ystring));
    }
}

impl SubmoduleNode {
    fn yang_version(&mut self, m: &SubmoduleHeaderStmtsYangVersionStmt) {
        self.version = Some(yang_version(&m.yang_version_stmt));
    }

    fn belongs_to(&mut self, m: &SubmoduleHeaderStmtsBelongsToStmt) {
        let name = identifier_arg_str(&m.belongs_to_stmt.identifier_arg_str);
        let mut node = BelongsToNode::new(name);
        let prefix = identifier_arg_str(&m.belongs_to_stmt.prefix_stmt.identifier_arg_str);
        node.prefix = Some(prefix);
        self.belongs_to = Some(node);
    }

    fn organization(&mut self, m: &MetaStmtsOrganizationStmt) {
        self.organization = Some(ystring(&m.organization_stmt.ystring));
    }

    fn contact(&mut self, m: &MetaStmtsContactStmt) {
        self.contact = Some(ystring(&m.contact_stmt.ystring));
    }

    fn description(&mut self, m: &MetaStmtsDescriptionStmt) {
        self.description = Some(ystring(&m.description_stmt.ystring));
    }

    fn reference(&mut self, m: &MetaStmtsReferenceStmt) {
        self.reference = Some(ystring(&m.reference_stmt.ystring));
    }
}

fn datadef(node: &mut DatadefNode, m: &DataDefStmt) {
    match m {
        DataDefStmt::ContainerStmt(m) => {
            let n = container(&m.container_stmt);
            node.container.push(n);
        }
        DataDefStmt::ListStmt(m) => {
            let n = list(&m.list_stmt);
            node.list.push(n);
        }
        DataDefStmt::LeafStmt(m) => {
            let n = leaf(&m.leaf_stmt);
            node.leaf.push(n);
        }
        DataDefStmt::AnydataStmt(m) => {
            let n = anydata(&m.anydata_stmt);
            node.anydata.push(n);
        }
        DataDefStmt::AnyxmlStmt(m) => {
            let n = anyxml(&m.anyxml_stmt);
            node.anyxml.push(n);
        }
        DataDefStmt::ChoiceStmt(m) => {
            let n = choice(&m.choice_stmt);
            node.choice.push(n);
        }
        DataDefStmt::LeafListStmt(m) => {
            let n = leaf_list(&m.leaf_list_stmt);
            node.leaf_list.push(n);
        }
        DataDefStmt::UsesStmt(m) => {
            let n = uses(&m.uses_stmt);
            node.uses.push(n);
        }
    }
}

fn must(m: &MustStmt) -> MustNode {
    let name = ystring(&m.ystring);
    MustNode::new(name)
}

fn presence(m: &PresenceStmt) -> PresenceNode {
    let name = ystring(&m.ystring);
    PresenceNode::new(name)
}

fn when(m: &WhenStmt) -> WhenNode {
    let name = ystring(&m.ystring);
    let mut node = WhenNode::new(name);
    if let WhenStmtSuffix::LBraceWhenStmtListRBrace(m) = &*m.when_stmt_suffix {
        for m in m.when_stmt_list.iter() {
            match &*m.when_stmt_list_group {
                WhenStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                WhenStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
            }
        }
    }
    node
}

fn container(m: &ContainerStmt) -> ContainerNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = ContainerNode::new(name);
    if let ContainerStmtSuffix::LBraceContainerStmtListRBrace(m) = &*m.container_stmt_suffix {
        for m in m.container_stmt_list.iter() {
            match &*m.container_stmt_list_group {
                ContainerStmtListGroup::WhenStmt(m) => {
                    let n = when(&m.when_stmt);
                    node.when = Some(n);
                }
                ContainerStmtListGroup::IfFeatureStmt(_m) => {}
                ContainerStmtListGroup::MustStmt(m) => {
                    let n = must(&m.must_stmt);
                    node.must.push(n);
                }
                ContainerStmtListGroup::PresenceStmt(m) => {
                    let n = presence(&m.presence_stmt);
                    node.presence = Some(n);
                }
                ContainerStmtListGroup::ConfigStmt(m) => {
                    let n = config(&m.config_stmt);
                    node.config = Some(n);
                }
                ContainerStmtListGroup::StatusStmt(m) => {
                    let n = status(&m.status_stmt);
                    node.status = Some(n);
                }
                ContainerStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                ContainerStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
                ContainerStmtListGroup::DataDefStmt(m) => {
                    datadef(&mut node.d, &m.data_def_stmt);
                }
                ContainerStmtListGroup::ActionStmt(_m) => {}
                ContainerStmtListGroup::NotificationStmt(_m) => {}
                ContainerStmtListGroup::UnknownStmt(m) => {
                    let n = unknown(&m.unknown_stmt);
                    node.unknown.push(n);
                }
            }
        }
    }
    node
}

fn leaf(m: &LeafStmt) -> LeafNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = LeafNode::new(name);
    for m in m.leaf_stmt_list.iter() {
        match &*m.leaf_stmt_list_group {
            LeafStmtListGroup::WhenStmt(m) => {
                let n = when(&m.when_stmt);
                node.when = Some(n);
            }
            LeafStmtListGroup::StatusStmt(m) => {
                let n = status(&m.status_stmt);
                node.status = Some(n);
            }
            LeafStmtListGroup::IfFeatureStmt(_m) => {}
            LeafStmtListGroup::TypeStmt(m) => {
                let n = type_stmt(&m.type_stmt);
                node.type_stmt = Some(n);
            }
            LeafStmtListGroup::UnitsStmt(_m) => {}
            LeafStmtListGroup::MustStmt(m) => {
                let n = must(&m.must_stmt);
                node.must.push(n);
            }
            LeafStmtListGroup::DefaultStmt(_m) => {}
            LeafStmtListGroup::ConfigStmt(m) => {
                let n = config(&m.config_stmt);
                node.config = Some(n);
            }
            LeafStmtListGroup::MandatoryStmt(m) => {
                let n = mandatory(&m.mandatory_stmt);
                node.mandatory = Some(n);
            }
            LeafStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            LeafStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
            LeafStmtListGroup::UnknownStmt(m) => {
                let n = unknown(&m.unknown_stmt);
                node.unknown.push(n);
            }
        }
    }
    node
}

fn key(m: &KeyStmt) -> KeyNode {
    let mut keys = Vec::new();
    match &*m.key_arg_str.key_arg_str_suffix {
        KeyArgStrSuffix::KeyArg(m) => {
            keys.push(identifier_ref(&m.key_arg.identifier_ref));
        }
        KeyArgStrSuffix::DoubleQuotationKeyArgDoubleQuotation(m) => {
            keys.push(identifier_ref(&m.key_arg.identifier_ref));
        }
    }
    KeyNode::new(keys)
}

fn list(m: &ListStmt) -> ListNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    // Key node is mandatory for the list node.
    let mut key_node: Option<KeyNode> = None;
    for m in m.list_stmt_list.iter() {
        if let ListStmtListGroup::KeyStmt(m) = &*m.list_stmt_list_group {
            let n = key(&m.key_stmt);
            key_node = Some(n);
        }
    }
    let mut node = ListNode::new(name, key_node.unwrap_or(KeyNode::new(Vec::new())));
    for m in m.list_stmt_list.iter() {
        match &*m.list_stmt_list_group {
            ListStmtListGroup::WhenStmt(m) => {
                let n = when(&m.when_stmt);
                node.when = Some(n);
            }
            ListStmtListGroup::IfFeatureStmt(_m) => {}
            ListStmtListGroup::MustStmt(m) => {
                let n = must(&m.must_stmt);
                node.must.push(n)
            }
            ListStmtListGroup::KeyStmt(_already_handled) => {}
            ListStmtListGroup::ConfigStmt(m) => {
                let n = config(&m.config_stmt);
                node.config = Some(n);
            }
            ListStmtListGroup::MinElementsStmt(m) => {
                let n = min_elements(&m.min_elements_stmt);
                node.min_elements = Some(n);
            }
            ListStmtListGroup::MaxElementsStmt(m) => {
                let n = max_elements(&m.max_elements_stmt);
                node.max_elements = Some(n);
            }
            ListStmtListGroup::OrderedByStmt(_m) => {}
            ListStmtListGroup::StatusStmt(m) => {
                let n = status(&m.status_stmt);
                node.status = Some(n);
            }
            ListStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            ListStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
            ListStmtListGroup::DataDefStmt(m) => {
                datadef(&mut node.d, &m.data_def_stmt);
            }
            ListStmtListGroup::ActionStmt(_m) => {}
            ListStmtListGroup::NotificationStmt(_m) => {}
            ListStmtListGroup::UnknownStmt(m) => {
                let n = unknown(&m.unknown_stmt);
                node.unknown.push(n);
            }
        }
    }
    node
}

fn leaf_list(m: &LeafListStmt) -> LeafListNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = LeafListNode::new(name);
    for m in m.leaf_list_stmt_list.iter() {
        match &*m.leaf_list_stmt_list_group {
            LeafListStmtListGroup::WhenStmt(m) => {
                let n = when(&m.when_stmt);
                node.when = Some(n);
            }
            LeafListStmtListGroup::IfFeatureStmt(_m) => {}
            LeafListStmtListGroup::TypeStmt(m) => {
                let n = type_stmt(&m.type_stmt);
                node.type_stmt = Some(n);
            }
            LeafListStmtListGroup::UnitsStmt(_m) => {}
            LeafListStmtListGroup::MustStmt(_m) => {}
            LeafListStmtListGroup::DefaultStmt(_m) => {}
            LeafListStmtListGroup::ConfigStmt(m) => {
                let n = config(&m.config_stmt);
                node.config = Some(n);
            }
            LeafListStmtListGroup::MinElementsStmt(m) => {
                let n = min_elements(&m.min_elements_stmt);
                node.min_elements = Some(n);
            }
            LeafListStmtListGroup::MaxElementsStmt(m) => {
                let n = max_elements(&m.max_elements_stmt);
                node.max_elements = Some(n);
            }
            LeafListStmtListGroup::OrderedByStmt(_m) => {}
            LeafListStmtListGroup::StatusStmt(m) => {
                let n = status(&m.status_stmt);
                node.status = Some(n);
            }
            LeafListStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            LeafListStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
            LeafListStmtListGroup::UnknownStmt(m) => {
                let n = unknown(&m.unknown_stmt);
                node.unknown.push(n);
            }
        }
    }

    node
}

fn grouping(m: &GroupingStmt) -> GroupingNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = GroupingNode::new(name);

    for m in m.grouping_stmt_list.iter() {
        match &*m.grouping_stmt_list_group {
            GroupingStmtListGroup::StatusStmt(m) => {
                let n = status(&m.status_stmt);
                node.status = Some(n);
            }
            GroupingStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            GroupingStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
            GroupingStmtListGroup::TypedefStmt(_m) => {}
            GroupingStmtListGroup::GroupingStmt(_m) => {}
            GroupingStmtListGroup::DataDefStmt(m) => {
                datadef(&mut node.d, &m.data_def_stmt);
            }
            GroupingStmtListGroup::ActionStmt(_m) => {}
            GroupingStmtListGroup::NotificationStmt(_m) => {}
            GroupingStmtListGroup::UnknownStmt(_m) => {}
        }
    }

    node
}

fn import(m: &LinkageStmtsImportStmt) -> ImportNode {
    let name = identifier_arg_str(&m.import_stmt.identifier_arg_str);
    let mut node = ImportNode::new(name);

    for m in m.import_stmt.import_stmt_list.iter() {
        match &*m.import_stmt_list_group {
            ImportStmtListGroup::PrefixStmt(m) => {
                let prefix = identifier_arg_str(&m.prefix_stmt.identifier_arg_str);
                node.prefix = Some(prefix);
            }
            ImportStmtListGroup::RevisionDateStmt(m) => {
                let revision_date =
                    date_arg_str(&m.revision_date_stmt.date_arg_str.date_arg_str_suffix);
                node.revision_date = Some(revision_date);
            }
            ImportStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
            ImportStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
        }
    }
    node
}

fn include(m: &LinkageStmtsIncludeStmt) -> IncludeNode {
    let name = identifier_arg_str(&m.include_stmt.identifier_arg_str);
    let mut node = IncludeNode::new(name);

    if let IncludeStmtSuffix::LBraceIncludeStmtListRBrace(m) = &*m.include_stmt.include_stmt_suffix
    {
        for m in m.include_stmt_list.iter() {
            match &*m.include_stmt_list_group {
                IncludeStmtListGroup::RevisionDateStmt(m) => {
                    let revision_date =
                        date_arg_str(&m.revision_date_stmt.date_arg_str.date_arg_str_suffix);
                    node.revision_date = Some(revision_date);
                }
                IncludeStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                IncludeStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
            }
        }
    }
    node
}

fn revision(m: &RevisionStmt) -> RevisionNode {
    let name = date_arg_str(&m.date_arg_str.date_arg_str_suffix);
    let mut node = RevisionNode::new(name);

    for m in m.revision_stmt_list.iter() {
        match &*m.revision_stmt_list_group {
            RevisionStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            RevisionStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
        }
    }
    node
}

fn identity(m: &BodyStmtsIdentityStmt) -> IdentityNode {
    let name = identifier_arg_str(&m.identity_stmt.identifier_arg_str);
    let mut node = IdentityNode::new(name);
    for m in m.identity_stmt.identity_stmt_list.iter() {
        match &*m.identity_stmt_list_group {
            IdentityStmtListGroup::IfFeatureStmt(_m) => {}
            IdentityStmtListGroup::BaseStmt(m) => {
                let base = identifier_ref_arg_str(&m.base_stmt.identifier_ref_arg_str);
                node.base.push(base);
            }
            IdentityStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            IdentityStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
        }
    }
    node
}

fn namespace(m: &ModuleHeaderStmtsNamespaceStmt) -> String {
    match &*m.namespace_stmt.uri_str {
        UriStr::UriArg(m) => m.uri_arg.uri_arg.to_string(),
        UriStr::DoubleQuotationUriArgDoubleQuotation(m) => m.uri_arg.uri_arg.to_string(),
    }
}

fn prefix(m: &ModuleHeaderStmtsPrefixStmt) -> String {
    identifier_arg_str(&m.prefix_stmt.identifier_arg_str)
}

fn yang_version(m: &YangVersionStmt) -> String {
    match &*m.yang_version_arg_str.yang_version_arg_str_suffix {
        YangVersionArgStrSuffix::DoubleQuotationYangVersionArgDoubleQuotation(m) => {
            m.yang_version_arg.yang_version_arg.text().to_string()
        }
        YangVersionArgStrSuffix::YangVersionArg(m) => {
            m.yang_version_arg.yang_version_arg.text().to_string()
        }
    }
}

fn ystring(s: &Ystring) -> String {
    let mut line = String::new();
    let mut first = true;
    match &*s.basic_string {
        BasicString::DQString(m) => {
            for s in m.d_q_string.d_q_string_list.iter() {
                match &*s.d_q_char {
                    DQChar::DQUnescaped(m) => match &*m.d_q_unescaped {
                        DQUnescaped::DQNoEscape(m) => {
                            if first {
                                first = false
                            } else {
                                line.push('\n');
                            }
                            line.push_str(m.d_q_no_escape.d_q_no_escape.text());
                        }
                        DQUnescaped::NonAscii(m) => {
                            if first {
                                first = false
                            } else {
                                line.push('\n');
                            }
                            line.push_str(m.non_ascii.non_ascii.text());
                        }
                    },
                    DQChar::DQEscaped(_m) => {
                        // m.d_q_escaped.escape;
                        // m.d_q_escaped.d_q_escape_seq_char;
                    }
                }
            }
        }
        BasicString::SQString(_m) => {}
    }
    line
}

fn type_kind(name: &str) -> YangType {
    match name {
        "binary" => YangType::Binary,
        "bits" => YangType::Bits,
        "boolean" => YangType::Boolean,
        "decimal64" => YangType::Decimal64,
        "empty" => YangType::Empty,
        "enumeration" => YangType::Enumeration,
        "int8" => YangType::Int8,
        "int16" => YangType::Int16,
        "int32" => YangType::Int32,
        "int64" => YangType::Int64,
        "string" => YangType::String,
        "uint8" => YangType::Uint8,
        "uint16" => YangType::Uint16,
        "uint32" => YangType::Uint32,
        "uint64" => YangType::Uint64,
        "union" => YangType::Union,
        "leafref" => YangType::Leafref,
        "identityref" => YangType::Identityref,
        _ => YangType::Path,
    }
}

fn range_boundary<T: FromStr>(m: &RangeBoundary) -> Result<RangeVal<T>, ()> {
    match m {
        RangeBoundary::Min(_) => Ok(RangeVal::Min),
        RangeBoundary::Max(_) => Ok(RangeVal::Max),
        RangeBoundary::MinusQuestLBracket0Minus9RBracketPlus(m) => {
            let v = T::from_str(m.minus_quest_l_bracket0_minus9_r_bracket_plus.text());
            if let Ok(v) = v {
                Ok(RangeVal::Val(v))
            } else {
                Err(())
            }
        }
    }
}

fn range_part<T: FromStr>(m: &RangePart) -> Range<T> {
    let start = range_boundary(&m.range_boundary).unwrap();
    if let Some(end) = &m.range_part_opt {
        let end = range_boundary(&end.range_boundary).unwrap();
        Range {
            start,
            end: Some(end),
        }
    } else {
        Range { start, end: None }
    }
}

fn range_arg_type<T: FromStr>(v: &mut Vec<Range<T>>, m: &RangeArg) {
    let range = range_part::<T>(&m.range_part);
    v.push(range);
    if let Some(m) = &m.range_arg_opt {
        range_arg_type(v, &m.range_arg);
    }
}

fn range_arg(m: &RangeArg, kind: YangType) -> RangeNode {
    match kind {
        YangType::Int8 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<i8>(&mut ranges, m);
            RangeNode::I8(ranges)
        }
        YangType::Int16 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<i16>(&mut ranges, m);
            RangeNode::I16(ranges)
        }
        YangType::Int32 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<i32>(&mut ranges, m);
            RangeNode::I32(ranges)
        }
        YangType::Int64 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<i64>(&mut ranges, m);
            RangeNode::I64(ranges)
        }
        YangType::Uint8 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<u8>(&mut ranges, m);
            RangeNode::U8(ranges)
        }
        YangType::Uint16 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<u16>(&mut ranges, m);
            RangeNode::U16(ranges)
        }
        YangType::Uint32 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<u32>(&mut ranges, m);
            RangeNode::U32(ranges)
        }
        YangType::Uint64 => {
            let mut ranges = Vec::<_>::new();
            range_arg_type::<u64>(&mut ranges, m);
            RangeNode::U64(ranges)
        }
        _ => {
            let ranges = Vec::<Range<u8>>::new();
            RangeNode::U8(ranges)
        }
    }
}

fn range(m: &RangeStmt, kind: YangType) -> RangeNode {
    match &*m.range_arg_str {
        RangeArgStr::RangeArg(m) => range_arg(&m.range_arg, kind),
        RangeArgStr::DoubleQuotationRangeArgDoubleQuotation(m) => range_arg(&m.range_arg, kind),
    }
}

fn enum_stmt(m: &EnumStmt) -> EnumNode {
    let name = match &*m.enum_arg_str.enum_arg_str_suffix {
        EnumArgStrSuffix::AsciiNoBrace(m) => m.ascii_no_brace.ascii_no_brace.text().to_string(),
        EnumArgStrSuffix::DoubleQuotationAsciiNoBraceDoubleQuotation(m) => {
            m.ascii_no_brace.ascii_no_brace.text().to_string()
        }
    };
    EnumNode::new(name)
}

fn base_stmt(m: &BaseStmt) -> String {
    identifier_ref_arg_str(&m.identifier_ref_arg_str)
}

fn type_stmt(m: &TypeStmt) -> TypeNode {
    let name = identifier_ref_arg_str(&m.identifier_ref_arg_str);
    let kind = type_kind(&name);

    let mut node = TypeNode::new(name, kind);
    if let TypeStmtSuffix::LBraceTypeStmtListRBrace(m) = &*m.type_stmt_suffix {
        for m in m.type_stmt_list.iter() {
            match &*m.type_stmt_list_group {
                TypeStmtListGroup::FractionDigitsStmt(_m) => {}
                TypeStmtListGroup::EnumStmt(m) => {
                    let n = enum_stmt(&m.enum_stmt);
                    node.enum_stmt.push(n);
                }
                TypeStmtListGroup::BaseStmt(m) => {
                    let base = base_stmt(&m.base_stmt);
                    node.base = Some(base);
                }
                TypeStmtListGroup::LeafrefSpecification(_m) => {}
                TypeStmtListGroup::StringRestrictions(_m) => {}
                TypeStmtListGroup::RangeStmt(m) => {
                    let n = range(&m.range_stmt, kind);
                    node.range = Some(n);
                }
                TypeStmtListGroup::BitStmt(_m) => {}
                TypeStmtListGroup::TypeStmt(m) => {
                    let n = type_stmt(&m.type_stmt);
                    node.union.push(n);
                }
            }
        }
    }
    node
}

fn typedef(m: &TypedefStmt) -> TypedefNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = TypedefNode::new(name);

    for m in m.typedef_stmt_list.iter() {
        match &*m.typedef_stmt_list_group {
            TypedefStmtListGroup::TypeStmt(m) => {
                let n = type_stmt(&m.type_stmt);
                node.type_node = Some(n);
            }
            TypedefStmtListGroup::UnitsStmt(_m) => {}
            TypedefStmtListGroup::DefaultStmt(_m) => {}
            TypedefStmtListGroup::StatusStmt(m) => {
                let n = status(&m.status_stmt);
                node.status = Some(n);
            }
            TypedefStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            TypedefStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
        }
    }

    node
}

fn anydata(m: &AnydataStmt) -> AnydataNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    AnydataNode::new(name)
}

fn anyxml(m: &AnyxmlStmt) -> AnyxmlNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    AnyxmlNode::new(name)
}

fn choice(m: &ChoiceStmt) -> ChoiceNode {
    let name = identifier_ref_arg_str(&m.identifier_ref_arg_str);
    let mut node = ChoiceNode::new(name);
    if let ChoiceStmtSuffix::LBraceChoiceStmtListRBrace(m) = &*m.choice_stmt_suffix {
        for m in m.choice_stmt_list.iter() {
            match &*m.choice_stmt_list_group {
                ChoiceStmtListGroup::WhenStmt(m) => {
                    let n = when(&m.when_stmt);
                    node.when = Some(n);
                }
                ChoiceStmtListGroup::IfFeatureStmt(_m) => {}
                ChoiceStmtListGroup::DefaultStmt(_m) => {}
                ChoiceStmtListGroup::ConfigStmt(m) => {
                    let n = config(&m.config_stmt);
                    node.config = Some(n);
                }
                ChoiceStmtListGroup::MandatoryStmt(m) => {
                    let n = mandatory(&m.mandatory_stmt);
                    node.mandatory = Some(n)
                }
                ChoiceStmtListGroup::StatusStmt(m) => {
                    let n = status(&m.status_stmt);
                    node.status = Some(n);
                }
                ChoiceStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                ChoiceStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
                ChoiceStmtListGroup::ShortCaseStmt(_m) => {}
                ChoiceStmtListGroup::CaseStmt(m) => {
                    let _n = case(&m.case_stmt);
                }
            }
        }
    }
    node
}

fn case(m: &CaseStmt) -> CaseNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    CaseNode::new(name)
}

fn config(m: &ConfigStmt) -> ConfigNode {
    ConfigNode::new(m.config.text() == "true")
}

fn mandatory(m: &MandatoryStmt) -> MandatoryNode {
    MandatoryNode::new(m.mandatory.text() == "true")
}

fn min_elements(m: &MinElementsStmt) -> MinElementsNode {
    let num = m
        .l_bracket1_minus9_r_bracket_l_bracket0_minus9_r_bracket_star
        .text()
        .parse::<u64>();
    MinElementsNode::new(num.unwrap_or(0u64))
}

fn max_elements(m: &MaxElementsStmt) -> MaxElementsNode {
    let num = m
        .l_bracket1_minus9_r_bracket_l_bracket0_minus9_r_bracket_star
        .text()
        .parse::<u64>();
    MaxElementsNode::new(num.unwrap_or(0u64))
}

fn uses(m: &UsesStmt) -> UsesNode {
    let name = identifier_ref_arg_str(&m.identifier_ref_arg_str);
    let mut node = UsesNode::new(name);
    if let UsesStmtSuffix::LBraceUsesStmtListRBrace(m) = &*m.uses_stmt_suffix {
        for m in m.uses_stmt_list.iter() {
            match &*m.uses_stmt_list_group {
                UsesStmtListGroup::WhenStmt(m) => {
                    let n = when(&m.when_stmt);
                    node.when = Some(n);
                }
                UsesStmtListGroup::IfFeatureStmt(_m) => {}
                UsesStmtListGroup::StatusStmt(m) => {
                    let n = status(&m.status_stmt);
                    node.status = Some(n);
                }
                UsesStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                UsesStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
                UsesStmtListGroup::RefineStmt(_m) => {}
                UsesStmtListGroup::AugmentStmt(_m) => {}
            }
        }
    }
    node
}

fn status_arg(s: &str) -> StatusNodeEnum {
    match s {
        "current" => StatusNodeEnum::Current,
        "deprecated" => StatusNodeEnum::Deprecated,
        "obsolete" => StatusNodeEnum::Obsolete,
        _ => StatusNodeEnum::Current,
    }
}

fn status(m: &StatusStmt) -> StatusNode {
    let status = match &*m.status_arg_str.status_arg_str_suffix {
        StatusArgStrSuffix::StatusArg(m) => status_arg(m.status_arg.status_arg.text()),
        StatusArgStrSuffix::DoubleQuotationStatusArgDoubleQuotation(m) => {
            status_arg(m.status_arg.status_arg.text())
        }
    };
    StatusNode::new(status)
}

fn extension(m: &ExtensionStmt) -> ExtensionNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = ExtensionNode::new(name);
    if let ExtensionStmtSuffix::LBraceExtensionStmtListRBrace(m) = &*m.extension_stmt_suffix {
        for m in &m.extension_stmt_list {
            match &*m.extension_stmt_list_group {
                ExtensionStmtListGroup::ArgumentStmt(m) => {
                    let argument = identifier_arg_str(&m.argument_stmt.identifier_arg_str);
                    node.argument = Some(argument);
                }
                ExtensionStmtListGroup::StatusStmt(m) => {
                    let n = status(&m.status_stmt);
                    node.status = Some(n);
                }
                ExtensionStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                ExtensionStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
            }
        }
    }
    node
}

fn unknown(m: &UnknownStmt) -> UnknownNode {
    let name = identifier_ref(&m.identifier_ref);
    let mut node = UnknownNode::new(name);
    if let UnknownStmtSuffix0::YstringUnknownStmtSuffix(m) = &*m.unknown_stmt_suffix0 {
        node.argument = ystring(&m.ystring);
    }
    node
}

fn identifier_arg_str(arg: &IdentifierArgStr) -> String {
    match &*arg.identifier_arg_str_suffix {
        IdentifierArgStrSuffix::Identifier(i) => i.identifier.identifier.text().to_string(),
        IdentifierArgStrSuffix::DoubleQuotationIdentifierDoubleQuotation(i) => {
            i.identifier.identifier.text().to_string()
        }
    }
}

fn identifier_ref_arg_str(arg: &IdentifierRefArgStr) -> String {
    match &*arg.identifier_ref_arg_str_suffix {
        IdentifierRefArgStrSuffix::IdentifierRef(i) => {
            i.identifier_ref.identifier.identifier.text().to_string()
        }
        IdentifierRefArgStrSuffix::DoubleQuotationIdentifierRefDoubleQuotation(i) => {
            i.identifier_ref.identifier.identifier.text().to_string()
        }
    }
}

fn identifier_ref(m: &IdentifierRef) -> String {
    m.identifier.identifier.text().to_string()
}

fn date_arg_str(m: &DateArgStrSuffix) -> String {
    match m {
        DateArgStrSuffix::DateArg(m) => m.date_arg.date_arg.text().to_string(),
        DateArgStrSuffix::DoubleQuotationDateArgDoubleQuotation(m) => {
            m.date_arg.date_arg.text().to_string()
        }
    }
}
