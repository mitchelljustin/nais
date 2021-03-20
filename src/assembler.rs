use std::{cmp, fmt, fs};
use std::fmt::Formatter;
use std::io;
use std::num::ParseIntError;
use std::ops::RangeInclusive;

use AssemblyError::*;
use ParserError::*;

use crate::linker::{DebugInfo, Linker, LinkerError, TargetTerm};
use crate::mem::addrs;

pub enum AssemblyError {
    IOError(io::Error),
    ASMParserErrors(Vec<(usize, ParserError)>),
    LinkerErrors(Vec<LinkerError>),
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOError(e) => e.fmt(f),
            LinkerErrors(errors) => {
                for (index, err) in errors.iter().enumerate() {
                    if let Err(e) = writeln!(f, "{}. {}", index + 1, err) {
                        return Err(e);
                    }
                }
                Ok(())
            }
            ASMParserErrors(errors) => {
                for (loc, err) in errors.iter() {
                    if let Err(e) = writeln!(f, "Line {}: {}", loc + 1, err) {
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
    WrongNumberOfArguments { verb: String, expected: RangeInclusive<usize>, actual: usize },
    InvalidLiteral(ParseIntError),
    SyntaxError { reason: String },

    MultipleErrors(Vec<ParserError>),

    UnknownError,
    _ArgIsIdent,
}

pub struct AssemblyResult {
    pub binary: Vec<i32>,
    pub debug_info: DebugInfo,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn assemble_file(filename: &str) -> Result<AssemblyResult, AssemblyError> {
    match fs::File::open(filename) {
        Ok(f) => assemble_from_source(f),
        Err(err) => Err(IOError(err)),
    }
}

pub fn assemble_from_source<T: io::Read>(mut source: T) -> Result<AssemblyResult, AssemblyError> {
    let mut text = String::new();
    match source.read_to_string(&mut text) {
        Ok(_) => {}
        Err(err) => return Err(IOError(err)),
    };
    let mut assembler = Assembler::new();
    assembler.init();
    assembler.process(&text);
    assembler.finish()
}


struct Assembler {
    errors: Vec<(usize, ParserError)>,
    linker: Linker,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            linker: Linker::new(),
            errors: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.add_default_constants();
    }

    fn add_default_constants(&mut self) {
        self.linker.add_global_constant("pc", addrs::PC);
        self.linker.add_global_constant("sp", addrs::SP);
        self.linker.add_global_constant("fp", addrs::FP);
        self.linker.add_global_constant("retval", -3);
        for (callcode, (_, call_name)) in crate::environment::CALL_LIST.iter().enumerate() {
            let const_name = format!(".cc.{}", call_name);
            self.linker.add_global_constant(&const_name, callcode as i32);
        }
        self.linker.add_global_constant(".fd.stdin", 1);
        self.linker.add_global_constant(".fd.stdout", 1);
        self.linker.add_global_constant(".fd.stderr", 2);
    }

    pub fn process(&mut self, text: &str) {
        for (line_no, line) in text.lines().enumerate() {
            match self.process_line(line) {
                Err(e) => self.errors.push((line_no, e)),
                _ => {}
            }
        }
    }

    pub fn finish(mut self) -> Result<AssemblyResult, AssemblyError> {
        self.linker.finish();
        if !self.errors.is_empty() {
            return Err(ASMParserErrors(self.errors));
        }
        let binary = match self.linker.link_binary() {
            Ok(bin) => bin,
            Err(errs) => return Err(LinkerErrors(errs)),
        };
        let debug_info = DebugInfo::from(self.linker);
        Ok(AssemblyResult {
            binary,
            debug_info,
        })
    }

    fn process_line(&mut self, line: &str) -> Result<(), ParserError> {
        let line = line.to_string();
        let line = line.split(";").next().unwrap(); // Remove comments
        let words: Vec<&str> = line.split_ascii_whitespace().collect();
        if words.len() == 0 {
            return Ok(());
        }
        let verb = words[0];
        if verb.ends_with(":") {
            self.process_label(verb);
            return Ok(());
        }
        let args = &words[1..];
        self.process_statement(verb, args)
    }

    fn process_label(&mut self, name: &str) {
        let name = &name[..name.len() - 1];
        if name.starts_with("_") {
            self.linker.add_inner_label(name);
        } else {
            self.linker.add_top_level_label(name);
        }
    }

    fn process_statement(&mut self, verb: &str, args: &[&str]) -> Result<(), ParserError> {
        match verb {
            ".define" => {
                let (name, value) = Assembler::expect_name_and_literal(verb, args)?;
                self.linker.add_global_constant(name, value);
            }
            ".word" => {
                if args.is_empty() {
                    return Err(SyntaxError { reason: ".word needs arguments".to_string() });
                }
                let mut bytes: Vec<u8> = vec![];
                for arg in args {
                    if let Some(val) = Assembler::parse_hex_word(arg) {
                        self.linker.add_raw_word(val);
                    } else if let Ok(val) = Assembler::parse_integer(arg) {
                        bytes.push(val as u8);
                    } else if let Ok(st) = Assembler::expect_string(arg) {
                        bytes.extend(st.bytes());
                    } else {
                        return Err(SyntaxError {
                            reason: ".word expects literals or quoted strings".to_string(),
                        });
                    }
                }
                let nbytes = bytes.len();
                if nbytes == 0 {
                    return Ok(());
                }
                let num_words = cmp::max(1, nbytes / 4);
                for i in 0..num_words {
                    let end = cmp::min(nbytes, (i + 1) * 4);
                    let word_bytes = &bytes[i * 4..end];
                    let mut word = 0i32;
                    for (i, b) in word_bytes.iter().enumerate() {
                        word |= (*b as i32) << (24 - i * 8);
                    }
                    self.linker.add_raw_word(word);
                }
            }
            ".string" => {
                for arg in args {
                    if let Ok(val) = Assembler::parse_integer(arg) {
                        self.linker.add_raw_word(val);
                        continue;
                    }
                    let st = Assembler::expect_string(arg)?;
                    for ch in st.chars() {
                        self.linker.add_raw_word(ch as i32);
                    }
                }
            }
            ".param" => {
                let (name, size) = Assembler::expect_name_and_literal(verb, args)?;
                self.linker.add_param(name, size);
                self.linker.add_local_constant(&Assembler::var_size_name(name), size);
            }
            ".local" => {
                let (name, size) = Assembler::expect_name_and_literal(verb, args)?;
                self.linker.add_local_var(name, size);
                self.linker.add_local_constant(&Assembler::var_size_name(name), size);
            }
            ".start_frame" => {
                self.process_inst("loadi", &["fp"])?;
                self.process_inst("loadi", &["sp"])?;
                self.process_inst("storei", &["fp"])?;

                self.linker.add_inst("addsp", self.linker.cur_frame().locals_size);
            }
            ".end_frame" => {
                self.linker.add_inst("addsp", -self.linker.cur_frame().locals_size);

                self.process_inst("storei", &["fp"])?;
            }
            opname => self.process_inst(opname, args)?,
        }
        Ok(())
    }

    fn process_inst(&mut self, opname: &str, args: &[&str]) -> Result<(), ParserError> {
        if args.is_empty() {
            self.linker.add_inst(opname, 0);
            return Ok(());
        }
        let (terms, errs): (Vec<_>, Vec<_>) = args
            .iter()
            .map(|arg| arg.strip_prefix(",").unwrap_or(arg))
            .map(|arg| arg.strip_suffix(",").unwrap_or(arg))
            .map(|arg| match Assembler::parse_integer(arg) {
                Ok(literal) => Ok(TargetTerm::Literal(literal)),
                Err(_ArgIsIdent) => Ok(TargetTerm::Ident(arg.to_string())),
                Err(err) => Err(err),
            })
            .partition(|r| r.is_ok());
        if !errs.is_empty() {
            return Err(MultipleErrors(errs
                .into_iter()
                .map(|r| r.unwrap_err())
                .collect()));
        }
        let target: Vec<_> = terms
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        self.linker.add_placeholder_inst(opname, target);
        Ok(())
    }

    fn expect_string(arg: &str) -> Result<&str, ParserError> {
        if arg.len() < 2 || &arg[0..1] != "\"" || &arg[arg.len() - 1..] != "\"" {
            return Err(SyntaxError { reason: format!("bad .string formatting: {}", arg) });
        }
        let st = &arg[1..arg.len() - 1];
        Ok(st)
    }

    fn parse_hex_word(arg: &str) -> Option<i32> {
        if !arg.starts_with("0x") {
            return None;
        }
        let arg = &arg[2..];
        if arg.len() != 8 {
            return None;
        }
        match u32::from_str_radix(arg, 16) {
            Ok(val) => Some(val as i32),
            Err(_) => None,
        }
    }

    fn parse_integer(arg: &str) -> Result<i32, ParserError> {
        if arg.starts_with("0x") {
            return match u32::from_str_radix(&arg[2..], 16) {
                Ok(arg) => Ok(arg as i32),
                Err(err) => Err(InvalidLiteral(err)),
            };
        }
        if let Ok(arg) = i32::from_str_radix(arg, 10) {
            return Ok(arg);
        }
        if arg.len() == 3 && arg.starts_with("'") && arg.ends_with("'") {
            let mut chars = arg.chars();
            chars.next();
            return Ok(chars.next().unwrap() as i32);
        }
        Err(_ArgIsIdent)
    }

    fn expect_num_args<'a>(verb: &'a str, args: &'a [&'a str], expected: RangeInclusive<usize>)
                           -> Result<&'a [&'a str], ParserError> {
        if !expected.contains(&args.len()) {
            return Err(WrongNumberOfArguments {
                expected,
                actual: args.len(),
                verb: verb.to_string(),
            });
        }
        Ok(args)
    }

    fn expect_name_and_literal<'a>(verb: &'a str, args: &'a [&'a str]) -> Result<(&'a str, i32), ParserError> {
        if let &[name, literal] = Assembler::expect_num_args(verb, args, 2..=2)? {
            let literal = Assembler::parse_integer(literal)?;
            return Ok((name, literal));
        }
        Err(UnknownError)
    }

    fn var_size_name(var_name: &str) -> String {
        format!(".sizeof.{}", var_name)
    }
}
