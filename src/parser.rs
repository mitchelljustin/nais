use crate::tokenizer::{Token, tokenize, TokenType};
use std::num::ParseIntError;
use crate::parser::ParserError::NoTransitionForToken;

enum ParserError {
    NoTransitionForToken(Token),
    CouldNotParseLiteral(ParseIntError),
}

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
}

struct Parser {}

/*
start -> program
program -> entry func_defs

entry -> "fn" "main" "(" ")" "{" func_body "}"

func_body -> local_defs stmts

local_defs -> local_def local_defs
local_defs -> ""

local_def -> "let" IDENT ":" ty ";"

stmts -> stmt stmts
stmts -> ""

stmt -> assn ";"
stmt -> expr ";"
stmt -> "return" expr ";"

assn -> assn_target "=" expr
assn_target -> IDENT
assn_target -> IDENT "[" expr "]"

expr -> "(" expr ")"
expr -> LITERAL
expr -> IDENT
expr -> bin_expr
expr -> func_call

bin_expr -> expr OP expr

func_call -> IDENT "(" args ")"

args -> arg "," args
args -> ""

arg -> expr

func_defs -> func_def func_defs
func_defs -> ""

func_def -> "fn" IDENT "(" param_defs ")" retval_def "{" func_body "}"

retval_def -> ""
retval_def -> "->" "i32"

param_defs -> param_def "," param_defs
param_defs -> ""

param_def -> IDENT ":" ty

ty -> "i32"
ty -> "[" "i32" ";" LITERAL "]"

*/

impl Parser {
    fn new() -> Parser {
        Parser {}
    }

    fn parse(&mut self, tokens: &[Token]) -> Result<ast::Node, ParserError> {
        let token_tys = tokens.iter().map(|t| t.ty).collect::<Vec<_>>();
        let node = match token_tys[..] {
            [TokenType::Literal] =>
                ast::Node::Expr(ast::Expr::Literal { val: 3 }),
            _ => return Err(NoTransitionForToken(tokens[0].clone()))
        };
        Ok(node)
    }
}


mod tests {
    use crate::tokenizer::dump_tokens;

    use super::*;

    #[test]
    fn test_simple() {
        let tokens = tokenize(&"
            fn main() {
                return 0;
            }
        ").unwrap();
        let mut parser = Parser::new();
        println!("{}", dump_tokens(&tokens));
        parser.parse(&tokens);
    }
}