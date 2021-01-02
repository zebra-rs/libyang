use super::*;
use crate::modules::*;
use crate::Node;
use nom::branch::{alt, permutation};
use nom::bytes::complete::{tag, take_until, take_while, take_while1};
use nom::character::complete::{anychar, char, multispace0, multispace1, none_of};
use nom::combinator::{recognize, verify};
use nom::error::{make_error, ErrorKind};
use nom::multi::{many0, separated_list};
use nom::sequence::{delimited, pair};
use nom::Err;
use nom::IResult;

// RFC7950 6.2.  Identifiers
//     Identifiers are used to identify different kinds of YANG items by
//     name.  Each identifier starts with an uppercase or lowercase ASCII
//     letter or an underscore character, followed by zero or more ASCII
//     letters, digits, underscore characters, hyphens, and dots.
//     Implementations MUST support identifiers up to 64 characters in
//     length and MAY support longer identifiers.  Identifiers are case
//     sensitive.  The identifier syntax is formally defined by the rule
//     "identifier" in Section 14.  Identifiers can be specified as quoted
//     or unquoted strings.

pub fn is_identifier(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
}

pub fn identifier(s: &str) -> IResult<&str, &str> {
    recognize(pair(
        verify(anychar, |c: &char| c.is_ascii_alphabetic() || c == &'_'),
        take_while(is_identifier),
    ))(s)
}

pub fn path_identifier(s: &str) -> IResult<&str, &str> {
    let (s, _) = identifier(s)?;
    let (s, _) = tag(":")(s)?;
    let (s, id) = identifier(s)?;
    Ok((s, id))
}

// RFC7950 6.1.3.  Quoting
//     Within a double-quoted string (enclosed within " "), a backslash
//     character introduces a representation of a special character, which
//     depends on the character that immediately follows the backslash:
//
//     \n      newline
//     \t      a tab character
//     \"      a double quote
//     \\      a single backslash
//
//     The backslash MUST NOT be followed by any other character.

fn is_nonescaped_string_char(c: char) -> bool {
    let cv = c as u32;
    // 0x22 is double quote and 0x5C is backslash.
    (cv == 0x0a) || (cv == 0x0d) || ((cv >= 0x20) && (cv != 0x22) && (cv != 0x5c))
}

fn nonescaped_string(s: &str) -> IResult<&str, &str> {
    take_while1(is_nonescaped_string_char)(s)
}

fn escape_code(s: &str) -> IResult<&str, &str> {
    recognize(pair(
        tag("\\"),
        alt((tag("n"), tag("t"), tag("\""), tag("\\"))),
    ))(s)
}

fn string_body(s: &str) -> IResult<&str, &str> {
    recognize(many0(alt((nonescaped_string, escape_code))))(s)
}

pub fn double_quoted_string(s: &str) -> IResult<&str, &str> {
    // let parser = delimited(tag("\""), string_body, tag("\""));
    // map_res(parser, |x| unescape(x))(s)
    delimited(tag("\""), string_body, tag("\""))(s)
}

pub fn string_token_parse(s: &str) -> IResult<&str, &str> {
    let (s, v) = alt((string_body, double_quoted_string))(s)?;
    Ok((s, v))
}

pub fn quoted_string(s: &str) -> IResult<&str, String> {
    let (s, v) = delimited(tag("'"), many0(none_of("'")), tag("'"))(s)?;
    Ok((s, v.into_iter().collect()))
}

pub fn quoted_string_list(s: &str) -> IResult<&str, String> {
    let (s, v) = separated_list(
        permutation((multispace0, char('+'), multispace0)),
        quoted_string,
    )(s)?;
    Ok((s, v.into_iter().collect()))
}

pub fn boolean_parse(s: &str) -> IResult<&str, bool> {
    let (s, v) = alt((tag("true"), tag("false")))(s)?;
    match v {
        "true" => Ok((s, true)),
        "false" => Ok((s, false)),
        _ => Err(Err::Error(make_error(s, ErrorKind::Fix))),
    }
}

pub fn c_comment_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("/*")(s)?;
    let (s, _) = take_until("*/")(s)?;
    let (s, _) = tag("*/")(s)?;
    let (s, _) = multispace0(s)?;
    Ok((s, Node::EmptyNode))
}

