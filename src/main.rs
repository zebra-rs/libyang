use libyang::*;

extern crate parol_runtime;
use crate::yang_grammar::YangGrammar;
use crate::yang_parser::parse;
use anyhow::{Context, Result};
use parol_runtime::{log::debug, Report};
use std::{env, fs, time::Instant};

struct ErrorReporter;
impl Report for ErrorReporter {}

#[allow(dead_code)]
fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let mut args: Vec<String> = env::args().collect();
    while let Some(file_name) = args.pop() {
        if args.is_empty() {
            return Ok(());
        }
        println!("path {file_name}");
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut yang_grammar = YangGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut yang_grammar) {
            Ok(_) => {
                let elapsed_time = now.elapsed();
                println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                // println!("Success!\n{}", yang_grammar);
            }
            Err(e) => {
                return ErrorReporter::report_error(&e, file_name);
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn main2() -> Result<()> {
    let file_name = "yang/ietf-bgp@2023-07-05.yang";
    // let file_name = "yang/coreswitch.yang";
    // let file_name = "yang/iana-bfd-types@2021-10-21.yang";
    // let file_name = "yang/ietf-bgp-rib-attributes@2023-07-05.yang";
    let input =
        fs::read_to_string(file_name).with_context(|| format!("Can't read file {file_name}"))?;
    let mut yang_grammar = YangGrammar::new();
    match parse(&input, file_name, &mut yang_grammar) {
        Ok(_) => {
            let _ = yang(yang_grammar);
        }
        Err(e) => {
            return ErrorReporter::report_error(&e, file_name);
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn main3() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let mut args: Vec<String> = env::args().collect();
    while let Some(file_name) = args.pop() {
        if args.is_empty() {
            return Ok(());
        }
        println!("path {file_name}");
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {file_name}"))?;
        let mut yang_grammar = YangGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut yang_grammar) {
            Ok(_) => {
                let elapsed_time = now.elapsed();
                println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                // println!("Success!\n{}", yang_grammar);
            }
            Err(e) => {
                return ErrorReporter::report_error(&e, file_name);
            }
        }
    }
    Ok(())
}
