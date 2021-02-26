use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};

use crate::isa::{Inst, Encoder};

macro_rules! parse_asm_line {
    ( $p:ident label $label:ident ) => {
        $p.add_label(stringify!($label));
    };
    ( $p:ident inner $label:ident ) => {
        $p.add_inner_label(stringify!($label));
    };
    ( $p:ident global $label:ident $loc:literal ) => {
        $p.add_global_var(stringify!($label), $loc);
    };
    ( $p:ident arg $label:ident ) => {
        $p.add_arg_var(stringify!($label));
    };
    ( $p:ident local $label:ident ) => {
        $p.add_local_var(stringify!($label));
    };
    ( $p:ident mov $dest:ident $src:ident ) => {
        parse_asm_line!($p load $src);
        parse_asm_line!($p store $dest);
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

macro_rules! assemble {
    ( $( $mnem:ident $($label:ident)* $($a:literal)* );+; ) => {
       {
           let mut p = Program::new();
           p.init();
           $(
                parse_asm_line!(p $mnem $($label)* $($a)*);
           )+
           p.relocate_all();
           p
       }
    };
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for (i, inst) in self.code.iter().enumerate() {
            if let Err(e) = write!(f, "1 {:04x} {}\n", i, inst) {
                return Err(e);
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct LabelEntry {
    code_loc: i32,
    nlocals: i32,
    nargs: i32,
    frame_labels: HashMap<String, i32>,
}


#[derive(Clone)]
pub struct Program {
    code: Vec<Inst>,
    labels: HashMap<String, LabelEntry>,
    globals: HashMap<String, i32>,
    inst_context: Vec<String>,
    cur_label: String,
    reloc_tab: Vec<(i32, String)>,
    encoder: Encoder,
}

impl Program {
    pub fn new() -> Program {
        Program {
            code: Vec::new(),
            labels: HashMap::new(),
            globals: HashMap::new(),
            inst_context: Vec::new(),
            reloc_tab: Vec::new(),
            cur_label: String::new(),
            encoder: Encoder::new(),
        }
    }

    pub fn init(&mut self) {
        self.add_global_var("pc", 0);
        self.add_global_var("sp", 1);
        self.add_global_var("fp", 2);
        self.add_label("_entry");
    }

    pub fn add_inst(&mut self, opname: &str, arg: i32) {
        let op = self.encoder.op_with_name(opname);
        self.code.push(Inst {
            op,
            arg,
        });
        self.inst_context.push(self.cur_label.clone());
    }

    pub fn add_placeholder_inst(&mut self, opname: &str, label: &str) {
        self.reloc_tab.push((self.code.len() as i32, String::from(label)));
        self.add_inst(opname, 0);
    }

    pub fn add_label(&mut self, name: &str) {
        self.add_inner_label(name);
        self.cur_label = String::from(name);
    }

    pub fn add_inner_label(&mut self, name: &str) {
        let name = String::from(name);
        self.labels.insert(name.clone(), LabelEntry {
            code_loc: self.code.len() as i32,
            frame_labels: HashMap::new(),
            nlocals: 0,
            nargs: 0,
        });
    }

    fn cur_label_entry(&mut self) -> &mut LabelEntry {
        self.labels
            .get_mut(&self.cur_label)
            .expect("current label not found")
    }

    pub fn add_global_var(&mut self, name: &str, abs_loc: i32) {
        self.globals.insert(
            String::from(name),
            abs_loc,
        );
    }

    pub fn add_local_var(&mut self, name: &str) {
        let label_entry = self.cur_label_entry();
        label_entry.frame_labels.insert(
            String::from(name),
            label_entry.nlocals,
        );
        label_entry.nlocals += 1;
    }

    pub fn add_arg_var(&mut self, name: &str) {
        let label_entry = self.cur_label_entry();
        label_entry.frame_labels.insert(
            String::from(name),
            -label_entry.nargs - 3,
        );
        label_entry.nargs += 1;
    }

    pub fn relocate_all(&mut self) {
        for (inst_loc, name) in self.reloc_tab.iter() {
            let inst = &mut self.code[*inst_loc as usize];

            // Code relocation
            if let Some(label_entry) = self.labels.get(name) {
                let offset = label_entry.code_loc - *inst_loc - 1;
                inst.arg = offset;
            }
            // Global relocation
            else if let Some(global_loc) = self.globals.get(name) {
                inst.arg = *global_loc;
            }
            // Local relocation
            else {
                let context_label = &self.inst_context[*inst_loc as usize];
                let label_entry = self.labels.get(context_label).unwrap();
                if let Some(frame_offset) = label_entry.frame_labels.get(name) {
                    inst.arg = *frame_offset;
                } else {
                    panic!("Label not found in code, globals or locals: {}", name)
                }
            }
        }
        self.reloc_tab.clear();
    }

    pub fn as_binary(&self) -> Vec<i32> {
        self.code.iter().map(|inst| {
            let opcode = self.encoder.opcode_for_op(inst.op) as i32;
            let arg_part = inst.arg & 0xffffff;
            let bin_inst = (opcode << 24) | (arg_part);
            bin_inst
        }).collect()
    }
}
