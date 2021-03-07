use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use crate::tokenizer;
use crate::tokenizer::Token;

#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum Symbol {
    START,

    program_items,
    program_item,
    func_def,
    struct_def,
    struct_def_items,
    struct_def_item,
    const_def,
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
    array_item,
    array_literal,
    array_literal_elems,
    struct_literal,
    struct_literal_items,
    struct_item,
    deref,

    undefined,
}


#[derive(Debug, Clone, PartialEq)]
pub struct TokenMatcher {
    token: Token,
    exact_val: bool,
}

impl TokenMatcher {
    pub fn matches(&self, tok: &Token) -> bool {
        if self.exact_val {
            let mut tok = tok.clone();
            tok.clear();
            self.token == tok
        } else {
            &self.token == tok
        }
    }

    pub fn slices_match(prefix: &[TokenMatcher], tokens: &[Token]) -> bool {
        prefix
            .iter()
            .zip(tokens)
            .all(|(m, tok)| m.matches(tok))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Matcher {
    NonTerm(Symbol),
    Term(TokenMatcher),
}

impl Matcher {
    fn is_terminal(&self) -> bool {
        match self {
            Matcher::Term(_) => true,
            Matcher::NonTerm(_) => false,
        }
    }
}

impl From<Symbol> for Matcher {
    fn from(symbol: Symbol) -> Self {
        Matcher::NonTerm(symbol)
    }
}

impl From<char> for Matcher {
    fn from(ch: char) -> Self {
        Matcher::from(ch.to_string().as_str())
    }
}

impl From<&str> for Matcher {
    fn from(text: &str) -> Self {
        let token = tokenizer::tokenize(text).unwrap().get(0).unwrap().to_owned();
        let matcher = TokenMatcher {
            token,
            exact_val: true,
        };
        Matcher::Term(matcher)
    }
}

pub enum TokenMatcherTypeAlias {
    Ident,
    Literal,
    EOF,
}

impl From<TokenMatcherTypeAlias> for Matcher {
    fn from(alias: TokenMatcherTypeAlias) -> Self {
        let token = match alias {
            TokenMatcherTypeAlias::Ident =>
                Token::Ident(String::new()),
            TokenMatcherTypeAlias::Literal =>
                Token::Literal(String::new()),
            TokenMatcherTypeAlias::EOF =>
                Token::EOF,
        };
        Matcher::Term(TokenMatcher {
            token,
            exact_val: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionRule {
    pub lhs: Symbol,
    pub rhs: Vec<Matcher>,
}

pub const _DUMMY_RULE: ProductionRule = ProductionRule {
    lhs: Symbol::undefined,
    rhs: vec![],
};

pub type Grammar = Vec<ProductionRule>;


#[macro_export]
macro_rules! production_rules {
    {$(
        $lhs:ident -> $( $matcher:expr )*;
    )+} => {
        {
            #[allow(unused)]
            use crate::parser::rule::Symbol::*;
            #[allow(unused)]
            use crate::parser::rule::Matcher::*;
            #[allow(unused)]
            use crate::parser::rule::TokenMatcherTypeAlias::*;

            use crate::parser::rule::{Matcher, ProductionRule};

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

const PATTERN_MAX_LEN: usize = 5;

#[derive(Debug, Clone)]
struct Transition {
    pattern: Vec<TokenMatcher>,
    rule: ProductionRule,
}

#[derive(Debug)]
pub struct ParseTable {
    rules: Vec<ProductionRule>,
    transitions: HashMap<Symbol, Vec<Transition>>,
}

impl ParseTable {
    fn new() -> ParseTable {
        ParseTable {
            rules: Vec::new(),
            transitions: HashMap::new(),
        }
    }

    fn transitions_for(&mut self, symbol: Symbol) -> &mut Vec<Transition> {
        if self.transitions.get(&symbol).is_none() {
            self.transitions.insert(symbol, Vec::new());
        }
        self.transitions.get_mut(&symbol).unwrap()
    }

    /*
        START -> program;

        program -> "let" var '=' literal;
        program -> EMPTY;

        var -> Ident;
        literal -> Literal;
    */

    pub fn add_rule(&mut self, rule: &ProductionRule) {
        self.rules.push(rule.to_owned());
        let terminals = rule.rhs
            .iter()
            .take_while(|m| m.is_terminal())
            .map(|m| match m {
                Matcher::Term(tm) => tm,
                Matcher::NonTerm(_) => panic!(),
            })
            .cloned()
            .collect::<Vec<_>>();
        let pattern_len = terminals.len();
        self.transitions_for(rule.lhs).push(Transition {
            pattern: terminals,
            rule: rule.clone(),
        });
        if pattern_len < PATTERN_MAX_LEN {
            let _first_nonterm = rule.rhs.get(pattern_len);

        }
    }

    pub fn finish(&self) {
        for (symbol, transitions) in self.transitions.iter() {
            let mut done = HashSet::<(usize, usize)>::new();
            for (i1, t1) in transitions.iter().enumerate() {
                for (i2, t2) in transitions.iter().enumerate() {
                    if i1 != i2 && t1.pattern == t2.pattern && !done.contains(&(i2, i1)) {
                        println!("WARNING: Ambiguous transitions from {:?}: {:?} -> {:?} and {:?}",
                                 symbol, t1.pattern, t1.rule.rhs, t2.rule.rhs);
                        done.insert((i1, i2));
                    }
                }
            }
        }
    }

    pub fn get(&self, symbol: &Symbol, input: &[Token]) -> Option<ProductionRule> {
        let transitions = self.transitions.get(symbol)?;
        for transition in transitions {
            if TokenMatcher::slices_match(&transition.pattern, input) {
                return Some(transition.rule.to_owned());
            }
        }
        None
    }
}

impl From<&Grammar> for ParseTable {
    fn from(grammar: &Grammar) -> Self {
        let mut table = ParseTable::new();
        for rule in grammar.into_iter() {
            table.add_rule(rule);
        }
        table.finish();
        table
    }
}


mod tests {
    use super::*;
    use crate::tokenizer::ident;

    #[test]
    fn test_simple_grammar() {
        let simple_grammar = production_rules! {
            START -> expr EOF;

            expr -> Ident;
        };
        let table = ParseTable::from(&simple_grammar);
        assert_eq!(
            table.get(&Symbol::START, &[Token::EOF]),
            None,
        );
        assert_eq!(
            table.get(&Symbol::START, &[ident("x"), Token::EOF]).as_ref(),
            simple_grammar.get(0),
        );
        assert_eq!(
            table.get(&Symbol::expr, &[ident("x"), Token::EOF]).as_ref(),
            simple_grammar.get(1),
        );
    }
}