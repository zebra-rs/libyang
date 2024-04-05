use std::process;

use parol::{build::Builder, InnerAttributes, ParolErrorReporter};
use parol_runtime::Report;

fn main() {
    // CLI equivalent is:
    // parol -f ./yang.par -e ./yang-exp.par -p ./src/yang_parser.rs -a ./src/yang_grammar_trait.rs -t YangGrammar -m yang_grammar -g
    if let Err(err) = Builder::with_explicit_output_dir("src")
        .grammar_file("yang.par")
        .expanded_grammar_output_file("../yang-exp.par")
        .parser_output_file("yang_parser.rs")
        .actions_output_file("yang_grammar_trait.rs")
        .enable_auto_generation()
        .user_type_name("YangGrammar")
        .user_trait_module_name("yang_grammar")
        .inner_attributes(vec![InnerAttributes::AllowTooManyArguments])
        .trim_parse_tree()
        .generate_parser()
    {
        ParolErrorReporter::report_error(&err, "yang.par").unwrap_or_default();
        process::exit(1);
    }
}