// Single statement 'keyword: "double quoted string";'
pub fn single_statement_parse(s: &str, key: String) -> IResult<&str, &str> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag(key.as_str())(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = double_quoted_string(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    Ok((s, v))
}

pub fn description_parse(s: &str) -> IResult<&str, Node> {
    let (s, v) = single_statement_parse(s, String::from("description"))?;
    let n = DescriptionNode::new(v.to_owned());
    Ok((s, Node::Description(Box::new(n))))
}

pub fn reference_parse(s: &str) -> IResult<&str, Node> {
    let (s, v) = single_statement_parse(s, String::from("reference"))?;
    let node = ReferenceNode::new(v.to_owned());
    Ok((s, Node::Reference(Box::new(node))))
}

pub fn mandatory_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("mandatory")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = boolean_parse(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let mut node = MandatoryNode::new(String::from("mandatory"));
    node.mandatory = v;
    Ok((s, Node::Mandatory(Box::new(node))))
}

pub fn config_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("config")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = boolean_parse(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let mut node = ConfigNode::new(String::from("config"));
    node.config = v;
    Ok((s, Node::Config(Box::new(node))))
}

pub fn if_feature_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("if-feature")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let node = MandatoryNode::new(v.to_owned());
    Ok((s, Node::Mandatory(Box::new(node))))
}

pub fn semicolon_end_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = tag(";")(s)?;
    Ok((s, vec![]))
}

// RFC7950 14. AYNG ABNF Grammar for yang-version-stmt.
//
// yang-version-stmt   = yang-version-keyword sep yang-version-arg-str
// stmtend
//
// yang-version-arg-str = < a string that matches the rule >
// < yang-version-arg >
//
// yang-version-arg    = "1.1"
//
// Note: we've added yang-version-arg = "1" so that we can support RFC6020 YANG 1.
//
fn yang_version_arg_parse(s: &str) -> IResult<&str, &str> {
    let (s, v) = alt((tag("1.1"), tag("1")))(s)?;
    Ok((s, v))
}

fn yang_version_arg_auote_parse(s: &str) -> IResult<&str, &str> {
    let (s, v) = delimited(char('"'), yang_version_arg_parse, char('"'))(s)?;
    Ok((s, v))
}

fn yang_version_arg_str_parse(s: &str) -> IResult<&str, &str> {
    let (s, v) = alt((yang_version_arg_parse, yang_version_arg_auote_parse))(s)?;
    Ok((s, v))
}

fn yang_version_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("yang-version")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = yang_version_arg_str_parse(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let node = YangVersionNode::new(v.to_owned());
    Ok((s, Node::YangVersion(Box::new(node))))
}

fn module_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, k) = alt((
        tag("namespace"),
        tag("organization"),
        tag("contact"),
        tag("description"),
    ))(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = alt((double_quoted_string, identifier))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let node = match k {
        "namespace" => {
            let n = NamespaceNode::new(v.to_owned());
            Node::Namespace(Box::new(n))
        }
        "organization" => {
            let n = OrganizationNode::new(v.to_owned());
            Node::Organization(Box::new(n))
        }
        "contact" => {
            let n = ContactNode::new(v.to_owned());
            Node::Contact(Box::new(n))
        }
        "description" => {
            let n = DescriptionNode::new(v.to_owned());
            Node::Description(Box::new(n))
        }
        _ => Node::EmptyNode,
    };
    Ok((s, node))
}

// import b {
//     revision-date 2015-01-01;
// }
fn prefix_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("prefix")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = alt((double_quoted_string, identifier))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let n = PrefixNode::new(v.to_owned());
    Ok((s, Node::Prefix(Box::new(n))))
}

pub fn import_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, nodes) = many0(alt((
        prefix_parse,
        description_parse,
        reference_parse,
        revision_date_stmt_parse,
        base_parse,
        status_parse,
    )))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, nodes))
}

// The import's Substatements
// +---------------+---------+-------------+
// | substatement  | section | cardinality |
// +---------------+---------+-------------+
// | description   | 7.21.3  | 0..1        |
// | prefix        | 7.1.4   | 1           |
// | reference     | 7.21.4  | 0..1        |
// | revision-date | 7.1.5.1 | 0..1        |
// +---------------+---------+-------------+
pub fn import_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("import")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, mut subs) = alt((import_sub_parse, semicolon_end_parse))(s)?;
    let mut node = ImportNode::new(String::from(v));
    while let Some(sub) = subs.pop() {
        match sub {
            Node::Prefix(n) => {
                node.prefix = n.name.to_owned();
            }
            Node::Description(n) => {
                node.description = Some(n.name.to_owned());
            }
            Node::Reference(n) => {
                node.reference = Some(n.name.to_owned());
            }
            Node::RevisionDate(n) => {
                node.revision_date = n.name.to_owned();
            }
            _ => {}
        }
    }
    Ok((s, Node::Import(Box::new(node))))
}

