use std::{cmp, fmt, fs};
use std::fmt::{Formatter, Write};
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
                    writeln!(f, "{}. {}", index + 1, err)?;
                }
                Ok(())
            }
            ASMParserErrors(errors) => {
                for (loc, err) in errors.iter() {
                    writeln!(f, "Line {}: {}", loc + 1, err)?;
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
    StructureError(String),
    MultipleErrors(Vec<ParserError>),

    _NotAnInteger,
}

pub struct AssemblyResult {
    pub binary: Vec<i32>,
    pub debug_info: DebugInfo,
    pub expanded_source: String,
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


#[derive(Clone)]
struct ForLoop {
    counter_var: String,
    limit_var: String,
    label_name: String,
}

struct Assembler {
    errors: Vec<(usize, ParserError)>,
    linker: Linker,

    line_no: usize,
    expanded_source: String,

    frame_extra_setup: String,
    frame_nloops: usize,
    frame_cur_loop: Option<ForLoop>,
}

impl Assembler {

    pub fn new() -> Assembler {
        Assembler {
            linker: Linker::new(),
            errors: Vec::new(),
            line_no: 0,

            expanded_source: String::new(),

            frame_extra_setup: String::new(),
            frame_nloops: 0,
            frame_cur_loop: None,
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
        for (i, line) in text.lines().enumerate() {
            self.line_no = i + 1;
            match self.process_line(line) {
                Err(e) => self.errors.push((self.line_no, e)),
                _ => {}
            }
        }
    }

    fn process_line(&mut self, line: &str) -> Result<(), ParserError> {
        write!(self.expanded_source, "{}\n", line).unwrap();
        let line = line.to_string();
        let line = line.split(";").next().unwrap(); // Remove comments
        let words: Vec<&str> = line.split_ascii_whitespace().collect();
        if words.len() == 0 {
            return Ok(());
        }
        let verb = words[0];
        let args = &words[1..];
        self.process_statement(verb, args)?;
        Ok(())
    }

    fn process_internal(&mut self, text: &str) -> Result<(), ParserError> {
        self.process_line("; BEGIN {{")?;
        for line in text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                self.process_line(&format!("    {}", line))?;
            }
        }
        self.process_line("; }} END")?;
        Ok(())
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
        let expanded_source = self.expanded_source;
        Ok(AssemblyResult {
            expanded_source,
            binary,
            debug_info,
        })
    }

    fn process_statement(&mut self, verb: &str, args: &[&str]) -> Result<(), ParserError> {
        match verb {
            label_name if label_name.ends_with(":") =>
                self.process_label(&label_name[..label_name.len() - 1]),
            macro_name if macro_name.starts_with(".") =>
                self.process_macro(macro_name, args),
            op_name =>
                self.process_instruction(op_name, args),
        }
    }

    fn process_label(&mut self, label_name: &str) -> Result<(), ParserError> {
        if label_name.starts_with("_") {
            self.linker.add_inner_label(label_name);
        } else {
            self.linker.add_top_level_label(label_name);
        }
        Ok(())
    }

    fn process_macro(&mut self, macro_name: &str, args: &[&str]) -> Result<(), ParserError> {
        match macro_name {
            ".define" => {
                let (name, value) = Assembler::expect_ident_and_int(macro_name, args)?;
                self.linker.add_global_constant(name, value);
                Ok(())
            }
            ".param" => {
                let (name, size) = Assembler::expect_ident_and_int(macro_name, args)?;
                self.linker.add_param(name, size);
                self.linker.add_local_constant(&Assembler::sizeof_name(name), size);
                Ok(())
            }
            ".local" => {
                let (name, size) = Assembler::expect_ident_and_int(macro_name, args)?;
                self.linker.add_local_var(name, size);
                self.linker.add_local_constant(&Assembler::sizeof_name(name), size);
                Ok(())
            }
            ".word" => self.process_word_macro(args),
            ".string" => {
                for arg in args {
                    if let Ok(val) = Assembler::expect_int_literal(arg) {
                        self.linker.add_raw_word(val);
                        continue;
                    }
                    let string = Assembler::expect_string_literal(arg)?;
                    for ch in string.chars() {
                        self.linker.add_raw_word(ch as i32);
                    }
                }
                Ok(())
            }
            ".start_frame" => {
                let extra_code = self.frame_extra_setup.clone();
                let size = self.linker.cur_frame().locals_size;
                self.process_internal(&format!("
                    loadi fp
                    loadi sp
                    storei fp
                    addsp {size}
                    {extra_code}
                ", extra_code = extra_code, size = size))?;

                self.frame_extra_setup.clear();
                Ok(())
            }
            ".end_frame" => {
                self.process_internal(&format!("
                    addsp -{size}
                    storei fp
                ", size = self.linker.cur_frame().locals_size))?;

                self.frame_nloops = 0;
                Ok(())
            }
            ".addr_of" => {
                if args.len() != 1 {
                    return Err(SyntaxError(format!("{} takes 1 arg: {:?}", macro_name, args)));
                }
                let var_name = Assembler::expect_ident(args[0])?;
                let addr_name = format!("{}.addr", var_name);
                self.linker.add_local_var(&addr_name, 1);
                write!(self.frame_extra_setup, "
                    loadi fp
                    addi {var_name}
                    storef {addr_name}
                ", var_name = var_name, addr_name = addr_name).unwrap();
                Ok(())
            }
            ".for" => {
                if args.len() != 4 || args[2] != "to" {
                    return Err(SyntaxError(format!("{} format: `var` `init` to `limit`", macro_name)));
                }
                let counter_var = Assembler::expect_ident(args[0])?.to_string();
                let init_val = Assembler::expect_int_literal(args[1])?.to_string();
                let limit_var = Assembler::expect_ident(args[3])?.to_string();
                let label_name = format!("_loop.{}", self.frame_nloops);
                self.frame_nloops += 1;
                self.process_internal(&format!("
                    push {init_val}
                    storef {counter_var}
                    {label_name}:
                ", counter_var = counter_var, init_val = init_val, label_name = label_name))?;
                self.frame_cur_loop = Some(ForLoop {
                    counter_var,
                    limit_var,
                    label_name,
                });
                Ok(())
            }
            ".end_for" => {
                match self.frame_cur_loop.clone() {
                    None =>
                        return Err(StructureError("no current for loop to end".to_string())),
                    Some(ForLoop { label_name, counter_var, limit_var }) =>
                        self.process_internal(&format!("
                            loadf {counter_var}
                            addi 1
                            storef {counter_var}
                            loadf {counter_var}
                            loadf {limit_var}
                            blt {label_name}
                        ", counter_var = counter_var, limit_var = limit_var, label_name = label_name))?
                }
                self.frame_cur_loop = None;
                Ok(())
            }
            ".call" => self.process_call_macro(args),
            unknown => Err(UnknownMacro(unknown.to_string())),
        }
    }

    fn process_call_macro(&mut self, args: &[&str]) -> Result<(), ParserError> {
        if args.len() == 0 {
            return Err(SyntaxError(format!(".call takes at least 1 arg")));
        }
        let mut code = String::new();
        let call_target = args[0];
        let mut call_args = args[1..].to_vec();
        let nargs = call_args.len();
        call_args.reverse();
        let mut ret_target = None;
        for arg in call_args.into_iter() {
            if let &[ty, val] = &arg.split(":").collect::<Vec<_>>()[..] {
                let op_name = match ty {
                    "lf" => "loadf",
                    "p" => "push",
                    "ret" => {
                        ret_target = Some(val);
                        continue;
                    }
                    _ => return Err(SyntaxError(format!("unexpected ty: {}", ty)))
                };
                write!(code, "{} {}\n", op_name, val).unwrap();
            } else {
                return Err(SyntaxError(format!("expected T:VAL format: {}", arg)));
            }
        }
        if call_target.starts_with("env.") {
            let env_call_name = &call_target[4..];
            let epilogue = match ret_target {
                None =>
                    "addsp -1".to_string(),
                Some(target) => format!("
                    storef {}
                ", target),
            };
            write!(code, "
                ecall .cc.{}
                {}
            ", env_call_name, epilogue).unwrap();
        } else {
            let epilogue = match ret_target {
                None => format!("
                    addsp -{}
                ", nargs + 1),
                Some(target) => format!("
                    storef {}
                    addsp -{}
                ", target, nargs - 1)
            };
            write!(code, "
                push
                jal {call_target}
                {epilogue}
            ", call_target = call_target, epilogue = epilogue).unwrap();
        }
        self.process_internal(&code)
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
        if !first_char.is_alphabetic() && first_char != '_' {
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
