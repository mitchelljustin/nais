mod ast {
    pub struct Program {
        pub main: FuncDef,
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
        I32Array { size: i32 },
    }

    pub enum Stmt {
        Assignment { target: AssnTarget, value: Expr },
        Expr { expr: Expr },
        Return { retval: Expr },
    }

    pub enum AssnTarget {
        Variable { name: String },
        ArrayItem { array: Expr, index: Expr },
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
}

fn parse(text: &str) -> ast::Program {
    return ast::Program {
        main: ast::FuncDef {
            name: "main".to_string(),
            params: Vec::new(),
            locals: Vec::new(),
            body: vec![
                ast::Stmt::Return {
                    retval: ast::Expr::Literal {
                        val: 0,
                    }
                }
            ],
        },
        func_defs: Vec::new(),
    };
}


mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        parse("");
    }
}