use crate::parser::ParserError::SyntaxError;
use crate::tokenizer::{Token, tokenize, TokenType};

#[allow(unused)]
mod ast;

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


#[derive(Debug)]
enum ParserError {
    SyntaxError { last_state: ParserState, tokens_left: Vec<Token> },
}


#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum ParserState {
    start,
    program,
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
            (func_defs, 0),
        (program, []) =>
            (accept, 0),

        (func_defs, [(Keyword, "fn"), ..]) =>
            (func_def, 0),
        (func_defs, []) =>
            (program, 0),

        (func_def, [(Keyword, "fn"), ..]) =>
            (param_list, 3),
        (func_def, [(RParen, ")"), ..]) =>
            (ret_ty, 1),
        (func_def, [(LBrac, "{"), ..]) =>
            (func_body, 1),
        (func_def, [(RBrac, "}"), ..]) =>
            (func_defs, 1),

        (ret_ty, [(RArrow, "->"), ..]) =>
            (ret_ty, 2),
        (ret_ty, [(LBrac, "{"), ..]) =>
            (func_def, 0),

        (param_list, [(Ident, _), ..]) =>
            (params, 0),
        (param_list, [_, ..]) =>
            (func_def, 0),

        (params, [(Ident, _), ..]) =>
            (param, 0),

        (param, [(Ident, _)]) =>
            (ty, 2),

        (func_body, [(Keyword, "let"), ..]) =>
            (local_defs, 0),
        (func_body, [(RBrac, "}"), ..]) =>
            (func_def, 0),
        (func_body, [_, ..]) =>
            (stmts, 0),

        (local_defs, [(Keyword, "let"), ..]) =>
            (local_def, 0),
        (local_defs, [_, ..]) =>
            (func_body, 0),

        (local_def, [(Keyword, "let"), ..]) =>
            (ty, 3),
        (local_def, [(Semi, ";"), ..]) =>
            (local_defs, 1),

        (ty, [(Keyword, "i32"), (Semi, ";"), ..]) =>
            (local_def, 1),
        (ty, [(LSqBrac, "["), ..]) =>
            (ty, 3),

        (stmts, [(RBrac, "}"), ..]) =>
            (func_body, 0),
        (stmts, [_, ..]) =>
            (stmt, 0),

        (stmt, [(Keyword, "return"), ..]) =>
            (expr, 1),
        (stmt, [(Ident, _), (Eq, "="), ..]) =>
            (assn, 0),
        (stmt, [(Semi, ";"), ..]) =>
            (stmts, 1),

        (assn, [(Ident, _), (Eq, "="), ..]) =>
            (assn_target, 0),
        (assn, [(Eq, "="), ..]) =>
            (expr, 1),

        (assn_target, [(Ident, _), (LSqBrac, "["), ..]) =>
            (expr, 2),
        (assn_target, [(Ident, _), ..]) =>
            (assn, 1),
        (assn_target, [(RSqBrac, "]"), ..]) =>
            (assn, 1),

        (expr, [(Literal, _), ..]) =>
            (literal, 0),
        (expr, [(Semi, ";"), ..]) =>
            (stmt, 0),

        (literal, [(Literal, _), ..]) =>
            (expr, 1),

        // ...

        _ => (reject, 0),
    }
}

type Transition<'a> = (ParserState, &'a [(TokenType, &'a str)]);


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
    if state == ParserState::reject {
        return Err(SyntaxError {
            last_state: transitions.last().unwrap().0,
            tokens_left: tokens.iter().map(Token::from).collect(),
        });
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
        match parse(&tokens) {
            Err(e) => panic!("parser error: {:?}", e),
            _ => {}
        }
    }
}