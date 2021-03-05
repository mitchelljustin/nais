use std::collections::HashMap;

use crate::tokenizer::{QuickToken, TokenType};

#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
pub(crate) enum State {
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

    START,
    ACCEPT,
    REJECT,
}


/*
(program, [])
(func_defs, [])
(func_def, [(Keyword, "fn"), (Ident, "main"), (LParen, "(")])
(param_list, [])
(ret_ty, [])
(func_body, [])
(local_defs, [])
(local_def, [(Keyword, "let"), (Ident, "x"), (Colon, ":")])
(ty, [(Keyword, "i32")])
(func_body, [])
(stmts, [])
(stmt, [])
(assn, [])
(assn_target, [(Ident, "x")])
(expr, [])
(literal, [(Literal, "3")])

 */

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
ret_ty -> "->" ty

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


pub(crate) fn state_transition(state: State, tokens: &[(TokenType, &str)]) -> (State, usize, bool) {
    use State::*;
    use TokenType::*;
    match (state, tokens) {
        (START, _) =>
            (program, 0, false),

        (program, [(Keyword, "fn"), ..]) =>
            (func_defs, 0, true),
        (program, []) =>
            (ACCEPT, 0, false),

        (func_defs, [(Keyword, "fn"), ..]) =>
            (func_def, 0, true),
        (func_defs, []) =>
            (program, 0, false),

        (func_def, [(Keyword, "fn"), ..]) =>
            (param_list, 3, true),
        (func_def, [(RParen, ")"), ..]) =>
            (ret_ty, 1, false),
        (func_def, [(LBrac, "{"), ..]) =>
            (func_body, 1, false),
        (func_def, [(RBrac, "}"), ..]) =>
            (func_def, 1, false),
        (func_def, []) =>
            (program, 0, false),

        (ret_ty, [(RArrow, "->"), ..]) =>
            (ty, 1, true),
        (ret_ty, [(LBrac, "{"), ..]) =>
            (func_def, 0, true),

        (param_list, [(Ident, _), ..]) =>
            (params, 0, true),
        (param_list, [_, ..]) =>
            (func_def, 0, true),

        (params, [(Ident, _), ..]) =>
            (param, 0, true),
        (params, [..]) =>
            (func_def, 0, false),

        (param, [(Ident, _), ..]) =>
            (ty, 2, true),
        (param, [(Comma, ","), ..]) =>
            (params, 1, false),

        (func_body, [(RBrac, "}"), ..]) =>
            (func_def, 0, false),
        (func_body, [_, ..]) =>
            (local_defs, 0, false),

        (local_defs, [(Keyword, "let"), ..]) =>
            (local_def, 0, true),
        (local_defs, [..]) =>
            (func_body, 0, false),

        (local_def, [(Keyword, "let"), ..]) =>
            (ty, 3, true),
        (local_def, [(Semi, ";"), ..]) =>
            (local_defs, 1, false),

        (ty, [(Keyword, "i32"), ..]) =>
            (ty, 1, true),
        (ty, [(LSqBrac, "["), ..]) =>
            (literal, 3, true),
        // +
        (ty, [(Semi, ";"), ..]) =>
            (local_defs, 1, false),
        (ty, [(RSqBrac, "]"), ..]) =>
            (ty, 3, false),
        (ty, [(LBrac, "{"), ..]) =>
            (func_def, 0, false),
        // +
        (ty, [..]) =>
            (param, 1, false),

        (stmts, [(RBrac, "}"), ..]) =>
            (func_body, 0, false),
        (stmts, [_, ..]) =>
            (stmt, 0, true),

        (stmt, [(Keyword, "return"), ..]) =>
            (expr, 1, true),
        (stmt, [(Ident, _), (Eq, "="), ..]) =>
            (assn, 0, true),
        (stmt, [(Ident, _), (LSqBrac, "["), ..]) =>
            (assn, 0, true),
        (stmt, [(Semi, ";"), ..]) =>
            (stmt, 1, false),
        (stmt, [(RBrac, "}"), ..]) =>
            (stmts, 0, false),

        (assn, [(Ident, _), (LSqBrac, "["), ..]) =>
            (assn_target, 0, true),
        (assn, [(Ident, _), (Eq, "="), ..]) =>
            (assn_target, 0, true),
        (assn, [(Eq, "="), ..]) =>
            (expr, 1, false),

        (assn_target, [(Ident, _), (LSqBrac, "["), ..]) =>
            (expr, 2, true),
        (assn_target, [(Ident, _), ..]) =>
            (assn, 1, true),
        (assn_target, [(RSqBrac, "]"), ..]) =>
            (assn, 1, false),

        (expr, [(Literal, _), ..]) =>
            (literal, 0, true),
        (expr, [(Ident, _), ..]) =>
            (expr, 1, true),
        (expr, [(Semi, ";"), ..]) =>
            (stmt, 0, false),

        (literal, [(Literal, _), (RSqBrac, "]")]) =>
            (ty, 1, true),
        (literal, [(Literal, _), ..]) =>
            (expr, 1, true),

        // ...

        _ => (REJECT, 0, true),
    }
}

