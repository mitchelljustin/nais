use std::{fmt, fs};
use std::fmt::Formatter;
use std::io;
use std::num::ParseIntError;
use std::ops::RangeInclusive;

use Error::*;
use ParserError::*;

use crate::assembler::Assembler;

pub enum Error {
    IOError(io::Error),
    ParserErrors(Vec<(usize, ParserError)>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOError(e) => e.fmt(f),
            ParserErrors(errors) => {
                for (loc, err) in errors.into_iter() {
                    if let Err(e) = write!(f, "Line {}: {}", loc, err) {
                        return Err(e);
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    UnknownMacro { verb: String },
    WrongNumberOfArguments { verb: String, expected: RangeInclusive<usize>, actual: usize },
    InvalidIntegerArg(ParseIntError),
    OnlyAsciiCharsSupported { char: String },
    InstHasMultipleArgs { verb: String, args: Vec<String> },

    _NotAnIntegerArg,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
    let mut parser = Parser::new();
    parser.init();
    parser.process(text);
    parser.finish();
    if !parser.errors.is_empty() {
        Err(ParserErrors(parser.errors))
    } else {
        Ok(parser.assem)
    }
}


struct Parser {
    pub errors: Vec<(usize, ParserError)>,
    pub assem: Assembler,

    local_addrs: Vec<String>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            assem: Assembler::new(),
            errors: Vec::new(),
            local_addrs: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.assem.init();
    }

    pub fn process(&mut self, text: String) {
        for (line_no, line) in text.lines().enumerate() {
            match self.process_asm_line(line) {
                Err(e) => self.errors.push((line_no, e)),
                _ => {}
            }
        }
    }

    pub fn finish(&mut self) {
        self.assem.finish();
    }

    fn process_asm_line(&mut self, line: &str) -> Result<(), ParserError> {
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
                self.assem.add_inner_label(name);
            } else {
                self.assem.add_top_level_label(name);
            }
            return Ok(());
        }
        let args = &words[1..];
        if verb.starts_with(".") {
            return self.process_macro(verb, args);
        }
        match args {
            [] => {
                self.assem.add_inst(verb, 0);
            },
            [arg] => {
                return match Parser::parse_integer(arg) {
                    Ok(arg) => {
                        self.assem.add_inst(verb, arg);
                        Ok(())
                    }
                    Err(_NotAnIntegerArg) => {
                        self.assem.add_placeholder_inst(verb, arg);
                        Ok(())
                    }
                    Err(err) => Err(err),
                };
            }
            _ => return Err(InstHasMultipleArgs {
                verb: verb.to_string(),
                args: args.into_iter().map(|s| s.to_string()).collect(),
            }),
        }
        Ok(())
    }

    fn process_macro(&mut self, verb: &str, args: &[&str]) -> Result<(), ParserError> {
        match verb {
            ".args" => {
                if let Some(err) = Parser::expect_num_args(verb, args, 1..=10) {
                    return Err(err);
                }
                for arg_name in args {
                    self.assem.add_arg_var(arg_name, 1);
                }
            }
            ".locals" => {
                if let Some(err) = Parser::expect_num_args(verb, args, 1..=10) {
                    return Err(err);
                }
                for name in args {
                    self.assem.add_local_var(name, 1);
                    self.assem.add_local_const(&format!("{}.len", name), 1);
                }
            }
            ".local_addrs" => {
                if let Some(err) = Parser::expect_num_args(verb, args, 1..=10) {
                    return Err(err);
                }
                for name in args {
                    self.assem.add_local_var(&Parser::addr_name(name), 1);
                    self.local_addrs.push(name.to_string());
                }
            }
            ".stack_array" => {
                if let Some(err) = Parser::expect_num_args(verb, args, 2..=2) {
                    return Err(err);
                }
                let name = args[0];
                let len = match i32::from_str_radix(args[1], 10) {
                    Ok(len) => len,
                    Err(err) => return Err(InvalidIntegerArg(err)),
                };
                self.assem.add_local_var(name, len);
                self.assem.add_local_const(&format!("{}.len", name), len);
            }
            ".return" => {
                if let Some(err) = Parser::expect_num_args(verb, args, 1..=1) {
                    return Err(err);
                }
                self.assem.set_retval_name(args[0]);
            }
            ".start_frame" => {
                self.assem.add_placeholder_inst("loadi", "fp");

                self.assem.add_placeholder_inst("loadi", "sp");
                self.assem.add_placeholder_inst("storei", "fp");

                self.assem.alloc_locals();

                for name in self.local_addrs.drain(..) {
                    self.assem.add_placeholder_inst("loadi", "fp");
                    self.assem.add_placeholder_inst("addi", &name);
                    self.assem.add_placeholder_inst("storef", &Parser::addr_name(&name));
                }
            }
            ".end_frame" => {
                self.assem.free_locals();

                self.assem.add_placeholder_inst("storei", "fp");
            }
            ".define" => {
                if let Some(err) = Parser::expect_num_args(verb, args, 2..=2) {
                    return Err(err);
                }
                let name = args[0];
                let value = match Parser::parse_integer(args[1]) {
                    Ok(value) => value,
                    Err(err) => return Err(err),
                };
                self.assem.add_constant(name, value);
            }
            unknown => return Err(UnknownMacro { verb: unknown.to_string() })
        }
        Ok(())
    }



    fn parse_integer(arg: &str) -> Result<i32, ParserError> {
        if arg.starts_with("0x") {
            return match i32::from_str_radix(&arg[2..], 16) {
                Ok(arg) => Ok(arg),
                Err(err) => Err(InvalidIntegerArg(err)),
            };
        }
        if let Ok(arg) = i32::from_str_radix(arg, 10) {
            return Ok(arg);
        }
        if arg.len() == 3 && arg.starts_with("'") && arg.ends_with("'") {
            let char = &arg[1..2];
            if !char.is_ascii() {
                return Err(OnlyAsciiCharsSupported { char: char.to_string() });
            }
            return Ok(char.bytes().next().unwrap() as i32);
        }
        Err(_NotAnIntegerArg)
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

    fn addr_name(local_name: &str) -> String {
        format!("{}.addr", local_name)
    }
}

