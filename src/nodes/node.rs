use crate::RangeNode;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Module(Box<ModuleNode>),
    Submodule(Box<SubmoduleNode>),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ModuleNode {
    pub name: String,
    pub version: Option<String>,
    pub namespace: Option<String>,
    pub prefix: Option<String>,
    pub import: Vec<ImportNode>,
    pub include: Vec<IncludeNode>,
    pub organization: Option<String>,
    pub contact: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub revision: Vec<RevisionNode>,
    pub d: DatadefNode,
    pub identity: Vec<IdentityNode>,
    pub typedef: Vec<TypedefNode>,
    pub extension: Vec<ExtensionNode>,
    pub grouping: Vec<GroupingNode>,
    pub unknown: Vec<UnknownNode>,
    pub identities: HashMap<String, Vec<String>>,
}

impl ModuleNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SubmoduleNode {
    pub name: String,
    pub version: Option<String>,
    pub belongs_to: Option<BelongsToNode>,
    pub import: Vec<ImportNode>,
    pub include: Vec<IncludeNode>,
    pub organization: Option<String>,
    pub contact: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub revision: Vec<RevisionNode>,
    pub d: DatadefNode,
    pub identity: Vec<IdentityNode>,
    pub typedef: Vec<TypedefNode>,
    pub grouping: Vec<GroupingNode>,
    pub unknown: Vec<UnknownNode>,
    pub identities: HashMap<String, Vec<String>>,
}

impl SubmoduleNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ImportNode {
    pub name: String,
    pub prefix: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub revision_date: Option<String>,
}

impl ImportNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn prefix(&self) -> String {
        match &self.prefix {
            Some(s) => s.to_owned(),
            None => String::from(""),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct IncludeNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub revision_date: Option<String>,
}

impl IncludeNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct RevisionNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
}

impl RevisionNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct BelongsToNode {
    pub name: String,
    pub prefix: Option<String>,
}

impl BelongsToNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct IdentityNode {
    pub name: String,
    pub base: Vec<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub status: Option<String>,
}

impl IdentityNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DatadefNode {
    pub container: Vec<ContainerNode>,
    pub leaf: Vec<LeafNode>,
    pub leaf_list: Vec<LeafListNode>,
    pub list: Vec<ListNode>,
    pub choice: Vec<ChoiceNode>,
    pub anydata: Vec<AnydataNode>,
    pub anyxml: Vec<AnyxmlNode>,
    pub uses: Vec<UsesNode>,
}

impl DatadefNode {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ContainerNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub when: Option<WhenNode>,
    pub status: Option<StatusNode>,
    pub presence: Option<PresenceNode>,
    pub config: Option<ConfigNode>,
    pub d: DatadefNode,
    pub must: Vec<MustNode>,
    pub unknown: Vec<UnknownNode>,
}

impl ContainerNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct LeafNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub when: Option<WhenNode>,
    pub status: Option<StatusNode>,
    pub config: Option<ConfigNode>,
    pub type_stmt: Option<TypeNode>,
    pub mandatory: Option<MandatoryNode>,
    pub must: Vec<MustNode>,
    pub unknown: Vec<UnknownNode>,
}

impl LeafNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct LeafListNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub when: Option<WhenNode>,
    pub status: Option<StatusNode>,
    pub config: Option<ConfigNode>,
    pub type_stmt: Option<TypeNode>,
    pub min_elements: Option<MinElementsNode>,
    pub max_elements: Option<MaxElementsNode>,
    pub unknown: Vec<UnknownNode>,
}

impl LeafListNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ListNode {
    pub name: String,
    pub key: KeyNode,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub when: Option<WhenNode>,
    pub status: Option<StatusNode>,
    pub config: Option<ConfigNode>,
    pub d: DatadefNode,
    pub min_elements: Option<MinElementsNode>,
    pub max_elements: Option<MaxElementsNode>,
    pub must: Vec<MustNode>,
    pub unknown: Vec<UnknownNode>,
}

