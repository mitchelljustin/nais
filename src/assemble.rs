use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;

use crate::assemble::AssemblyError::MissingTarget;
use crate::constants::SEG_CODE_START;
use crate::isa::{Encoder, Inst, OP_INVALID};
use crate::isa;
use crate::machine::DebugInfo;

macro_rules! parse_asm_line {
    ( $p:ident label $label:ident ) => {
        $p.add_label(stringify!($label));
    };
    ( $p:ident inner $label:ident ) => {
        $p.add_inner_label(stringify!($label));
    };
    ( $p:ident global $name:ident $loc:literal ) => {
        $p.add_global_var(stringify!($name), $loc);
    };
    ( $p:ident const $name:ident $value:literal ) => {
        $p.add_constant(stringify!($name), $value);
    };
    ( $p:ident arg $($name:ident)+ ) => {
        $(
            $p.add_arg_var(stringify!($name), 1);
        )+
    };
    ( $p:ident local $($name:ident)+ ) => {
        $(
            $p.add_local_var(stringify!($name), 1);
        )+
    };
    ( $p:ident array $name:ident $size:literal ) => {
        $p.add_local_var(stringify!($name), $size);
    };
    ( $p:ident start_frame ) => {
         $p.start_frame();
    };
    ( $p:ident loadf $name:ident ) => {
        parse_asm_line!($p loadi fp );
        parse_asm_line!($p load $name );
    };
    ( $p:ident storef $name:ident ) => {
        parse_asm_line!($p loadi fp );
        parse_asm_line!($p store $name );
    };
    ( $p:ident end_frame ) => {
         $p.end_frame();
    };
    ( $p:ident $mnem:ident $label:ident ) => {
        $p.add_placeholder_inst(stringify!($mnem), stringify!($label));
    };
    ( $p:ident $mnem:ident ) => {
        parse_asm_line!($p $mnem 0);
    };
    ( $p:ident $mnem:ident $arg:literal ) => {
        $p.add_inst(stringify!($mnem), $arg);
    };
}

macro_rules! add_asm {
 ( [program: $p:ident] $( $mnem:ident $($label:ident)* $($a:literal)* );+; ) => {
       $(
            parse_asm_line!($p $mnem $($label)* $($a)*);
       )+
    };
}

macro_rules! program_from_asm {
    ( $( $mnem:ident $($label:ident)* $($a:literal)* );+; ) => {
       {
           let mut p = Program::new();
           p.init();
           add_asm! {
               [program: p]
               $(
                    $mnem $($label)* $($a)*;
               )+
           }
           p
       }
    };
}

#[derive(Copy, Clone)]
pub enum LabelType {
    Constant,
    Global,
    Code,
    InnerCode,
    Frame,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LabelType::Constant => "C",
            LabelType::Global => "G",
            LabelType::Code => "L",
            LabelType::InnerCode => "I",
            LabelType::Frame => "F",
        })
    }
}


#[derive(Clone)]
struct LabelEntry {
    code_loc: i32,
    locals_size: i32,
    args_size: i32,
    frame_labels: HashMap<String, i32>,
}

#[derive(Clone, Debug)]
pub enum AssemblyError {
    MissingTarget(Inst, String),
    NoSuchOp(usize, String),
}

impl Display for AssemblyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MissingTarget(inst, target) => {
                write!(f, "MissingTarget({} \"{}\")", inst, target)
            }
            other => write!(f, "{:?}", other)
        }
    }
}

