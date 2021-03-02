use std::fs;
use std::io;
use std::num::ParseIntError;
use std::ops::RangeInclusive;

use Error::*;
use ParserError::*;

use crate::assemble::Assembler;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    ParserErrors(Vec<(usize, ParserError)>),
}

#[derive(Debug)]
pub enum ParserError {
    UnknownMacro { verb: String },
    WrongNumberOfArguments { verb: String, expected: RangeInclusive<usize>, actual: usize },
    InvalidIntegerArg(ParseIntError),
    InstHasMultipleArgs { verb: String, args: Vec<String> },
}

pub fn load_asm_file(filename: &str) -> Result<Assembler, Error> {
    match fs::File::open(filename) {
        Ok(f) => parse_asm(f),
        Err(err) => Err(IOError(err)),
    }
}

pub fn parse_asm<T: io::Read>(mut source: T) -> Result<Assembler, Error> {
    let mut text = String::new();
    match source.read_to_string(&mut text) {
        Ok(_) => {}
        Err(err) => return Err(IOError(err)),
    };
    let mut assem = Assembler::new();
    let mut errors: Vec<(usize, ParserError)> = Vec::new();
    assem.init();
    for (line_no, line) in text.lines().enumerate() {
        match process_asm_line(&mut assem, line) {
            Err(e) => errors.push((line_no, e)),
            _ => {}
        }
    }
    assem.finish();
    if !errors.is_empty() {
        return Err(ParserErrors(errors));
    }
    Ok(assem)
}

fn process_asm_line(assem: &mut Assembler, line: &str) -> Result<(), ParserError> {
    let line = line.to_string();
    let line = line.split(";").next().unwrap(); // Remove comments
    let words: Vec<&str> = line.split_ascii_whitespace().collect();
    if words.len() == 0 {
        return Ok(());
    }
    let verb = words[0];
    if verb.ends_with(":") {
        let name = &verb[..verb.len() - 1];
        if verb.starts_with("_") {
            assem.add_inner_label(name);
        } else {
            assem.add_top_level_label(name);
        }
        return Ok(());
    }
    let args = &words[1..];
    if verb.starts_with(".") {
        return process_macro(assem, verb, args);
    }
    match args {
        [] => {
            assem.add_inst(verb, 0);
        }
        [arg] => {
            if arg.starts_with("0x") {
                match i32::from_str_radix(&arg[2..], 16) {
                    Ok(arg) => assem.add_inst(verb, arg),
                    Err(err) => return Err(InvalidIntegerArg(err)),
                };
            } else if let Ok(arg) = i32::from_str_radix(arg, 10) {
                assem.add_inst(verb, arg);
            } else {
                assem.add_placeholder_inst(verb, arg);
            }
        }
        _ => return Err(InstHasMultipleArgs {
            verb: verb.to_string(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
        })
    }
    Ok(())
}

fn expect_num_args(verb: &str, args: &[&str], expected: RangeInclusive<usize>) -> Option<ParserError> {
    if !expected.contains(&args.len()) {
        Some(WrongNumberOfArguments {
            expected,
            actual: args.len(),
            verb: verb.to_string(),
        })
    } else {
        None
    }
}

fn process_macro(assem: &mut Assembler, verb: &str, args: &[&str]) -> Result<(), ParserError> {
    match verb {
        ".args" => {
            if let Some(err) = expect_num_args(verb, args, 1..=10) {
                return Err(err);
            }
            for arg_name in args {
                assem.add_arg_var(arg_name, 1);
            }
        }
        ".locals" => {
            if let Some(err) = expect_num_args(verb, args, 1..=10) {
                return Err(err);
            }
            for local_name in args {
                assem.add_local_var(local_name, 1);
            }
        }
        ".array" => {
            if let Some(err) = expect_num_args(verb, args, 2..=2) {
                return Err(err);
            }
            let name = args[0];
            let len = match i32::from_str_radix(args[1], 10) {
                Ok(len) => len,
                Err(err) => return Err(InvalidIntegerArg(err)),
            };
            assem.add_local_var(name, len);
            assem.add_local_const(&format!("{}.len", name), len);
        }
        ".start_frame" => {
            assem.start_frame();
        }
        ".end_frame" => {
            assem.end_frame();
        }
        unknown => return Err(UnknownMacro { verb: unknown.to_string() })
    }
    Ok(())
}
