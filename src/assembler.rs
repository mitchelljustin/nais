use std::{cmp, fmt, fs};
use std::fmt::Formatter;
use std::io;
use std::num::ParseIntError;

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
    InvalidIntLiteral(ParseIntError),
    UnknownMacro(String),
    SyntaxError(String),
    MultipleErrors(Vec<ParserError>),

    _NotAnInteger,
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

    extra_frame_setup: Vec<String>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            linker: Linker::new(),
            errors: Vec::new(),
            extra_frame_setup: Vec::new(),
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
        let args = &words[1..];
        self.process_statement(verb, args)
    }

    fn process_statement(&mut self, verb: &str, args: &[&str]) -> Result<(), ParserError> {
        match verb {
            label_name if label_name.ends_with(":") =>
                self.process_label(label_name)?,
            macro_name if macro_name.starts_with(".") =>
                self.process_macro(macro_name, args)?,
            op_name =>
                self.process_instruction(op_name, args)?,
        }
        Ok(())
    }

    fn process_label(&mut self, name: &str) -> Result<(), ParserError> {
        let name = &name[..name.len() - 1];
        if name.starts_with("_") {
            self.linker.add_inner_label(name);
        } else {
            self.linker.add_top_level_label(name);
        }
        Ok(())
    }

    fn process_macro(&mut self, macro_name: &str, args: &[&str]) -> Result<(), ParserError> {
        match macro_name {
            ".define" => {
                let (name, value) = Assembler::expect_ident_and_int(macro_name, args)?;
                self.linker.add_global_constant(name, value);
            }
            ".param" => {
                let (name, size) = Assembler::expect_ident_and_int(macro_name, args)?;
                self.linker.add_param(name, size);
                self.linker.add_local_constant(&Assembler::sizeof_name(name), size);
            }
            ".local" => {
                let (name, size) = Assembler::expect_ident_and_int(macro_name, args)?;
                self.linker.add_local_var(name, size);
                self.linker.add_local_constant(&Assembler::sizeof_name(name), size);
            }
            ".word" => return self.process_word_macro(args),
            ".string" => {
                for arg in args {
                    if let Ok(val) = Assembler::expect_int_literal(arg) {
                        self.linker.add_raw_word(val);
                        continue;
                    }
                    let st = Assembler::expect_string_literal(arg)?;
                    for ch in st.chars() {
                        self.linker.add_raw_word(ch as i32);
                    }
                }
            }
            ".start_frame" => {
                self.process_line("loadi fp")?;
                self.process_line("loadi sp")?;
                self.process_line("storei fp")?;

                self.linker.add_inst("addsp", self.linker.cur_frame().locals_size);

                let extra_lines = self.extra_frame_setup.drain(..).collect::<Vec<_>>();
                for line in extra_lines.iter() {
                    self.process_line(line)?;
                }
            }
            ".end_frame" => {
                self.linker.add_inst("addsp", -self.linker.cur_frame().locals_size);

                self.process_instruction("storei", &["fp"])?;
            }
            ".addr_of" => {
                if args.len() != 1 {
                    return Err(SyntaxError(format!("{} takes only one arg: {:?}", macro_name, args)));
                }
                let var_name = Assembler::expect_ident(args[0])?;
                let addr_name = format!("{}.addr", var_name);
                self.linker.add_local_var(&addr_name, 1);
                self.extra_frame_setup.extend_from_slice(&[
                    format!("loadi fp"),
                    format!("addi {}", var_name),
                    format!("storef {}", addr_name),
                ])
            }
            unknown => return Err(UnknownMacro(unknown.to_string())),
        }
        Ok(())
    }

    fn process_word_macro(&mut self, args: &[&str]) -> Result<(), ParserError> {
        if args.is_empty() {
            return Err(SyntaxError(".word needs arguments".to_string()));
        }
        let mut bytes: Vec<u8> = vec![];
        for arg in args {
            if let Some(val) = Assembler::parse_hex_i32(arg) {
                self.linker.add_raw_word(val);
            } else if let Ok(val) = Assembler::expect_int_literal(arg) {
                bytes.push(val as u8);
            } else if let Ok(string) = Assembler::expect_string_literal(arg) {
                bytes.extend(string.bytes());
            } else {
                return Err(SyntaxError(".word expects int or string literals".to_string()));
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
        Ok(())
    }

    fn process_instruction(&mut self, op_name: &str, args: &[&str]) -> Result<(), ParserError> {
        if args.is_empty() {
            self.linker.add_inst(op_name, 0);
            return Ok(());
        }
        let (terms, errs): (Vec<_>, Vec<_>) = args
            .iter()
            .map(|arg| arg.strip_prefix(",").unwrap_or(arg))
            .map(|arg| arg.strip_suffix(",").unwrap_or(arg))
            .map(|arg| match Assembler::expect_int_literal(arg) {
                Ok(literal) => Ok(TargetTerm::Literal(literal)),
                Err(_NotAnInteger) => Ok(TargetTerm::Ident(arg.to_string())),
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
        self.linker.add_placeholder_inst(op_name, target);
        Ok(())
    }

    fn parse_hex_i32(arg: &str) -> Option<i32> {
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

    fn expect_ident(arg: &str) -> Result<&str, ParserError> {
        if arg.len() == 0 {
            return Err(SyntaxError("where's the ident?".to_string()));
        }
        let mut chars = arg.chars();
        let first_char = chars.next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_'  {
            return Err(SyntaxError(format!("identifier must start with letter or '_': {}", arg)));
        }
        for ch in chars {
            if !ch.is_alphanumeric() && ch != '_' && ch != '.' {
                return Err(SyntaxError(format!("identifier must only contain alphanum, '_' or '.': {}", arg)));
            }
        }
        Ok(arg)
    }

    fn expect_string_literal(arg: &str) -> Result<&str, ParserError> {
        if arg.len() < 2 || &arg[0..1] != "\"" || &arg[arg.len() - 1..] != "\"" {
            return Err(SyntaxError(format!("invalid string literal: {}", arg)));
        }
        let string = &arg[1..arg.len() - 1];
        Ok(string)
    }

    fn expect_int_literal(arg: &str) -> Result<i32, ParserError> {
        if arg.starts_with("0x") {
            return match i32::from_str_radix(&arg[2..], 16) {
                Ok(arg) => Ok(arg),
                Err(err) => Err(InvalidIntLiteral(err)),
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
        Err(_NotAnInteger)
    }

    fn expect_ident_and_int<'a>(verb: &'a str, args: &'a [&'a str]) -> Result<(&'a str, i32), ParserError> {
        if let &[ident, int] = args {
            let ident = Assembler::expect_ident(ident)?;
            let int = Assembler::expect_int_literal(int)?;
            Ok((ident, int))
        } else {
            Err(SyntaxError(format!("{} expects ident + integer literal: {:?}", verb, args)))
        }
    }

    fn sizeof_name(var_name: &str) -> String {
        format!(".sizeof.{}", var_name)
    }
}