#[derive(Clone)]
pub struct Program {
    instructions: Vec<Inst>,
    scope_labels: HashMap<String, LabelEntry>,
    global_vars: HashMap<String, i32>,
    constants: HashMap<String, i32>,
    inst_scope: Vec<String>,
    cur_scope_label: String,
    reloc_tab: HashMap<usize, String>,
    resolved_labels: HashMap<String, (i32, LabelType)>,
    encoder: Encoder,
    errors: Vec<AssemblyError>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            instructions: Vec::new(),
            scope_labels: HashMap::new(),
            global_vars: HashMap::new(),
            constants: HashMap::new(),
            inst_scope: Vec::new(),
            reloc_tab: HashMap::new(),
            resolved_labels: HashMap::new(),
            cur_scope_label: String::new(),
            encoder: Encoder::new(),
            errors: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.add_global_var("pc", 0);
        self.add_global_var("sp", 1);
        self.add_global_var("fp", 2);
        self.add_label("_entry");
        for (callcode, name) in isa::ENV_CALLS.iter().enumerate() {
            self.add_constant(name, callcode as i32);
        }
    }

    pub fn add_inst(&mut self, opname: &str, arg: i32) {
        let loc = self.instructions.len();
        let addr = Some(SEG_CODE_START + loc as i32);
        let inst = match self.encoder.make_inst(opname, arg) {
            None => {
                self.errors.push(AssemblyError::NoSuchOp(
                    loc, String::from(opname)));
                self.instructions.push(Inst {
                    opcode: 0x00,
                    op: OP_INVALID,
                    arg,
                    addr,
                });
                return;
            }
            Some(inst) => inst
        };
        self.instructions.push(Inst {
            addr,
            ..inst
        });
        self.inst_scope.push(self.cur_scope_label.clone());
    }

    pub fn add_placeholder_inst(&mut self, opname: &str, label: &str) {
        self.reloc_tab.insert(self.instructions.len(), String::from(label));
        self.add_inst(opname, 0);
    }

    pub fn add_label(&mut self, name: &str) {
        self.add_label_entry(name);
        self.cur_scope_label = String::from(name);
    }

    pub fn add_inner_label(&mut self, name: &str) {
        self.add_label_entry(&Program::make_inner_label(&self.cur_scope_label, name));
    }

    fn add_label_entry(&mut self, name: &str) {
        self.scope_labels.insert(String::from(name), LabelEntry {
            code_loc: self.instructions.len() as i32,
            frame_labels: HashMap::new(),
            locals_size: 0,
            args_size: 0,
        });
    }

    fn cur_label_entry(&mut self) -> &mut LabelEntry {
        self.scope_labels
            .get_mut(&self.cur_scope_label)
            .expect("current label not found")
    }

    pub fn add_global_var(&mut self, name: &str, abs_loc: i32) {
        self.global_vars.insert(
            String::from(name),
            abs_loc,
        );
    }

    pub fn add_local_var(&mut self, name: &str, sz: i32) {
        let label_entry = self.cur_label_entry();
        label_entry.frame_labels.insert(
            String::from(name),
            label_entry.locals_size,
        );
        label_entry.locals_size += sz;
        if sz > 1 {
            label_entry.frame_labels.insert(
                format!("{}_len", name),
                sz,
            );
        }
    }

    pub fn add_arg_var(&mut self, name: &str, sz: i32) {
        let label_entry = self.cur_label_entry();
        label_entry.frame_labels.insert(
            String::from(name),
            -label_entry.args_size - 3, // [..args retaddr savedfp || locals ]
        );
        label_entry.args_size += sz;
    }

    pub fn add_constant(&mut self, name: &str, value: i32) {
        self.constants.insert(String::from(name), value);
    }

    pub fn start_frame(&mut self) {
        self.add_placeholder_inst("loadi", "fp");
        self.add_placeholder_inst("loadi", "sp");
        self.add_placeholder_inst("storei", "fp");
        let extend_sz = self.cur_label_entry().locals_size;
        if extend_sz > 0 {
            self.add_inst("extend", extend_sz);
        }
    }

    pub fn end_frame(&mut self) {
        let drop_sz = self.cur_label_entry().locals_size;
        if drop_sz > 0 {
            self.add_inst("drop", drop_sz);
        }
        self.add_placeholder_inst("storei", "fp");
    }

    fn resolve_label(&self, target: &str, inst_loc: usize) -> Option<(i32, LabelType)> {
        if let Some(&entry) = self.resolved_labels.get(target) {
            return Some(entry);
        }
        // Constant
        if let Some(&value) = self.constants.get(target) {
            return Some((value, LabelType::Constant));
        }
        // Code label
        if let Some(label_entry) = self.scope_labels.get(target) {
            let value = Program::calc_inst_offset(
                label_entry.code_loc,
                inst_loc,
            );
            return Some((value, LabelType::Code));
        }
        // Global variable
        if let Some(&value) = self.global_vars.get(target) {
            return Some((value, LabelType::Global));
        }
        // Local scope
        let scope_label = &self.inst_scope[inst_loc];
        let scope_entry = self.scope_labels.get(scope_label).unwrap();
        let inner_label_name = Program::make_inner_label(scope_label, target);
        // Local code (inner label)
        if let Some(label_entry) = self.scope_labels.get(&inner_label_name) {
            let value = Program::calc_inst_offset(
                label_entry.code_loc,
                inst_loc,
            );
            return Some((value, LabelType::InnerCode));
        }
        // Local frame var
        if let Some(&value) = scope_entry.frame_labels.get(target) {
            return Some((value, LabelType::Frame));
        }
        None
    }

    pub fn relocate(&mut self) -> Vec<(usize, String)> {
        let mut unrelocated = Vec::<(usize, String)>::new();
        let mut arg_updates = Vec::<(usize, i32)>::new();
        for (inst_loc, target) in self.reloc_tab.iter() {
            match (&self).resolve_label(target, *inst_loc) {
                Some((value, label_type)) => {
                    self.resolved_labels.insert(target.clone(), (value, label_type));
                    arg_updates.push((*inst_loc, value));
                }
                None => {
                    unrelocated.push((*inst_loc, target.clone()));
                }
            }
        }
        for (loc, arg) in arg_updates.into_iter() {
            self.instructions[loc].arg = arg;
        }
        unrelocated
    }

    pub fn assemble(&mut self) -> Result<Vec<i32>, Vec<AssemblyError>> {
        let mut errors: Vec<AssemblyError> = self.errors
            .drain(..)
            .collect();

        let unrelocated = self.relocate();
        errors.extend(
            unrelocated.into_iter()
                .map(|(loc, target)|
                    MissingTarget(self.instructions[loc], target.clone()))
        );
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(
            self.instructions.iter()
                .map(|inst| self.encoder.encode(inst))
                .collect()
        )
    }

    fn calc_inst_offset(target_loc: i32, inst_loc: usize) -> i32 {
        target_loc - (inst_loc as i32) - 1
    }

    fn make_inner_label(scope_label: &str, name: &str) -> String {
        format!("{}.{}", scope_label, name)
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.instructions
            .iter()
            .map(|inst| inst.to_string())
            .collect::<Vec<String>>()
            .join("\n")
        )
    }
}

impl DebugInfo for Program {
    fn label_for_inst(&self, addr: i32) -> Option<(String, String)> {
        let loc = (addr - SEG_CODE_START) as usize;
        let label = match self.reloc_tab.get(&loc) {
            None => return None,
            Some(l) => l.clone()
        };
        let label_type = match self.resolved_labels.get(&label) {
            None => return None,
            Some((_, t)) => *t
        };
        Some((label, label_type.to_string()))
    }

    fn scope_for_inst(&self, addr: i32) -> Option<String> {
        let loc = (addr - SEG_CODE_START) as usize;
        self.inst_scope.get(loc).map(|s| s.clone())
    }

    fn value_for_label(&self, name: &str) -> Option<(i32, String)> {
        self.resolved_labels.get(name)
            .map(|&(val, t)| (val, t.to_string()))
    }
}