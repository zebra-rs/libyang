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
                let mut nested = Vec::new();
                hoist_typedefs(&mut node.d, &mut nested);
                for g in node.grouping.iter_mut() {
                    hoist_typedefs(&mut g.d, &mut nested);
                }
                node.typedef.append(&mut nested);
                Ok(Node::Module(Box::new(node)))
            }
            Yang::SubmoduleStmt(m) => {
                let name = identifier_arg_str(&m.submodule_stmt.identifier_arg_str);
                let mut node = SubmoduleNode::new(name);
                submodule(&mut node, m);
                let mut nested = Vec::new();
                hoist_typedefs(&mut node.d, &mut nested);
                for g in node.grouping.iter_mut() {
                    hoist_typedefs(&mut g.d, &mut nested);
                }
                node.typedef.append(&mut nested);
                Ok(Node::Submodule(Box::new(node)))
            }
        },
        None => Err(YangError::EmptyDocument),
    }
}

/// Move the typedefs written inside containers and lists under `d` to
/// `out`, the module's own.
///
/// RFC 7950 §5.5 scopes such a typedef to the node it is written in and
/// that node's descendants. A type reference is looked up among the
/// module's typedefs, so the nested ones join them, after the module's:
/// every reference the scoping rules allow still resolves, and a name
/// defined at both levels resolves to the module's.
fn hoist_typedefs(d: &mut DatadefNode, out: &mut Vec<TypedefNode>) {
    for c in d.container.iter_mut() {
        out.append(&mut c.typedef);
        hoist_typedefs(&mut c.d, out);
    }
    for l in d.list.iter_mut() {
        out.append(&mut l.typedef);
        hoist_typedefs(&mut l.d, out);
    }
    for choice in d.choice.iter_mut() {
        for case in choice.cases.iter_mut() {
            hoist_typedefs(&mut case.d, out);
        }
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
            BodyStmts::FeatureStmt(m) => {
                let n = feature(&m.feature_stmt);
                node.feature.push(n);
            }
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
            BodyStmts::AugmentStmt(m) => {
                let n = augment(&m.augment_stmt);
                node.augment.push(n);
            }
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
            BodyStmts::FeatureStmt(m) => {
                let n = feature(&m.feature_stmt);
                node.feature.push(n);
            }
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
            BodyStmts::AugmentStmt(m) => {
                let n = augment(&m.augment_stmt);
                node.augment.push(n);
            }
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

fn augment(m: &AugmentStmt) -> AugmentNode {
    let target = ystring(&m.augment_arg_str.ystring);
    let mut node = AugmentNode::new(target);
    for s in m.augment_stmt_list.iter() {
        match &*s.augment_stmt_list_group {
            AugmentStmtListGroup::DataDefStmt(m) => {
                datadef(&mut node.d, &m.data_def_stmt);
            }
            AugmentStmtListGroup::DescriptionStmt(m) => {
                node.description = Some(ystring(&m.description_stmt.ystring));
            }
            AugmentStmtListGroup::ReferenceStmt(m) => {
                node.reference = Some(ystring(&m.reference_stmt.ystring));
            }
            AugmentStmtListGroup::CaseStmt(m) => {
                node.cases.push(case(&m.case_stmt));
            }
            AugmentStmtListGroup::ActionStmt(m) => {
                node.action.push(action(&m.action_stmt));
            }
            AugmentStmtListGroup::WhenStmt(m) => {
                node.when = Some(when(&m.when_stmt));
            }
            AugmentStmtListGroup::StatusStmt(m) => {
                node.status = Some(status(&m.status_stmt));
            }
            AugmentStmtListGroup::IfFeatureStmt(m) => {
                node.if_feature.push(if_feature(&m.if_feature_stmt));
            }
            // notification is parsed but not yet modeled; see
            // AugmentNode's doc comment for the reason.
            AugmentStmtListGroup::NotificationStmt(_) => {}
        }
    }
    node
}

/// Convert a `feature` statement (RFC 7950 §7.20.1) into a
/// `FeatureNode`, including the `if-feature` dependencies a feature
/// may declare on other features.
fn feature(m: &FeatureStmt) -> FeatureNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = FeatureNode::new(name);
    if let FeatureStmtSuffix::LBraceFeatureStmtListRBrace(m) = &*m.feature_stmt_suffix {
        for m in m.feature_stmt_list.iter() {
            match &*m.feature_stmt_list_group {
                FeatureStmtListGroup::IfFeatureStmt(m) => {
                    node.if_feature.push(if_feature(&m.if_feature_stmt));
                }
                FeatureStmtListGroup::StatusStmt(m) => {
                    node.status = Some(status(&m.status_stmt));
                }
                FeatureStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                FeatureStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
            }
        }
    }
    node
}

/// Convert an `if-feature` statement's expression (RFC 7950 §7.20.2)
/// into the structured `IfFeatureExprNode` tree. Reusable by any
/// statement that accepts `if-feature`.
fn if_feature(m: &IfFeatureStmt) -> IfFeatureNode {
    IfFeatureNode::new(if_feature_expr(&m.if_feature_expr_str.if_feature_expr))
}

fn if_feature_expr(e: &IfFeatureExpr) -> IfFeatureExprNode {
    let term = if_feature_term(&e.if_feature_term);
    match &e.if_feature_expr_opt {
        // `term or expr` — `or` is right-associative as written.
        Some(opt) => IfFeatureExprNode::Or(
            Box::new(term),
            Box::new(if_feature_expr(&opt.if_feature_expr)),
        ),
        None => term,
    }
}

fn if_feature_term(t: &IfFeatureTerm) -> IfFeatureExprNode {
    let factor = if_feature_factor(&t.if_feature_factor);
    match &t.if_feature_term_opt {
        // `factor and term` — binds tighter than `or`.
        Some(opt) => IfFeatureExprNode::And(
            Box::new(factor),
            Box::new(if_feature_term(&opt.if_feature_term)),
        ),
        None => factor,
    }
}

fn if_feature_factor(f: &IfFeatureFactor) -> IfFeatureExprNode {
    match f {
        IfFeatureFactor::NotIfFeatureFactor(m) => {
            IfFeatureExprNode::Not(Box::new(if_feature_factor(&m.if_feature_factor)))
        }
        IfFeatureFactor::LParenIfFeatureExprRParen(m) => if_feature_expr(&m.if_feature_expr),
        IfFeatureFactor::Identifier(m) => {
            IfFeatureExprNode::Feature(m.identifier.identifier.text().to_string())
        }
        IfFeatureFactor::DoubleQuotationIdentifierDoubleQuotation(m) => {
            IfFeatureExprNode::Feature(m.identifier.identifier.text().to_string())
        }
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
                ContainerStmtListGroup::IfFeatureStmt(m) => {
                    node.if_feature.push(if_feature(&m.if_feature_stmt));
                }
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
                ContainerStmtListGroup::TypedefStmt(m) => {
                    node.typedef.push(typedef(&m.typedef_stmt));
                }
                ContainerStmtListGroup::DataDefStmt(m) => {
                    datadef(&mut node.d, &m.data_def_stmt);
                }
                ContainerStmtListGroup::ActionStmt(m) => {
                    let n = action(&m.action_stmt);
                    node.action.push(n);
                }
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
            LeafStmtListGroup::IfFeatureStmt(m) => {
                node.if_feature.push(if_feature(&m.if_feature_stmt));
            }
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
    let mut arg = match &*m.key_arg_str {
        KeyArgStr::KeyArg(m) => &*m.key_arg,
        KeyArgStr::DoubleQuotationKeyArgDoubleQuotation(m) => &*m.key_arg,
    };
    // Every leaf the key names, in order (RFC 7950 §7.8.2): the
    // grammar reads `key "a b c"` as `a` followed by the rest.
    let mut keys = vec![identifier_ref(&arg.identifier_ref)];
    while let Some(rest) = &arg.key_arg_opt {
        arg = &rest.key_arg;
        keys.push(identifier_ref(&arg.identifier_ref));
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
            ListStmtListGroup::IfFeatureStmt(m) => {
                node.if_feature.push(if_feature(&m.if_feature_stmt));
            }
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
            ListStmtListGroup::OrderedByStmt(m) => {
                node.ordered_by = Some(ordered_by(&m.ordered_by_stmt));
            }
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
            ListStmtListGroup::TypedefStmt(m) => {
                node.typedef.push(typedef(&m.typedef_stmt));
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
            LeafListStmtListGroup::IfFeatureStmt(m) => {
                node.if_feature.push(if_feature(&m.if_feature_stmt));
            }
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
            LeafListStmtListGroup::OrderedByStmt(m) => {
                node.ordered_by = Some(ordered_by(&m.ordered_by_stmt));
            }
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
                let revision_date = date_arg_str(&m.revision_date_stmt.date_arg_str);
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
                    let revision_date = date_arg_str(&m.revision_date_stmt.date_arg_str);
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
    let name = date_arg_str(&m.date_arg_str);
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
            IdentityStmtListGroup::IfFeatureStmt(m) => {
                node.if_feature.push(if_feature(&m.if_feature_stmt));
            }
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
        UriStr::UriArg(m) => m.uri_arg.uri_arg.text().to_string(),
        UriStr::DoubleQuotationUriArgDoubleQuotation(m) => m.uri_arg.uri_arg.text().to_string(),
    }
}

fn prefix(m: &ModuleHeaderStmtsPrefixStmt) -> String {
    identifier_arg_str(&m.prefix_stmt.identifier_arg_str)
}

fn yang_version(m: &YangVersionStmt) -> String {
    match &*m.yang_version_arg_str {
        YangVersionArgStr::DoubleQuotationYangVersionArgDoubleQuotation(m) => {
            m.yang_version_arg.yang_version_arg.text().to_string()
        }
        YangVersionArgStr::YangVersionArg(m) => {
            m.yang_version_arg.yang_version_arg.text().to_string()
        }
    }
}

fn ystring(s: &Ystring) -> String {
    match s {
        Ystring::BasicStringYstringOpt(inner) => {
            let mut line = basic_string(&inner.basic_string);
            // The YANG `+` continuation (RFC 7950 §6.1.3 — adjacent
            // quoted strings are concatenated with no inserted
            // characters).
            if let Some(opt) = inner.ystring_opt.as_ref() {
                line.push_str(&ystring(&opt.ystring));
            }
            line
        }
        // RFC 7950 §6.1.3 unquoted-string form. The `Identifier`
        // token covers the common case (units / description /
        // organization / etc. with a single bareword argument); the
        // `+` concatenation grammar is reserved for the quoted form.
        Ystring::Identifier(inner) => inner.identifier.identifier.text().to_string(),
    }
}

/// The argument of `pattern` and `path`: quoted strings joined by `+`,
/// or one unquoted string, which may start with a character an
/// identifier does not (`[0-7]`, `/a:b/c:d`).
fn arg_str(s: &ArgStr) -> String {
    match s {
        ArgStr::BasicStringArgStrOpt(inner) => {
            let mut line = basic_string(&inner.basic_string);
            if let Some(opt) = inner.arg_str_opt.as_ref() {
                line.push_str(&arg_str(&opt.arg_str));
            }
            line
        }
        ArgStr::UnquotedArg(inner) => inner.unquoted_arg.unquoted_arg.text().to_string(),
    }
}

fn basic_string(s: &BasicString) -> String {
    match s {
        BasicString::DQString(m) => dq_string(&m.d_q_string),
        BasicString::SQString(m) => {
            let mut line = String::new();
            let mut first = true;
            for s in m.s_q_string.s_q_string_list.iter() {
                // A single-quoted string has no escapes (RFC 7950
                // §6.1.3): a backslash is in the text as it is.
                let text = match &*s.s_q_char.s_q_unescaped {
                    SQUnescaped::SQNoEscape(m) => m.s_q_no_escape.s_q_no_escape.text(),
                    SQUnescaped::NonAscii(m) => m.non_ascii.non_ascii.text(),
                };
                if first {
                    first = false
                } else {
                    line.push('\n');
                }
                line.push_str(text);
            }
            line
        }
    }
}

fn dq_string(m: &DQString) -> String {
    let mut line = String::new();
    let mut first = true;
    for s in m.d_q_string_list.iter() {
        match &*s.d_q_char {
            DQChar::DQUnescaped(m) => {
                let text = match &*m.d_q_unescaped {
                    DQUnescaped::DQNoEscape(m) => m.d_q_no_escape.d_q_no_escape.text(),
                    DQUnescaped::NonAscii(m) => m.non_ascii.non_ascii.text(),
                };
                if first {
                    first = false
                } else {
                    line.push('\n');
                }
                line.push_str(text);
            }
            DQChar::DQEscaped(_m) => {
                // m.d_q_escaped.escape;
                // m.d_q_escaped.d_q_escape_seq_char;
            }
        }
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

fn range_boundary<T: FromStr>(m: &RangeBoundary) -> Option<RangeVal<T>> {
    Some(match m {
        RangeBoundary::Min(_) => RangeVal::Min,
        RangeBoundary::Max(_) => RangeVal::Max,
        RangeBoundary::RangeNumber(m) => {
            RangeVal::Val(m.range_number.range_number.text().parse().ok()?)
        }
    })
}

/// The parts of a range as `T`, or `None` when a bound is not one.
fn range_parts<T: FromStr>(m: &RangeArg) -> Option<Vec<Range<T>>> {
    let mut ranges = Vec::new();
    let mut arg = m;
    loop {
        let start = range_boundary(&arg.range_part.range_boundary)?;
        let end = match &arg.range_part.range_part_opt {
            Some(end) => Some(range_boundary(&end.range_boundary)?),
            None => None,
        };
        ranges.push(Range { start, end });
        match &arg.range_arg_opt {
            Some(rest) => arg = &rest.range_arg,
            None => return Some(ranges),
        }
    }
}

/// The range `m` of a type of kind `kind`, or `None` when it is not
/// one that is kept.
///
/// The range of an integer type is `None` only when a bound is not a
/// number of that type, and `type_stmt_check` refuses a file with such
/// a bound before it is read here.
fn range_arg(m: &RangeArg, kind: YangType) -> Option<RangeNode> {
    Some(match kind {
        YangType::Int8 => RangeNode::I8(range_parts(m)?),
        YangType::Int16 => RangeNode::I16(range_parts(m)?),
        YangType::Int32 => RangeNode::I32(range_parts(m)?),
        YangType::Int64 => RangeNode::I64(range_parts(m)?),
        YangType::Uint8 => RangeNode::U8(range_parts(m)?),
        YangType::Uint16 => RangeNode::U16(range_parts(m)?),
        YangType::Uint32 => RangeNode::U32(range_parts(m)?),
        YangType::Uint64 => RangeNode::U64(range_parts(m)?),
        // A range on a typedef (`type my-number { range "1..10"; }`):
        // which integer type it restricts is known only when the
        // typedef is resolved, so its bounds are kept as the widest
        // integers that hold them until then (`RangeNode::cast`). A
        // range that reads as neither is one of a `decimal64`, which is
        // not kept, as for a `decimal64` written directly.
        YangType::Path => match range_parts::<i64>(m) {
            Some(ranges) => RangeNode::I64(ranges),
            None => RangeNode::U64(range_parts(m)?),
        },
        _ => RangeNode::U8(Vec::new()),
    })
}

fn range_arg_of<'a, 't>(m: &'a RangeArgStr<'t>) -> &'a RangeArg<'t> {
    match m {
        RangeArgStr::RangeArg(m) => &m.range_arg,
        RangeArgStr::DoubleQuotationRangeArgDoubleQuotation(m) => &m.range_arg,
    }
}

fn range(m: &RangeStmt, kind: YangType) -> Option<RangeNode> {
    range_arg(range_arg_of(&m.range_arg_str), kind)
}

fn length(m: &LengthStmt) -> Option<RangeNode> {
    range_arg(range_arg_of(&m.range_arg_str), YangType::Uint64)
}

/// Whether `text`, a bound of a range, is a number of the type `kind`.
/// Anything is one of a type that is not an integer: the bounds of its
/// range are not read.
fn bound_fits(text: &str, kind: YangType) -> bool {
    match kind {
        YangType::Int8 => text.parse::<i8>().is_ok(),
        YangType::Int16 => text.parse::<i16>().is_ok(),
        YangType::Int32 => text.parse::<i32>().is_ok(),
        YangType::Int64 => text.parse::<i64>().is_ok(),
        YangType::Uint8 => text.parse::<u8>().is_ok(),
        YangType::Uint16 => text.parse::<u16>().is_ok(),
        YangType::Uint32 => text.parse::<u32>().is_ok(),
        YangType::Uint64 => text.parse::<u64>().is_ok(),
        _ => true,
    }
}

/// The first bound of `m` that is not a number of the type `kind`, as
/// a syntax error at that bound: "`stmt` bound … is not `what`".
fn bounds_check(m: &RangeArg, kind: YangType, stmt: &str, what: &str) -> parol_runtime::Result<()> {
    let mut arg = m;
    loop {
        let part = &arg.range_part;
        let bounds = [
            Some(&*part.range_boundary),
            part.range_part_opt.as_ref().map(|end| &*end.range_boundary),
        ];
        for bound in bounds.into_iter().flatten() {
            if let RangeBoundary::RangeNumber(m) = bound {
                let token = &m.range_number.range_number;
                if !bound_fits(token.text(), kind) {
                    return Err(parol_runtime::ParserError::SyntaxErrors {
                        entries: vec![parol_runtime::SyntaxError {
                            cause: format!("{stmt} bound `{}` is not {what}", token.text()),
                            error_location: Box::new(token.location.clone()),
                            ..Default::default()
                        }],
                    }
                    .into());
                }
            }
        }
        match &arg.range_arg_opt {
            Some(rest) => arg = &rest.range_arg,
            None => return Ok(()),
        }
    }
}

/// What the grammar cannot say of a `type` statement: a bound of a
/// `range` on an integer type is a number of that type, and a bound of
/// a `length` an unsigned integer.
///
/// `RangeNumber` takes a fraction, for the range of a `decimal64`, and
/// any number of digits. This refuses the rest while the file is
/// parsed, so the file fails to load with a syntax error at the bound.
/// A range on a typedef is not checked: what it restricts is not known
/// here.
pub(crate) fn type_stmt_check(m: &TypeStmt) -> parol_runtime::Result<()> {
    let TypeStmtSuffix::LBraceTypeStmtListRBrace(body) = &*m.type_stmt_suffix else {
        return Ok(());
    };
    let name = identifier_ref_arg_str(&m.identifier_ref_arg_str);
    let kind = type_kind(&name);
    for m in body.type_stmt_list.iter() {
        match &*m.type_stmt_list_group {
            TypeStmtListGroup::RangeStmt(m) => bounds_check(
                range_arg_of(&m.range_stmt.range_arg_str),
                kind,
                "range",
                &format!("of type `{name}`"),
            )?,
            TypeStmtListGroup::StringRestrictions(m) => {
                if let StringRestrictions::LengthStmt(l) = &*m.string_restrictions {
                    bounds_check(
                        range_arg_of(&l.length_stmt.range_arg_str),
                        YangType::Uint64,
                        "length",
                        "an unsigned integer",
                    )?
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Whether `m` has `modifier invert-match`: the value must then not
/// match the pattern (RFC 7950 §9.4.6).
fn pattern_inverted(m: &PatternStmt) -> bool {
    let PatternStmtSuffix::LBracePatternStmtListRBrace(body) = &*m.pattern_stmt_suffix else {
        return false;
    };
    body.pattern_stmt_list.iter().any(|m| {
        matches!(&*m.restriction_substmt, RestrictionSubstmt::ModifierStmt(m)
            if ystring(&m.modifier_stmt.ystring) == "invert-match")
    })
}

fn enum_stmt(m: &EnumStmt) -> EnumNode {
    let name = match &*m.enum_arg_str {
        EnumArgStr::AsciiNoBrace(m) => m.ascii_no_brace.ascii_no_brace.text().to_string(),
        EnumArgStr::DoubleQuotationAsciiNoBraceDoubleQuotation(m) => {
            m.ascii_no_brace.ascii_no_brace.text().to_string()
        }
    };
    let mut node = EnumNode::new(name);
    if let EnumStmtSuffix::LBraceEnumStmtListRBrace(m) = &*m.enum_stmt_suffix {
        for m in m.enum_stmt_list.iter() {
            match &*m.enum_stmt_list_group {
                EnumStmtListGroup::IfFeatureStmt(m) => {
                    node.if_feature.push(if_feature(&m.if_feature_stmt));
                }
                EnumStmtListGroup::DescriptionStmt(_m) => {}
                EnumStmtListGroup::ValueStmt(_m) => {}
                EnumStmtListGroup::ReferenceStmt(_m) => {}
            }
        }
    }
    node
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
                TypeStmtListGroup::LeafrefSpecification(m) => {
                    if let LeafrefSpecification::PathStmt(p) = &*m.leafref_specification {
                        node.path = Some(arg_str(&p.path_stmt.arg_str));
                    }
                }
                TypeStmtListGroup::StringRestrictions(m) => match &*m.string_restrictions {
                    StringRestrictions::PatternStmt(p) => {
                        // `pattern` is one the value must match. An
                        // inverted pattern is not kept there: read as
                        // an ordinary one it would refuse every valid
                        // value.
                        if !pattern_inverted(&p.pattern_stmt) {
                            node.pattern = Some(arg_str(&p.pattern_stmt.arg_str));
                        }
                    }
                    StringRestrictions::LengthStmt(l) => {
                        node.length = length(&l.length_stmt);
                    }
                },
                TypeStmtListGroup::RangeStmt(m) => {
                    node.range = range(&m.range_stmt, kind);
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
                ChoiceStmtListGroup::IfFeatureStmt(m) => {
                    node.if_feature.push(if_feature(&m.if_feature_stmt));
                }
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
                ChoiceStmtListGroup::ShortCaseStmt(m) => {
                    // ShortCaseStmt creates an implicit case with the same name as the data definition
                    let mut case_node = CaseNode::new(String::from(""));
                    match &*m.short_case_stmt {
                        ShortCaseStmt::ChoiceStmt(m) => {
                            let n = choice(&m.choice_stmt);
                            case_node.name = n.name.clone();
                            case_node.d.choice.push(n);
                        }
                        ShortCaseStmt::ContainerStmt(m) => {
                            let n = container(&m.container_stmt);
                            case_node.name = n.name.clone();
                            case_node.d.container.push(n);
                        }
                        ShortCaseStmt::LeafStmt(m) => {
                            let n = leaf(&m.leaf_stmt);
                            case_node.name = n.name.clone();
                            case_node.d.leaf.push(n);
                        }
                        ShortCaseStmt::LeafListStmt(m) => {
                            let n = leaf_list(&m.leaf_list_stmt);
                            case_node.name = n.name.clone();
                            case_node.d.leaf_list.push(n);
                        }
                        ShortCaseStmt::ListStmt(m) => {
                            let n = list(&m.list_stmt);
                            case_node.name = n.name.clone();
                            case_node.d.list.push(n);
                        }
                        ShortCaseStmt::AnydataStmt(m) => {
                            let n = anydata(&m.anydata_stmt);
                            case_node.name = n.name.clone();
                            case_node.d.anydata.push(n);
                        }
                        ShortCaseStmt::AnyxmlStmt(m) => {
                            let n = anyxml(&m.anyxml_stmt);
                            case_node.name = n.name.clone();
                            case_node.d.anyxml.push(n);
                        }
                    }
                    node.cases.push(case_node);
                }
                ChoiceStmtListGroup::CaseStmt(m) => {
                    let n = case(&m.case_stmt);
                    node.cases.push(n);
                }
            }
        }
    }
    node
}

fn case(m: &CaseStmt) -> CaseNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = CaseNode::new(name);

    if let CaseStmtSuffix::LBraceCaseStmtListRBrace(m) = &*m.case_stmt_suffix {
        for m in m.case_stmt_list.iter() {
            match &*m.case_stmt_list_group {
                CaseStmtListGroup::WhenStmt(m) => {
                    let n = when(&m.when_stmt);
                    node.when = Some(n);
                }
                CaseStmtListGroup::IfFeatureStmt(m) => {
                    node.if_feature.push(if_feature(&m.if_feature_stmt));
                }
                CaseStmtListGroup::DataDefStmt(m) => {
                    datadef(&mut node.d, &m.data_def_stmt);
                }
                CaseStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                CaseStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
            }
        }
    }

    node
}

fn ordered_by(m: &OrderedByStmt) -> OrderedByNode {
    // `user` or `system`, bare or double-quoted.
    let text = match &*m.ordered_by_arg_str {
        OrderedByArgStr::OrderedByArg(m) => m.ordered_by_arg.ordered_by_arg.text(),
        OrderedByArgStr::DoubleQuotationOrderedByArgDoubleQuotation(m) => {
            m.ordered_by_arg.ordered_by_arg.text()
        }
    };
    OrderedByNode::new(text.to_string())
}

fn config(m: &ConfigStmt) -> ConfigNode {
    // Read the `true`/`false` argument (`config` takes a MandatoryArgStr).
    // Note: pre-4.x this read the `config` keyword token instead of the
    // argument, so it always evaluated to false — a latent bug now fixed.
    let text = match &*m.mandatory_arg_str {
        MandatoryArgStr::MandatoryArg(m) => m.mandatory_arg.mandatory_arg.text(),
        MandatoryArgStr::DoubleQuotationMandatoryArgDoubleQuotation(m) => {
            m.mandatory_arg.mandatory_arg.text()
        }
    };
    ConfigNode::new(text == "true")
}

fn mandatory(m: &MandatoryStmt) -> MandatoryNode {
    let text = match &*m.mandatory_arg_str {
        MandatoryArgStr::MandatoryArg(m) => m.mandatory_arg.mandatory_arg.text(),
        MandatoryArgStr::DoubleQuotationMandatoryArgDoubleQuotation(m) => {
            m.mandatory_arg.mandatory_arg.text()
        }
    };
    MandatoryNode::new(text == "true")
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
                UsesStmtListGroup::IfFeatureStmt(m) => {
                    node.if_feature.push(if_feature(&m.if_feature_stmt));
                }
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
                UsesStmtListGroup::AugmentStmt(m) => {
                    node.augment.push(augment(&m.augment_stmt));
                }
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
    let status = match &*m.status_arg_str {
        StatusArgStr::StatusArg(m) => status_arg(m.status_arg.status_arg.text()),
        StatusArgStr::DoubleQuotationStatusArgDoubleQuotation(m) => {
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
    // An extension that declares no argument is used with none
    // (`prefix:keyword;`): its argument is then empty.
    if let UnknownStmtSuffix0::YstringUnknownStmtSuffix(m) = &*m.unknown_stmt_suffix0 {
        node.argument = ystring(&m.ystring);
    }
    node
}

fn identifier_arg_str(arg: &IdentifierArgStr) -> String {
    match arg {
        IdentifierArgStr::Identifier(i) => i.identifier.identifier.text().to_string(),
        IdentifierArgStr::DoubleQuotationIdentifierDoubleQuotation(i) => {
            i.identifier.identifier.text().to_string()
        }
    }
}

fn identifier_ref_arg_str(arg: &IdentifierRefArgStr) -> String {
    match arg {
        IdentifierRefArgStr::IdentifierRef(i) => {
            i.identifier_ref.identifier.identifier.text().to_string()
        }
        IdentifierRefArgStr::DoubleQuotationIdentifierRefDoubleQuotation(i) => {
            i.identifier_ref.identifier.identifier.text().to_string()
        }
    }
}

fn identifier_ref(m: &IdentifierRef) -> String {
    m.identifier.identifier.text().to_string()
}

fn date_arg_str(m: &DateArgStr) -> String {
    match m {
        DateArgStr::DateArg(m) => m.date_arg.date_arg.text().to_string(),
        DateArgStr::DoubleQuotationDateArgDoubleQuotation(m) => {
            m.date_arg.date_arg.text().to_string()
        }
    }
}

fn action(m: &ActionStmt) -> ActionNode {
    let name = identifier_arg_str(&m.identifier_arg_str);
    let mut node = ActionNode::new(name);

    if let ActionStmtSuffix::LBraceActionStmtListRBrace(m) = &*m.action_stmt_suffix {
        for m in m.action_stmt_list.iter() {
            match &*m.action_stmt_list_group {
                ActionStmtListGroup::IfFeatureStmt(m) => {
                    node.if_feature.push(if_feature(&m.if_feature_stmt));
                }
                ActionStmtListGroup::StatusStmt(m) => {
                    let n = status(&m.status_stmt);
                    node.status = Some(n);
                }
                ActionStmtListGroup::DescriptionStmt(m) => {
                    node.description = Some(ystring(&m.description_stmt.ystring));
                }
                ActionStmtListGroup::ReferenceStmt(m) => {
                    node.reference = Some(ystring(&m.reference_stmt.ystring));
                }
                ActionStmtListGroup::InputStmt(m) => {
                    let n = input(&m.input_stmt);
                    node.input = Some(n);
                }
                ActionStmtListGroup::OutputStmt(m) => {
                    let n = output(&m.output_stmt);
                    node.output = Some(n);
                }
            }
        }
    }

    node
}

fn input(m: &InputStmt) -> InputNode {
    let mut node = InputNode::new();
    for m in m.input_stmt_list.iter() {
        datadef(&mut node.d, &m.data_def_stmt);
    }
    node
}

fn output(m: &OutputStmt) -> OutputNode {
    let mut node = OutputNode::new();
    for m in m.output_stmt_list.iter() {
        datadef(&mut node.d, &m.data_def_stmt);
    }
    node
}
