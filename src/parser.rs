use crate::tokenizer::{Token, tokenize, TokenType};

enum ParserError {
}

#[allow(unused)]
mod ast {
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
}

/*
start -> program
program -> func_defs

func_defs -> func_def func_defs
func_defs -> ""

func_def -> "fn" IDENT "(" param_list ")" ret_ty "{" func_body "}"

param_list -> params
param_list -> ""

params -> param "," params
params -> param

param -> IDENT ":" ty

ty -> "i32"
ty -> "[" "i32" ";" LITERAL "]"

ret_ty -> ""
ret_ty -> "->" "i32"

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

bin_expr -> product "*" expr
bin_expr -> product "/" expr
product -> expr

bin_expr -> term "+" expr
bin_expr -> term "-" expr
term -> expr

func_call -> IDENT "(" arg_list ")"

arg_list -> ""
arg_list -> args

args -> arg "," args
args -> arg

arg -> expr

*/

#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum ParserState {
    start,
    program,
    main_def,
    func_body,
    local_defs,
    local_def,
    stmts,
    stmt,
    assn,
    assn_target,
    expr,
    bin_expr,
    product,
    term,
    func_call,
    arg_list,
    args,
    arg,
    func_defs,
    func_def,
    param_list,
    params,
    param,
    ty,
    ret_ty,

    literal,
    ident,

    accept,
    reject,
}


fn parser_transition(state: ParserState, tokens: &[(TokenType, &str)]) -> (ParserState, usize) {
    use ParserState::*;
    use TokenType::*;
    match (state, tokens) {
        (start, _) =>
            (program, 0),
        (program, [(Keyword, "fn"), ..]) =>
            (main_def, 0),
        (program, [(Keyword, "fn"), ..]) =>
            (func_defs, 0),
        (program, []) =>
            (accept, 0),
        (main_def, [(Keyword, "fn"), ..]) =>
            (func_body, 5),
        (func_body, [(Keyword, "let"), ..]) =>
            (local_defs, 0),
        (local_defs, [(Keyword, "let"), ..]) =>
            (local_def, 0),
        (local_def, [(Keyword, "let"), (Ident, _), (Colon, _), ..]) =>
            (ty, 3),
        // ...
        (func_body, [(Keyword, "return"), ..]) =>
            (stmts, 0),
        (stmts, [(Keyword, "return"), ..]) =>
            (stmt, 0),
        (stmt, [(Keyword, "return"), ..]) =>
            (expr, 1),
        (expr, [(Literal, _), ..]) =>
            (literal, 0),
        (literal, [(Literal, _), (Semi, ";"), ..]) =>
            (stmt, 1),
        (stmt, [(Semi, ";"), ..]) =>
            (stmts, 1),
        (stmts, [(RBrac, "}")]) =>
            (func_body, 0),
        (func_body, [(RBrac, "}")]) =>
            (main_def, 1),
        (func_body, [(RBrac, "}"), ..]) =>
            (main_def, 1),
        (func_body, [(RBrac, "}"), ..]) =>
            (func_def, 1),
        (main_def, []) =>
            (program, 0),
        _ => (reject, 0),
    }
}


fn parse(tokens: &[Token]) -> Result<ast::Node, ParserError> {
    let tokens_as_tuples = tokens
        .iter()
        .map(|t| (t.ty, t.val.as_str()))
        .collect::<Vec<_>>();
    let mut tokens = &tokens_as_tuples[..];
    let mut state = ParserState::start;
    let mut transitions = Vec::new();
    while state != ParserState::accept && state != ParserState::reject {
        let (next_state, n_eat) = parser_transition(state, &tokens);
        let (eaten_tokens, next_tokens) = tokens.split_at(n_eat);
        let transition = (state, eaten_tokens);
        println!("{:?}", transition);
        transitions.push(transition);
        tokens = next_tokens;
        state = next_state;
    }

    let node = ast::Node::Expr(ast::Expr::Literal { val: 1 });
    Ok(node)
}


mod tests {
    use crate::tokenizer::dump_tokens;

    use super::*;

    #[test]
    fn test_simple() {
        let code = "
        fn main() {
            let x: i32;
            x = 3;
        }
        ";
        println!("{}", code);
        let tokens = tokenize(&code).unwrap();
        println!("{}\n", dump_tokens(&tokens));
        parse(&tokens);
    }
}