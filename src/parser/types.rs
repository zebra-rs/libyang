use crate::modules::*;
use crate::parser::*;
use crate::Node;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, multispace0, multispace1};
use nom::multi::many0;
use nom::IResult;

// 4.2.4.  Built-In Types
//
//    YANG has a set of built-in types, similar to those of many
//    programming languages, but with some differences due to special
//    requirements of network management.  The following table summarizes
//    the built-in types discussed in Section 9:
//
//        +---------------------+-------------------------------------+
//        | Name                | Description                         |
//        +---------------------+-------------------------------------+
//        | binary              | Any binary data                     |
//        | bits                | A set of bits or flags              |
//        | boolean             | "true" or "false"                   |
//        | decimal64           | 64-bit signed decimal number        |
//        | empty               | A leaf that does not have any value |
//        | enumeration         | One of an enumerated set of strings |
//        | identityref         | A reference to an abstract identity |
//        | instance-identifier | A reference to a data tree node     |
//        | int8                | 8-bit signed integer                |
//        | int16               | 16-bit signed integer               |
//        | int32               | 32-bit signed integer               |
//        | int64               | 64-bit signed integer               |
//        | leafref             | A reference to a leaf instance      |
//        | string              | A character string                  |
//        | uint8               | 8-bit unsigned integer              |
//        | uint16              | 16-bit unsigned integer             |
//        | uint32              | 32-bit unsigned integer             |
//        | uint64              | 64-bit unsigned integer             |
//        | union               | Choice of member types              |
//        +---------------------+-------------------------------------+

// #[derive(Debug)]
// enum TypeKind {
//     Ynone,
//     // Yint8,
//     Yenum,
// }

// #[derive(Debug)]
// struct YangType {
//     name: String,
//     kind: TypeKind,
// }

// impl Default for YangType {
//     fn default() -> Self {
//         Self {
//             name: String::from(""),
//             kind: TypeKind::Ynone,
//         }
//     }
// }

// impl YangType {
//     fn new(kind: TypeKind) -> Self {
//         YangType {
//             kind: kind,
//             ..Default::default()
//         }
//     }
// }

// #[derive(Debug)]
// pub struct TypedefNode {
//     pub name: String,
//     pub typ: Option<Node>,
// }

// impl TypedefNode {
//     fn new(name: String, typ: Option<Node>) -> Self {
//         TypedefNode {
//             name: name,
//             typ: typ,
//         }
//     }
// }

fn is_digit_value(c: char) -> bool {
    c.is_digit(10)
}

fn digit_parse(s: &str) -> IResult<&str, &str> {
    take_while1(is_digit_value)(s)
}

pub fn value_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("value")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = alt((double_quoted_string, digit_parse))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    let node = ValueNode {
        name: String::from(v),
        nodes: (),
    };
    Ok((s, Node::ValueNode(Box::new(node))))
}

fn enum_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, nodes) = many0(alt((description_parse, value_parse)))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, nodes))
}

fn semicolon_end_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = tag(";")(s)?;
    Ok((s, vec![]))
}

fn enum_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("enum")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, ident) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, sub) = alt((enum_sub_parse, semicolon_end_parse))(s)?;
    let node = EnumNode {
        name: String::from(ident),
        nodes: (sub,),
    };
    Ok((s, Node::EnumNode(Box::new(node))))
}

fn int_range_parse(s: &str) -> IResult<&str, (&str, &str)> {
    let (s, _) = multispace0(s)?;
    let (s, k) = tag("range")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = double_quoted_string(s)?;

    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;

    Ok((s, (k, v)))
}

fn uint_range_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("range")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = double_quoted_string(s)?;
    let (_, r) = range_uint_parse(v)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    Ok((s, Node::RangeUint(Box::new(r))))
}

fn int_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, _) = many0(int_range_parse)(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, vec![]))
}

fn uint_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, n) = many0(uint_range_parse)(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, n))
}

fn pattern_parse(s: &str) -> IResult<&str, (&str, &str)> {
    let (s, _) = multispace0(s)?;
    let (s, k) = tag("pattern")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _v) = quoted_string_list(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    Ok((s, (k, "")))
}

