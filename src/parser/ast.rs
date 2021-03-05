pub struct Program {
    pub entry: FuncDef,
    pub func_defs: Vec<FuncDef>,
}

pub struct FuncDef {
    pub name: String,
    pub params: Vec<VarDef>,
    pub locals: Vec<VarDef>,
    pub body: Vec<Stmt>,
}

pub struct VarDef {
    pub name: String,
    pub ty: VarType,
}

pub enum VarType {
    I32,
    I32Array { len: i32 },
}

pub enum Stmt {
    Assignment { target: AssnTarget, value: Expr },
    Expr { expr: Expr },
    Return { retval: Expr },
}

pub enum AssnTarget {
    Variable { name: String },
    ArrayItem { array_name: String, index: Expr },
}

pub enum Expr {
    Literal { val: i32 },
    Variable { name: String },
    BinExpr { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    FuncCall { func_name: String, args: Vec<Expr> },
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Sar,
}

pub enum Node {
    Program(Program),
    FuncDef(FuncDef),
    VarDef(VarDef),
    VarType(VarType),
    Stmt(Stmt),
    AssnTarget(AssnTarget),
    Expr(Expr),
    BinOp(BinOp),
}
