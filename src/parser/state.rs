use std::collections::HashMap;

use crate::tokenizer::{Token, TokenType};

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
    if_stmt,
    while_stmt,
    return_stmt,
    cond,
    bin_op,
    cmp_op,
}

#[derive(Debug, Clone)]
pub(crate) enum Matcher {
    S(State),
    T(TokenType),
    TV(TokenType, String),
    EMPTY,
}

impl From<char> for Matcher {
    fn from(ch: char) -> Self {
        Matcher::TV(
            TokenType::from(ch),
            ch.to_string(),
        )
    }
}

impl From<State> for Matcher {
    fn from(state: State) -> Self {
        Matcher::S(state)
    }
}

impl From<&str> for Matcher {
    fn from(keyword: &str) -> Self {
        Matcher::TV(
            TokenType::Keyword,
            keyword.to_string(),
        )
    }
}

impl From<TokenType> for Matcher {
    fn from(ty: TokenType) -> Self {
        Matcher::T(ty)
    }
}


impl Matcher {
    fn is_terminal(&self) -> bool {
        match self {
            Matcher::S(_) => false,
            Matcher::T(_) => true,
            Matcher::TV(_, _) => true,
            Matcher::EMPTY => true,
        }
    }
}


#[derive(Debug, Clone)]
pub(crate) struct ProductionRule {
    pub(crate) lhs: State,
    pub(crate) rhs: Vec<Matcher>,
}

pub(crate) type Grammar = Vec<ProductionRule>;

#[macro_export]
macro_rules! production_rules {
    {$(
        $lhs:ident -> $( $matcher:expr )+;
    )+} => {
        {
            #[allow(unused)]
            use crate::parser::state::State::*;
            #[allow(unused)]
            use crate::parser::state::Matcher::*;
            #[allow(unused)]
            use crate::tokenizer::TokenType::*;

            use crate::parser::state::Matcher;
            use crate::parser::state::ProductionRule;

            vec![
                $(
                    ProductionRule {
                        lhs: $lhs,
                        rhs: vec![ $(Matcher::from($matcher),)+ ]
                    },
                )+
            ]
        }
    };
}


#[derive(Debug)]
struct TransitionEntry {
    matchers: Vec<Matcher>,
    next_state: State,
    rule: Option<ProductionRule>,
}

#[derive(Debug)]
pub(crate) struct ParseTable {
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
                matchers: vec![Matcher::EMPTY],
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

    pub(crate) fn add_rule(&mut self, rule: ProductionRule) {
        self.rules.push(rule.clone());
    }

    pub(crate) fn transition(&self, state: State, tokens: &[Token]) -> (State, Option<ProductionRule>) {
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

    fn matches(matchers: &[Matcher], tokens: &[Token]) -> bool {
        if let Some(Matcher::EMPTY) = matchers.first() {
            return true;
        }
        for (m, Token { ty, val }) in matchers.iter().zip(tokens) {
            let ok = match m {
                Matcher::T(e_ty) =>
                    e_ty == ty,
                Matcher::TV(e_ty, e_val) =>
                    e_ty == ty && e_val == val,
                _ => panic!("Cannot match a non terminal to list of tokens: {:?}", m),
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

impl From<Grammar> for ParseTable {
    fn from(grammar: Grammar) -> Self {
        let mut pt = ParseTable::new();
        for rule in grammar.into_iter() {
            pt.add_rule(rule);
        }
        pt
    }
}

mod tests {
    #![allow(unused_imports)]
    use Matcher::*;
    use State::*;
    use TokenType::*;

    use super::*;

    fn test_grammar() -> Grammar {
        production_rules! {
            START   -> program;
            program -> Ident;
        }
    }
}