fn length_parse(s: &str) -> IResult<&str, (&str, &str)> {
    let (s, _) = multispace0(s)?;
    let (s, k) = tag("length")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = double_quoted_string(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    Ok((s, (k, v)))
}

fn path_parse(s: &str) -> IResult<&str, (&str, &str)> {
    let (s, _) = multispace0(s)?;
    let (s, k) = tag("path")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = double_quoted_string(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    Ok((s, (k, v)))
}

fn type_sub_parse(s: &str) -> IResult<&str, Vec<Node>> {
    let (s, _) = char('{')(s)?;
    let (s, _) = many0(alt((pattern_parse, length_parse, path_parse)))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    Ok((s, vec![]))
}

fn type_int_parse(s: &str, type_string: String, type_kind: TypeKind) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag(type_string.as_str())(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = alt((int_sub_parse, semicolon_end_parse))(s)?;

    let node = TypeNode::new(type_kind);
    Ok((s, Node::Type(Box::new(node))))
}

fn type_uint_parse(s: &str, type_string: String, type_kind: TypeKind) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag(type_string.as_str())(s)?;
    let (s, _) = multispace0(s)?;
    let (s, mut subs) = alt((uint_sub_parse, semicolon_end_parse))(s)?;
    let mut node = TypeNode::new(type_kind);
    while let Some(sub) = subs.pop() {
        match sub {
            Node::RangeUint(n) => {
                node.range_uint = Some(*n);
            }
            _ => {}
        }
    }
    Ok((s, Node::Type(Box::new(node))))
}

fn type_int8_parse(s: &str) -> IResult<&str, Node> {
    type_int_parse(s, String::from("int8"), TypeKind::Yint8)
}

fn type_int16_parse(s: &str) -> IResult<&str, Node> {
    type_int_parse(s, String::from("int16"), TypeKind::Yint16)
}

fn type_int32_parse(s: &str) -> IResult<&str, Node> {
    type_int_parse(s, String::from("int32"), TypeKind::Yint32)
}

fn type_int64_parse(s: &str) -> IResult<&str, Node> {
    type_int_parse(s, String::from("int64"), TypeKind::Yint64)
}

fn type_uint8_parse(s: &str) -> IResult<&str, Node> {
    type_uint_parse(s, String::from("uint8"), TypeKind::Yuint8)
}

fn type_uint16_parse(s: &str) -> IResult<&str, Node> {
    type_uint_parse(s, String::from("uint16"), TypeKind::Yuint16)
}

fn type_uint32_parse(s: &str) -> IResult<&str, Node> {
    type_uint_parse(s, String::from("uint32"), TypeKind::Yuint32)
}

fn type_uint64_parse(s: &str) -> IResult<&str, Node> {
    type_uint_parse(s, String::from("uint64"), TypeKind::Yuint64)
}

fn type_string_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag("string")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = alt((type_sub_parse, semicolon_end_parse))(s)?;
    Ok((s, Node::EmptyNode))
}

fn type_boolean_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag("boolean")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = alt((type_sub_parse, semicolon_end_parse))(s)?;
    Ok((s, Node::EmptyNode))
}

fn type_enumeration_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag("enumeration")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('{')(s)?;
    let (s, enums) = many0(enum_parse)(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;

    let node = EnumerationNode::new(enums);

    Ok((s, Node::EnumerationNode(Box::new(node))))
}

// Single statement 'keyword: identity;'
fn single_identity_parse(s: &str, key: String) -> IResult<&str, &str> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag(key.as_str())(s)?;
    let (s, _) = multispace1(s)?;
    let (s, v) = alt((path_identifier, identifier))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(';')(s)?;
    Ok((s, v))
}

pub fn base_parse(s: &str) -> IResult<&str, Node> {
    let (s, v) = single_identity_parse(s, String::from("base"))?;
    let node = BaseNode::new(v.to_owned());
    Ok((s, Node::Base(Box::new(node))))
}

fn type_identityref_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag("identityref")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('{')(s)?;
    let (s, enums) = many0(base_parse)(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;

    let node = EnumerationNode::new(enums);

    Ok((s, Node::EnumerationNode(Box::new(node))))
}

fn type_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = path_identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = tag(";")(s)?;
    Ok((s, Node::EmptyNode))
}

fn type_union_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag("union")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('{')(s)?;
    let (s, enums) = many0(type_parse)(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;
    let node = EnumerationNode::new(enums);
    Ok((s, Node::EnumerationNode(Box::new(node))))
}