// +--------------+---------+-------------+
// | substatement | section | cardinality |
// +--------------+---------+-------------+
// | base         | 7.18.2  | 0..n        |
// | description  | 7.21.3  | 0..1        |
// | if-feature   | 7.20.2  | 0..n        |
// | reference    | 7.21.4  | 0..1        |
// | status       | 7.21.2  | 0..1        |
// +--------------+---------+-------------+
pub fn identity_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("identity")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, mut subs) = alt((import_sub_parse, semicolon_end_parse))(s)?;
    let mut node = ImportNode::new(String::from(v));
    while let Some(sub) = subs.pop() {
        match sub {
            Node::Prefix(n) => {
                node.prefix = n.name.to_owned();
            }
            Node::Description(n) => {
                node.description = Some(n.name.to_owned());
            }
            Node::Reference(n) => {
                node.reference = Some(n.name.to_owned());
            }
            Node::RevisionDate(n) => {
                node.revision_date = n.name.to_owned();
            }
            _ => {}
        }
    }
    Ok((s, Node::Import(Box::new(node))))
}

// +--------------+---------+-------------+
// | substatement | section | cardinality |
// +--------------+---------+-------------+
// | description  | 7.21.3  | 0..1        |
// | if-feature   | 7.20.2  | 0..n        |
// | reference    | 7.21.4  | 0..1        |
// | status       | 7.21.2  | 0..1        |
// +--------------+---------+-------------+
pub fn feature_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("feature")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, mut subs) = alt((import_sub_parse, semicolon_end_parse))(s)?;
    let mut node = ImportNode::new(String::from(v));
    while let Some(sub) = subs.pop() {
        match sub {
            Node::Prefix(n) => {
                node.prefix = n.name.to_owned();
            }
            Node::Description(n) => {
                node.description = Some(n.name.to_owned());
            }
            Node::Reference(n) => {
                node.reference = Some(n.name.to_owned());
            }
            Node::RevisionDate(n) => {
                node.revision_date = n.name.to_owned();
            }
            _ => {}
        }
    }
    Ok((s, Node::Import(Box::new(node))))
}

pub fn units_parse(s: &str) -> IResult<&str, Node> {
    let (s, v) = single_statement_parse(s, String::from("units"))?;
    let n = UnitsNode::new(v.to_owned());
    Ok((s, Node::Units(Box::new(n))))
}

pub fn leaf_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, nodes) = many0(alt((
        description_parse,
        reference_parse,
        types_parse,
        mandatory_parse,
        config_parse,
        default_parse,
        if_feature_parse,
        units_parse,
        status_parse,
    )))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, nodes))
}

pub fn leaf_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("leaf")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _subs) = alt((leaf_sub_parse, semicolon_end_parse))(s)?;
    let node = LeafNode::new(String::from(v));
    Ok((s, Node::Leaf(Box::new(node))))
}

pub fn leaf_list_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("leaf-list")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _subs) = alt((leaf_sub_parse, semicolon_end_parse))(s)?;
    let node = LeafListNode::new(String::from(v));
    Ok((s, Node::LeafList(Box::new(node))))
}

pub fn key_parse(s: &str) -> IResult<&str, Node> {
    let (s, v) = single_statement_parse(s, String::from("key"))?;
    let node = KeyNode::new(v.to_owned());
    Ok((s, Node::Key(Box::new(node))))
}

pub fn list_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, nodes) = many0(alt((
        description_parse,
        key_parse,
        leaf_parse,
        leaf_list_parse,
        container_parse,
        status_parse,
    )))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, nodes))
}

pub fn list_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("list")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _subs) = alt((list_sub_parse, semicolon_end_parse))(s)?;
    let node = ListNode::new(String::from(v));
    Ok((s, Node::List(Box::new(node))))
}

// "status" parse.
pub fn status_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("status")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = alt((tag("current"), tag("obsolete"), tag("deprecated")))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let n = StatusNode::new(v.to_owned());
    Ok((s, Node::Status(Box::new(n))))
}

pub fn container_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, nodes) = many0(alt((
        description_parse,
        list_parse,
        config_parse,
        leaf_parse,
        status_parse,
    )))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, nodes))
}