#[derive(Debug, Clone)]
enum Matcher {
    NT(State),
    Ty(TokenType),
    Ex(TokenType, &'static str),
    EOT,
}

impl Matcher {
    fn is_terminal(&self) -> bool {
        match self {
            Matcher::NT(_) => false,
            Matcher::Ty(_) => true,
            Matcher::Ex(_, _) => true,
            Matcher::EOT => true,
        }
    }
}

#[derive(Debug)]
struct ParseTableEntry {
    matchers: Vec<Matcher>,
    next_state: State,
    consume: bool,
}

#[derive(Debug)]
struct ParseTable {
    transitions: HashMap<State, Vec<ParseTableEntry>>,
}

impl ParseTable {
    fn new() -> ParseTable {
        ParseTable {
            transitions: HashMap::new(),
        }
    }

    pub fn build() -> ParseTable {
        let mut pt = ParseTable::new();
        pt.fill();
        pt
    }

    fn get_entries(&mut self, from: State) -> &mut Vec<ParseTableEntry> {
        if self.transitions.get(&from).is_none() {
            self.transitions.insert(from, Vec::new());
        }
        self.transitions.get_mut(&from).unwrap()
    }

    fn insert(&mut self, from: State, matchers: &[Matcher], next_state: State, consume: bool) {
        self.get_entries(from).push(ParseTableEntry {
            matchers: matchers.to_vec(),
            next_state,
            consume,
        })
    }

    fn take_terminals(matchers: &[Matcher]) -> Vec<Matcher> {
        matchers
            .iter()
            .cloned()
            .take_while(|m| m.is_terminal())
            .collect::<Vec<_>>()
    }

    fn add_rule(&mut self, from: State, mut to: &[Matcher]) {
        let mut state = from;
        while to.len() > 0 {
            let _to_vec = to.to_vec();
            let prefix = ParseTable::take_terminals(to);
            let prefix_len = prefix.len();
            let non_term = to.get(prefix_len).cloned();
            if let Some(Matcher::NT(next_state)) = non_term {
                self.insert(state, &prefix, next_state, true);
                state = next_state;
            }
            to = &to[..prefix_len + 1];
        }
    }

    fn fill(&mut self) {
        use State::*;
        use TokenType::*;
        use Matcher::*;

        self.insert(START, &[EOT], ACCEPT, true);

        self.add_rule(START, &[NT(program), EOT]);

        // self.add_rule(program, &[NT(func_defs), EOT]);
        //
        // self.add_rule(func_defs, &[NT(func_def), NT(func_defs), EOT]);
        // self.add_rule(func_defs, &[EOT]);
        //
        // self.add_rule(func_def, &[
        //     Ex(Keyword, "fn"),
        //     Ty(Ident),
        //     Ty(LParen),
        //     NT(param_list),
        //     Ty(RParen),
        //     NT(ret_ty),
        //     Ty(LBrac),
        //     NT(func_body),
        //     Ty(RBrac),
        //     EOT,
        // ]);
    }

    fn matches(matchers: &[Matcher], tokens: &[QuickToken]) -> bool {
        for (m, (ty, val)) in matchers.iter().zip(tokens) {
            let ok = match m {
                Matcher::Ty(e_ty) =>
                    e_ty == ty,
                Matcher::Ex(e_ty, e_val) =>
                    e_ty == ty && e_val == val,
                Matcher::EOT => false,
                Matcher::NT(_) =>
                    panic!("Cannot match a non terminal to list of tokens: {:?}", m),
            };
            if !ok {
                return false;
            }
        }
        true
    }

    fn next(&self, state: State, tokens: &[QuickToken]) -> (State, usize) {
        let entries = match self.transitions.get(&state) {
            None => return (State::REJECT, 0),
            Some(entries) => entries,
        };
        for ParseTableEntry { matchers, next_state, consume } in entries {
            if ParseTable::matches(&matchers, tokens) {
                let n_consume = if consume {
                    if let Some(Matcher::EOT) = matchers.last() {
                        matchers.len() - 1
                    }  else {
                        matchers.len()
                    }
                } else {
                    0
                };
                return (*next_state, n_consume);
            }
        }
        (State::REJECT, 0)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_table() {
        let table = ParseTable::build();
        println!("{:?}", table)
    }
}