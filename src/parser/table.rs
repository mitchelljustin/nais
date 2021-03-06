use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;

use crate::tokenizer::{Token, TokenType};

#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
pub enum Symbol {
    START,

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
    array_read,
    array_target,
    var_target,
    var,
    literal,
}


#[derive(Debug, Clone)]
enum TokenMatcher {
    Type(TokenType),
    TypeAndVal(TokenType, String),
}

impl TokenMatcher {
    pub fn matches(&self, Token { ty, val }: &Token) -> bool {
        match self {
            TokenMatcher::Type(e_ty) =>
                e_ty == ty,
            TokenMatcher::TypeAndVal(e_ty, e_val) =>
                e_ty == ty && e_val == val,
        }
    }

    pub fn slices_match(prefix: &[TokenMatcher], tokens: &[Token]) -> bool {
        prefix
            .iter()
            .zip(tokens)
            .all(|(m, tok)| m.matches(tok))
    }
}

#[derive(Debug, Clone)]
pub enum Matcher {
    NonTerm(Symbol),
    Term(TokenMatcher),
}

impl From<char> for Matcher {
    fn from(ch: char) -> Self {
        Matcher::Term(TokenMatcher::TypeAndVal(
            TokenType::from(ch),
            ch.to_string(),
        ))
    }
}

impl From<Symbol> for Matcher {
    fn from(symbol: Symbol) -> Self {
        Matcher::NonTerm(symbol)
    }
}

impl From<&str> for Matcher {
    fn from(keyword: &str) -> Self {
        Matcher::Term(TokenMatcher::TypeAndVal(
            TokenType::Keyword,
            keyword.to_string(),
        ))
    }
}

impl From<TokenType> for Matcher {
    fn from(ty: TokenType) -> Self {
        Matcher::Term(TokenMatcher::Type(ty))
    }
}

#[derive(Debug, Clone)]
pub struct ProductionRule {
    pub lhs: Symbol,
    pub rhs: Vec<Matcher>,
}

pub type Grammar = Vec<ProductionRule>;

#[macro_export]
macro_rules! production_rules {
    {$(
        $lhs:ident -> $( $matcher:expr )*;
    )+} => {
        {
            #[allow(unused)]
            use crate::parser::table::Symbol::*;
            #[allow(unused)]
            use crate::parser::table::Matcher::*;
            #[allow(unused)]
            use crate::tokenizer::TokenType::*;

            use crate::parser::table::Matcher;
            use crate::parser::table::ProductionRule;

            vec![
                $(
                    ProductionRule {
                        lhs: $lhs,
                        rhs: vec![ $(Matcher::from($matcher),)* ]
                    },
                )+
            ]
        }
    };
}

impl From<Matcher> for Option<TokenMatcher> {
    fn from(m: Matcher) -> Self {
        match m {
            Matcher::Term(tm) => Some(tm),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Transition {
    pattern: Vec<TokenMatcher>,
    rule: ProductionRule,
}

#[derive(Debug)]
pub struct ParseTable {
    rules: Vec<ProductionRule>,
    transition_tab: HashMap<Symbol, Vec<Transition>>,
}

impl ParseTable {
    fn new() -> ParseTable {
        let mut pt = ParseTable {
            rules: Vec::new(),
            transition_tab: HashMap::new(),
        };
        pt
    }

    fn transitions_from(&mut self, symbol: Symbol) -> &mut Vec<Transition> {
        if self.transition_tab.get(&symbol).is_none() {
            self.transition_tab.insert(symbol, Vec::new());
        }
        self.transition_tab.get_mut(&symbol).unwrap()
    }

    fn add_transition(&mut self, symbol: Symbol, pattern: Vec<TokenMatcher>, rule: ProductionRule) {
        self.transitions_from(symbol).push(Transition {
            pattern,
            rule,
        });
    }

    /*
        START -> program;

        program -> "let" var '=' literal;
        program -> EMPTY;

        var -> Ident;
        literal -> Literal;
    */

    pub fn add_rule(&mut self, rule: ProductionRule) {
        self.rules.push(rule.clone());
        let ProductionRule { lhs, mut rhs } = rule;
        let prefix = rhs
            .iter()
            .take_while(|m| m.is_terminal())
            .cloned()
            .filter_map(|m| Option::<TokenMatcher>::from(m))
            .collect::<Vec<_>>();
    }

    pub fn lookup(&self, symbol: Symbol, tokens: &[Token]) -> Option<ProductionRule> {
        let transitions = self.transition_tab.get(&symbol)?;
        for t in transitions {
            if TokenMatcher::slices_match(&t.pattern, tokens) {
                return Some(t.rule.clone());
            }
        }
        None
    }
}

impl From<Grammar> for ParseTable {
    fn from(grammar: Grammar) -> Self {
        let mut table = ParseTable::new();
        for rule in grammar.into_iter() {
            table.add_rule(rule);
        }
        table
    }
}

mod tests {
    #![allow(unused_imports)]

    use Matcher::*;
    use Matcher::*;
    use TokenType::*;

    use super::*;

    fn trivial_grammar() -> Grammar {
        production_rules! {
            START   -> program;
            program -> '+' Literal;
            program -> '+';
        }
    }

    fn medium_grammar() -> Grammar {
        production_rules! {
            START -> program;

            program -> "let" var '=' literal;
            program -> ;

            var -> Ident;
            literal -> Literal;
        }
    }
}