fn type_path_identifier_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = path_identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = alt((type_sub_parse, semicolon_end_parse))(s)?;

    Ok((s, Node::EmptyNode))
}

fn type_identifier_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("type")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = alt((type_sub_parse, semicolon_end_parse))(s)?;

    Ok((s, Node::EmptyNode))
}

pub fn find_type_node(nodes: &mut Vec<Node>) -> Option<Node> {
    let index = nodes.iter().position(|x| match x {
        Node::EnumerationNode(_) => true,
        _ => false,
    })?;
    Some(nodes.swap_remove(index))
}

pub fn default_parse(s: &str) -> IResult<&str, Node> {
    let (s, v) = single_statement_parse(s, String::from("default"))?;
    let n = DefaultNode::new(v.to_owned());
    Ok((s, Node::Default(Box::new(n))))
}

// Module:top
pub fn typedef_parse(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _k) = tag("typedef")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, ident) = identifier(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('{')(s)?;
    let (s, mut nodes) = many0(alt((
        default_parse,
        description_parse,
        reference_parse,
        types_parse,
        status_parse,
    )))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char('}')(s)?;

    let node = TypedefNode::new(String::from(ident), find_type_node(&mut nodes));
    Ok((s, Node::Typedef(Box::new(node))))
}

pub fn types_parse(s: &str) -> IResult<&str, Node> {
    alt((
        type_int8_parse,
        type_int16_parse,
        type_int32_parse,
        type_int64_parse,
        type_uint8_parse,
        type_uint16_parse,
        type_uint32_parse,
        type_uint64_parse,
        type_string_parse,
        type_boolean_parse,
        type_enumeration_parse,
        type_union_parse,
        type_identifier_parse,
        type_path_identifier_parse,
        type_identityref_parse,
    ))(s)
}

// WIP for range match function.
fn match_uint_range_node(range: &Vec<RangeUint>, n: u64) -> bool {
    for r in range {
        if r.end == RangeVal::<u64>::None {
            // Only start exists.
            let start = match r.start {
                RangeVal::<u64>::Val(v) => v,
                RangeVal::<u64>::Min => 0,
                RangeVal::<u64>::Max => 0,
                _ => 0,
            };
            if n == start {
                return true;
            } else {
                return false;
            }
        } else {
            let start = match r.start {
                RangeVal::<u64>::Val(v) => v,
                RangeVal::<u64>::Min => 0,
                RangeVal::<u64>::Max => 0,
                _ => 0,
            };
            let end = match r.end {
                RangeVal::<u64>::Val(v) => v,
                RangeVal::<u64>::Min => 0,
                RangeVal::<u64>::Max => 0,
                _ => 0,
            };
            if start <= n && n <= end {
                return true;
            } else {
                return false;
            }
        }
    }
    false
}

// WIP for match function.
pub fn match_node(node: &TypeNode, s: &str) -> bool {
    if node.kind != TypeKind::Yuint8 {
        return false;
    }
    if let Ok(v) = s.parse::<u64>() {
        if let Some(range) = &node.range_uint {
            return match_uint_range_node(&range, v);
        } else {
            // TODO Type value range check.
            return false;
        }
    } else {
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yang_type() {
        // let ytype = YangType::new(TypeKind::Yenum);
        // println!("{:?}", ytype);
    }

    #[test]
    fn test_value_parse() {
        let literal = "1a";
        let result = value_parse(literal);
        println!("XXX test_value_parse: {:?}", result);
        //assert_eq!(result, Ok(("", true)));
    }

    #[test]
    fn test_base_parse() {
        let literal = "base if:interface-type;";
        let result = base_parse(literal);
        println!("XXX test_base_parse: {:?}", result);
        //assert_eq!(result, Ok(("", true)));
    }

    #[test]
    fn test_identityref_parse() {
        let literal = r#"
        type identityref {
            base interface-type;
        }"#;
        let result = type_identityref_parse(literal);
        println!("XXX test_identityref_parse: {:?}", result);
    }

    #[test]
    fn test_uint8_range() {
        let literal = r#"
        type uint8 {
            range "0..63";
        }"#;
        let (_, result) = type_uint8_parse(literal).unwrap();
        match result {
            Node::Type(t) => {
                println!("test_uint8_range {:?}", t);
                assert_eq!(match_node(&t, "10"), true);
                assert_eq!(match_node(&t, "64"), false);
            }
            _ => {}
        }
    }
}