pub fn container_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("container")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _subs) = alt((container_sub_parse, semicolon_end_parse))(s)?;
    let node = ContainerNode::new(String::from(v));

    // while let Some(sub) = subs.pop() {
    //     match sub {
    //         Node::Prefix(n) => {
    //             node.prefix = n.name.to_owned();
    //         }
    //         Node::Description(n) => {
    //             node.description = Some(n.name.to_owned());
    //         }
    //         Node::Reference(n) => {
    //             node.reference = Some(n.name.to_owned());
    //         }
    //         Node::RevisionDate(n) => {
    //             node.revision_date = n.name.to_owned();
    //         }
    //         _ => {}
    //     }
    // }
    Ok((s, Node::Container(Box::new(node))))
}

pub fn yang_parse(s: &str) -> IResult<&str, Module> {
    let (s, _) = tag("module")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, name) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('{')(s)?;
    let (s, mut nodes) = many0(alt((
        yang_version_parse,
        module_parse,
        prefix_parse,
        reference_parse,
        revision_parse,
        c_comment_parse,
        typedef_parse,
        import_parse,
        identity_parse,
        feature_parse,
        container_parse,
        leaf_parse,
    )))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;

    let mut module = Module::new(name.to_owned());

    while let Some(node) = nodes.pop() {
        match node {
            Node::Namespace(n) => {
                module.namespace = n.name.to_owned();
            }
            Node::Prefix(n) => {
                module.prefix = n.name.to_owned();
            }
            Node::Organization(n) => {
                module.organization = Some(n.name.to_owned());
            }
            Node::Contact(n) => {
                module.contact = Some(n.name.to_owned());
            }
            Node::Description(n) => {
                module.description = Some(n.name.to_owned());
            }
            Node::Revision(n) => {
                module.revisions.push(*n);
            }
            Node::Typedef(n) => {
                module.typedefs.insert(n.name.to_owned(), *n);
            }
            _ => {}
        }
    }
    Ok((s, module))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yang_version_arg_parse() {
        for literal in vec!["1", "1.1"] {
            match yang_version_arg_parse(literal) {
                Ok((_, v)) => {
                    assert_eq!(v, literal);
                }
                Err(e) => {
                    panic!("identifier {}", e);
                }
            }
        }
    }

    #[test]
    fn test_yang_version_token_parse() {
        for (literal, output) in vec![("1", "1"), ("1.1", "1.1"), (r#""1""#, "1")] {
            match yang_version_arg_str_parse(literal) {
                Ok((_, v)) => {
                    assert_eq!(v, output);
                }
                Err(e) => {
                    panic!("identifier {}", e);
                }
            }
        }
    }

    #[test]
    fn test_double_quoted_string() {
        let literal = r#""hoge\thoga\nhoge""#;
        let output = r#"hoge\thoga\nhoge"#;
        match double_quoted_string(literal) {
            Ok((_, o)) => {
                assert_eq!(o, output);
            }
            Err(e) => {
                panic!("double_quoted_string_test {}", e);
            }
        }
    }

    #[test]
    fn test_quoted_string_list() {
        let literal = r#"'collection abc' + 'hogehoge'"#;
        let (_, v) = quoted_string_list(literal).unwrap();
        assert_eq!(v, "collection abchogehoge");
    }

    #[test]
    fn test_prefix_parse() {
        let literal = r#"prefix if;"#;
        let (_, v) = prefix_parse(literal).unwrap();
        let node = PrefixNode::new(String::from("if"));
        assert_eq!(v, Node::Prefix(Box::new(node)));
    }

    #[test]
    fn test_prefix_parse_quote() {
        let literal = r#"prefix "if";"#;
        let (_, v) = prefix_parse(literal).unwrap();
        let node = PrefixNode::new(String::from("if"));
        assert_eq!(v, Node::Prefix(Box::new(node)));
    }

    #[test]
    fn test_nonescaped_string() {
        let literal = r#"main-routine_1 "#;
        let result = nonescaped_string(literal).unwrap();
        assert_eq!(result.1, "main-routine_1 ");
    }

    #[test]
    fn test_escape_code() {
        let literal = r"\na";
        let result = escape_code(literal).unwrap();
        assert_eq!(result.1, "\\n");
    }

    #[test]
    fn test_boolean_parse() {
        let literal = "true";
        let result = boolean_parse(literal);
        assert_eq!(result, Ok(("", true)));

        let literal = "false";
        let result = boolean_parse(literal);
        assert_eq!(result, Ok(("", false)));

        let literal = "hoge";
        let result = boolean_parse(literal);
        assert_eq!(result, Err(Err::Error((literal, ErrorKind::Tag))));
    }
}
