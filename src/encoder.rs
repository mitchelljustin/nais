use std::collections::HashMap;

use crate::isa::{Inst, Operation, OP_LIST};

#[derive(Clone)]
pub struct Encoder {
    pub name_to_op: HashMap<&'static str, &'static Operation>,
    pub op_to_opcode: HashMap<&'static str, u8>,
    pub opcode_to_op: HashMap<u8, &'static Operation>,
}

impl Encoder {
    pub fn new() -> Encoder {
        let mut enc = Encoder {
            name_to_op: HashMap::new(),
            op_to_opcode: HashMap::new(),
            opcode_to_op: HashMap::new(),
        };
        for (i, op) in OP_LIST.iter().enumerate() {
            let opcode = i as u8;
            enc.name_to_op.insert(op.name, op);
            enc.op_to_opcode.insert(op.name, opcode);
            enc.opcode_to_op.insert(opcode, op);
        }
        enc
    }

    pub fn make_inst(&self, op_name: &str, arg: i32) -> Option<Inst> {
        match self.name_to_op.get(op_name) {
            None => return None,
            Some(&op) => {
                let opcode = *self.op_to_opcode.get(op_name).unwrap();
                Some(Inst {
                    addr: None,
                    op,
                    opcode,
                    arg,
                })
            }
        }
    }

    pub fn encode(&self, inst: &Inst) -> i32 {
        let opcode = inst.opcode as i32;
        let arg_part = inst.arg & 0xffffff;
        let bin_inst = (opcode << 24) | (arg_part);
        bin_inst
    }

    pub fn decode(&self, bin_inst: i32) -> Option<Inst> {
        let opcode = ((bin_inst >> 24) & 0xff) as u8;
        let mut arg = bin_inst & 0xffffff;
        if arg >> 23 != 0 {
            // sign extend
            arg |= 0xff000000u32 as i32;
        }
        let op = *self.opcode_to_op.get(&opcode)?;
        Some(Inst {
            addr: None,
            opcode,
            op,
            arg,
        })
    }
}