impl ListNode {
    pub fn new(name: String, key: KeyNode) -> Self {
        Self {
            name,
            key,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct KeyNode {
    pub keys: Vec<String>,
}

impl KeyNode {
    pub fn new(keys: Vec<String>) -> Self {
        Self { keys }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ChoiceNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub status: Option<StatusNode>,
    pub mandatory: Option<MandatoryNode>,
    pub when: Option<WhenNode>,
    pub config: Option<ConfigNode>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct WhenNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
}

impl WhenNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct IfFeatureNode {
    pub name: String,
}

impl IfFeatureNode {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct OrderedByNode {
    pub name: String,
}

impl OrderedByNode {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl ChoiceNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AnydataNode {
    pub name: String,
    pub description: Option<String>,
}

impl AnydataNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AnyxmlNode {
    pub name: String,
    pub description: Option<String>,
}

impl AnyxmlNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UnitsNode {
    pub name: String,
    pub description: Option<String>,
}

impl UnitsNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct MustNode {
    pub name: String,
    pub description: Option<String>,
}

impl MustNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DefaultNode {
    pub name: String,
    pub description: Option<String>,
}

impl DefaultNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ConfigNode {
    pub config: bool,
}

impl ConfigNode {
    pub fn new(config: bool) -> Self {
        Self { config }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PresenceNode {
    pub name: String,
    pub description: Option<String>,
}

impl PresenceNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ActionNode {
    pub name: String,
    pub description: Option<String>,
}

impl ActionNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CaseNode {
    pub name: String,
    pub description: Option<String>,
}

impl CaseNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TypeNode {
    pub name: String,
    pub kind: YangType,
    pub description: Option<String>,
    pub pattern: Option<String>,
    pub range: Option<RangeNode>,
    pub enum_stmt: Vec<EnumNode>,
    pub base: Option<String>,
    pub union: Vec<TypeNode>,
    pub typedef: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, Default, Eq, Hash)]
pub enum YangType {
    #[default]
    Binary,
    Bits,
    Boolean,
    Decimal64,
    Empty,
    Enumeration,
    Int8,
    Int16,
    Int32,
    Int64,
    String,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Union,
    Leafref,
    Identityref,
    Path,
    // Extension for zebra.
    Ipv4Addr,
    Ipv4Prefix,
    Ipv6Addr,
    Ipv6Prefix,
}

impl TypeNode {
    pub fn new(name: String, kind: YangType) -> Self {
        Self {
            name,
            kind,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TypedefNode {
    pub name: String,
    pub type_node: Option<TypeNode>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub status: Option<StatusNode>,
}

impl TypedefNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct MandatoryNode {
    pub mandatory: bool,
}

impl MandatoryNode {
    pub fn new(mandatory: bool) -> Self {
        Self { mandatory }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct StatusNode {
    pub status: StatusNodeEnum,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum StatusNodeEnum {
    #[default]
    Current,
    Obsolete,
    Deprecated,
}

impl StatusNode {
    pub fn new(status: StatusNodeEnum) -> Self {
        Self { status }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ExtensionNode {
    pub name: String,
    pub argument: Option<String>,
    pub status: Option<StatusNode>,
    pub description: Option<String>,
    pub reference: Option<String>,
}

impl ExtensionNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UsesNode {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub when: Option<WhenNode>,
    pub status: Option<StatusNode>,
    pub d: DatadefNode,
}

impl UsesNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct GroupingNode {
    pub name: String,
    pub status: Option<StatusNode>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub typedef: Vec<TypedefNode>,
    pub grouping: Vec<GroupingNode>,
    pub d: DatadefNode,
}

impl GroupingNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct MinElementsNode {
    pub num: u64,
}

impl MinElementsNode {
    pub fn new(num: u64) -> Self {
        Self { num }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct MaxElementsNode {
    pub num: u64,
}

impl MaxElementsNode {
    pub fn new(num: u64) -> Self {
        Self { num }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct EnumNode {
    pub name: String,
}

impl EnumNode {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UnknownNode {
    pub name: String,
    pub argument: String,
}

impl UnknownNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
