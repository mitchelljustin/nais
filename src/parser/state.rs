use std::collections::HashMap;

use crate::tokenizer::{QuickToken, TokenType};
use crate::parser::minirust;

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
pub enum Matcher {
    S(State),
    T(TokenType),
    TV(TokenType, String),
    E,
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
            Matcher::E => true,
        }
    }
}


#[derive(Debug, Clone)]
pub struct ProductionRule {
    lhs: State,
    rhs: Vec<Matcher>,
}

pub type Grammar = Vec<ProductionRule>;

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
pub struct ParseTable {
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

    pub(crate) fn minirust() -> ParseTable {
        ParseTable::from(minirust::grammar())
    }

    fn transitions_from(&mut self, from: State) -> &mut Vec<TransitionEntry> {
        if self.transitions.get(&from).is_none() {
            self.transitions.insert(from, Vec::new());
        }
        self.transitions.get_mut(&from).unwrap()
    }

    pub fn add_rule(&mut self, rule: ProductionRule) {
        self.rules.push(rule.clone());
    }

    pub(crate) fn transition(&self, state: State, tokens: &[QuickToken]) -> (State, Option<ProductionRule>) {
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
                Matcher::TV(e_ty, e_val) =>
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

    #[test]
    fn test_parse_table() {
        println!("{:?}", test_grammar());

        let table = ParseTable::from(test_grammar());
        println!("{:?}", table);

        let tokens = &[
            (Ident, "ok"),
        ];

        assert_eq!(table.transition(program, tokens).0, START);
        assert_eq!(table.transition(START, tokens).0, program);
        assert_eq!(table.transition(START, &[]).0, ACCEPT);
    }
}