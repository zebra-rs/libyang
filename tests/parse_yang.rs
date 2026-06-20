// Smoke test for the YANG parsing pipeline, migrated from the old
// `src/main.rs` scratch binary. It parses a real IETF module and
// drives it through `yang()` to build the node tree.

use libyang::yang;
use libyang::yang_grammar::YangGrammar;
use libyang::yang_parser::parse;
use std::fs;

const SAMPLE: &str = "yang/ietf-bgp@2023-07-05.yang";

#[test]
fn parses_sample_module() {
    let input = fs::read_to_string(SAMPLE).expect("read sample module");
    let mut yang_grammar = YangGrammar::new();
    parse(&input, SAMPLE, &mut yang_grammar).expect("parse sample module");
}

#[test]
fn builds_node_tree_from_sample_module() {
    let input = fs::read_to_string(SAMPLE).expect("read sample module");
    let mut yang_grammar = YangGrammar::new();
    parse(&input, SAMPLE, &mut yang_grammar).expect("parse sample module");
    yang(yang_grammar).expect("build node tree");
}
