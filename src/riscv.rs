/*
0 	Zero 	Always zero
x1 	ra 	Return address 	Caller
x2 	sp 	Stack pointer 	Callee
x3 	gp 	Global pointer
x4 	tp 	Thread pointer
x5 	t0 	Temporary / alternate return address 	Caller
x6–7 	t1–2 	Temporary 	Caller
x8 	s0/fp 	Saved register / frame pointer 	Callee
x9 	s1 	Saved register 	Callee
x10–11 	a0–1 	Function argument / return value 	Caller
x12–17 	a2–7 	Function argument 	Caller
x18–27 	s2–11 	Saved register 	Callee
x28–31 	t3–6 	Temporary 	Caller
 */


macro_rules! reg_alias {
    (zero) => (0);
    (ra) => (1);
    (sp) => (2);
    (gp) => (3);
    (tp) => (4);
    (t0) => (5);
    (t1) => (6);
    (t2) => (7);
    (s0) => (8);
    (fp) => (8);
    (s1) => (9);
    (a0) => (10);
    (a1) => (11);
    (a2) => (12);
    (a3) => (13);
    (a4) => (14);
    ($x:ident) => {
        compile_error!("not a valid register alias");
    };
}

macro_rules! exec_instr {
    ($vm:ident mv $rd:ident $rs1:ident ) => { // psuedo
        $vm.add(reg_alias!($rd), 0, reg_alias!($rs1));
    };
    ($vm:ident mvi $rd:ident $imm:literal ) => { // psuedo
        $vm.addi(reg_alias!($rd), 0, $imm);
    };
    ($vm:ident $op:ident $( $reg:ident )* $( $imm:literal )*) => {
        $vm.$op ( $(reg_alias!($reg),)* $($imm,)* );
    };
}

macro_rules! run_asm {
    { in $vm:ident
        $($op:ident $( $reg:ident )* $( $imm:literal )*);+;
    } => {
        $(exec_instr! {$vm $op $( $reg )* $( $imm )*})+
    };
}

macro_rules! impl_op {
    ( [R] $mnem:ident $op:expr ) => {
        pub fn $mnem(&mut self, i_rd: i32, i_rs1: i32, i_rs2: i32) {
            let rs1: i32 = *self.reg(i_rs1);
            let rs2: i32 = *self.reg(i_rs2);
            *self.reg(i_rd) = $op(rs1, rs2);
            self.pc += 4;
        }
    };
    ( [I] $mnem:ident $op:expr ) => {
        pub fn $mnem(&mut self, i_rd: i32, i_rs1: i32, imm: i32) {
            assert!(-(1 << 11) <= imm && imm <= (1 << 11 - 1), "imm out of range (12 bits)");
            let rs1: i32 = *self.reg(i_rs1);
            *self.reg(i_rd) = $op(rs1, imm);
            self.pc += 4;
        }
    };
}

macro_rules! impl_ops {
    ($( [$opty:ident] $mnem:ident $op:expr ) ;+;) => {
        $(impl_op! ([$opty] $mnem $op));+;
    };
}

#[derive(Debug)]
pub struct Machine {
    pc: i32,
    regs: [i32; 32],
    mem_size: usize,
    mem: Vec<i32>,
}

impl Machine {
    pub fn new(memsize: usize) -> Machine {
        Machine {
            pc: 0,
            regs: [0i32; 32],
            mem: Vec::with_capacity(memsize),
            mem_size: memsize,
        }
    }

    fn reg(&mut self, idx: i32) -> &mut i32 {
        return &mut self.regs[idx as usize];
    }

    impl_ops! {
        [R] add  |rs1, rs2| rs1 + rs2;
        [R] sub  |rs1, rs2| rs1 - rs2;
        [R] mul  |rs1, rs2| rs1 * rs2;
        [R] div  |rs1, rs2| rs1 / rs2;
        [R] and  |rs1, rs2| rs1 & rs2;
        [R] or   |rs1, rs2| rs1 | rs2;
        [R] xor  |rs1, rs2| rs1 ^ rs2;

        [I] subi |rs1, imm| rs1 - imm;
        [I] addi |rs1, imm| rs1 + imm;
        [I] muli |rs1, imm| rs1 * imm;
        [I] divi |rs1, imm| rs1 / imm;
        [I] andi |rs1, imm| rs1 & imm;
        [I] ori  |rs1, imm| rs1 | imm;
        [I] xori |rs1, imm| rs1 ^ imm;

        [I] slti |rs1, imm| if rs1 < imm {0} else {1};
        [I] sltiu |rs1, imm| if rs1 < imm {0} else {1}; // TODO

        [I] slli |rs1, imm| rs1 << imm;
        [I] srli |rs1, imm| rs1 >> imm;
        [I] srai |rs1, imm| rs1 << imm; // TODO
    }
}