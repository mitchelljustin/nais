use std::collections::HashMap;

use crate::tokenizer::{QuickToken, TokenType};

#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
pub(crate) enum State {
    START,
    ACCEPT,
    REJECT,

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
}

#[derive(Debug, Clone)]
enum Matcher {
    S(State),
    T(TokenType),
    TT(TokenType, String),
    E,
}

#[allow(non_snake_case)]
fn C(ch: char) -> Matcher {
    Matcher::TT(
        TokenType::from(ch),
        ch.to_string(),
    )
}

#[allow(non_snake_case)]
fn K(name: &str) -> Matcher {
    Matcher::TT(
        TokenType::Keyword,
        name.to_string()
    )
}

impl Matcher {
    fn is_terminal(&self) -> bool {
        match self {
            Matcher::S(_) => false,
            Matcher::T(_) => true,
            Matcher::TT(_, _) => true,
            Matcher::E => true,
        }
    }
}


#[derive(Debug, Clone)]
pub struct ProductionRule {
    lhs: State,
    rhs: &'static [Matcher],
}

macro_rules! production_rules {
    {$(
        $lhs:ident -> $( $mat:expr )+;
    )+} => {
        {
            #[allow(unused)]
            use crate::parser::state::State::*;
            #[allow(unused)]
            use crate::parser::state::Matcher::*;
            #[allow(unused)]
            use crate::tokenizer::TokenType::*;
            vec![
                $(
                    ProductionRule {lhs: $lhs, rhs: &[ $($mat,)+ ]},
                )+
            ]
        }
    };
}

pub fn minirust_grammar() -> Vec<ProductionRule> {
    production_rules! {
        START -> S(program);

        program -> S(func_defs);

        func_defs -> S(func_def) S(func_defs);
        func_defs -> E;

        func_def -> K("fn") T(Ident) C('(') S(param_list) C(')') S(ret_ty) C('{') S(func_body) C('}');

        param_list -> S(params);
        param_list -> E;

        params -> S(param) C(',') S(params);
        params -> S(param);

        param -> T(Ident) C(':') S(ty);

        ty -> K("i32");
        ty -> C('[') K("i32") C(';') T(Literal) C(']');

        ret_ty -> E;
        ret_ty -> T(RArrow) S(ty);

        func_body -> S(local_defs) S(stmts);

        local_defs -> S(local_def) S(local_defs);
        local_defs -> E;

        local_def -> K("let") T(Ident) C(':') S(ty) C(';');

        stmts -> S(stmt) S(stmts);
        stmts -> E;

        stmt -> S(assn) C(';');
        stmt -> S(expr) C(';');
        stmt -> K("return") S(expr) C(';');

        assn -> S(assn_target) C('=') S(expr);
        assn_target -> T(Ident);
        assn_target -> T(Ident) C('[') S(expr) C(']');

        expr -> C('(') S(expr) C(')');
        expr -> T(Literal);
        expr -> T(Ident);
        expr -> S(bin_expr);
        expr -> S(func_call);

        bin_expr -> S(product) C('*') S(expr);
        bin_expr -> S(product) C('/') S(expr);
        product -> S(expr);

        bin_expr -> S(term) C('+') S(expr);
        bin_expr -> S(term) C('-') S(expr);
        term -> S(expr);

        func_call -> T(Ident) C('(') S(arg_list) C(')');

        arg_list -> E;
        arg_list -> S(args);

        args -> S(arg) C(',') S(args);
        args -> S(arg);

        arg -> S(expr);
    }
}

#[derive(Debug)]
struct TransitionEntry {
    matchers: Vec<Matcher>,
    next_state: State,
    rule: Option<ProductionRule>,
}

#[derive(Debug)]
struct ParseTable {
    rules: Vec<ProductionRule>,
    transitions: HashMap<State, Vec<TransitionEntry>>,
}

impl ParseTable {
    fn new() -> ParseTable {
        let mut pt = ParseTable {
            rules: Vec::new(),
            transitions: HashMap::new(),
        };
        pt.transitions_from(State::START)
            .push(TransitionEntry {
                matchers: vec![Matcher::E],
                next_state: State::ACCEPT,
                rule: None,
            });
        pt
    }

    fn transitions_from(&mut self, from: State) -> &mut Vec<TransitionEntry> {
        if self.transitions.get(&from).is_none() {
            self.transitions.insert(from, Vec::new());
        }
        self.transitions.get_mut(&from).unwrap()
    }

    pub fn add_rule(&mut self, rule: ProductionRule) {
        self.rules.push(rule.clone());
        let ProductionRule { lhs, rhs } = rule;
        let mut matchers = rhs.to_vec();
        let transitions = self.transitions_from(lhs);
        while matchers.len() > 0 {
            let terminals_prefix = ParseTable::take_terminals(&matchers);
        }
    }

    fn next(&self, state: State, tokens: &[QuickToken]) -> (State, Option<ProductionRule>) {
        let entries = match self.transitions.get(&state) {
            None => return (State::REJECT, None),
            Some(entries) => entries,
        };
        for TransitionEntry {
            matchers, next_state, rule,
        } in entries {
            if ParseTable::matches(&matchers, tokens) {
                return (*next_state, rule.clone());
            }
        }
        (State::REJECT, None)
    }

    fn matches(matchers: &[Matcher], tokens: &[QuickToken]) -> bool {
        for (m, (ty, val)) in matchers.iter().zip(tokens) {
            let ok = match m {
                Matcher::T(e_ty) =>
                    e_ty == ty,
                Matcher::TT(e_ty, e_val) =>
                    e_ty == ty && e_val == val,
                Matcher::E => false,
                Matcher::S(_) =>
                    panic!("Cannot match a non terminal to list of tokens: {:?}", m),
            };
            if !ok {
                return false;
            }
        }
        true
    }


    fn take_terminals(matchers: &[Matcher]) -> Vec<Matcher> {
        matchers
            .iter()
            .cloned()
            .take_while(|m| m.is_terminal())
            .collect::<Vec<_>>()
    }
}

impl From<&[ProductionRule]> for ParseTable {
    fn from(rules: &[ProductionRule]) -> Self {
        let mut pt = ParseTable::new();
        for rule in rules {
            pt.add_rule(rule.clone());
        }
        pt
    }
}

mod tests {
    use Matcher::*;
    use State::*;
    use TokenType::*;

    use super::*;

    #[test]
    fn test_parse_table() {
        let test_grammar = production_rules! {
            START   -> S(program);
            program -> T(Ident);
        };

        let table = ParseTable::from(&test_grammar);
        println!("{:?}", table);

        let tokens = &[
            (Ident, "ok"),
        ];

        assert_eq!(table.next(program, tokens).0, START);
        assert_eq!(table.next(START, tokens).0, program);
        assert_eq!(table.next(START, &[]).0, ACCEPT);
    }
}