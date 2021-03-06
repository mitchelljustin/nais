
const KEYWORDS: &[&str] = &[
    "fn",
    "let",
    "return",
    "i32",
];

#[derive(Debug)]
pub enum TokenizerError {
    UnrecognizedChar(usize, char),
}


#[derive(Debug, PartialEq,  Clone, Eq, Hash)]
pub enum Token {
    Unknown(String),

    Space(String),
    Ident(String),
    Keyword(String),
    Literal(String),

    Sym(char),

    EOF,
}

pub fn keyword(s: &str) -> Token {
    Token::Keyword(s.to_string())
}

pub fn ident(s: &str) -> Token {
    Token::Ident(s.to_string())
}

pub fn literal(s: &str) -> Token {
    Token::Literal(s.to_string())
}

impl Token {
    fn val_mut(&mut self) -> Option<&mut String> {
        match self {
            Token::Space(x) |
            Token::Ident(x) |
            Token::Keyword(x) |
            Token::Literal(x) => Some(x),
            _ => None,
        }
    }

    pub fn push(&mut self, ch: char) {
        if let Some(val) = self.val_mut() {
            val.push(ch);
        }
    }

    pub fn clear(&mut self) {
        if let Some(val) = self.val_mut() {
            val.clear();
        }
    }
}

impl From<char> for Token {
    fn from(ch: char) -> Self {
        match ch {
            '\t' | '\n' | '\x0C' | '\r' | ' ' =>
                Token::Space(ch.to_string()),
            '0'..='9' =>
                Token::Literal(ch.to_string()),
            'a'..='z' | 'A'..='Z' =>
                Token::Ident(ch.to_string()),
            '(' | ')' | '{' | '}' | '[' | ']' | '>' | '<' | ':' | ';' | ',' | '=' | '+' | '-' | '!' | '*' | '/' =>
                Token::Sym(ch),
            _ =>
                Token::Unknown(ch.to_string()),
        }
    }
}


pub fn dump_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| format!("{:?}", t))
        .collect::<Vec<_>>()
        .join("\n")
}


pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenizerError> {
    let mut tokens = Vec::new();

    let mut tok = Token::Space("".to_string());
    for (i, ch) in text.chars().enumerate() {
        match (&tok, Token::from(ch)) {
            (_, Token::Unknown(_)) =>
                return Err(TokenizerError::UnrecognizedChar(i, ch)),
            (Token::Space(_), Token::Space(_)) |
            (Token::Ident(_), Token::Ident(_)) |
            (Token::Ident(_), Token::Literal(_)) |
            (Token::Literal(_), Token::Literal(_)) =>
                tok.push(ch), // append
            (_, new) => {
                if let Token::Ident(name) = &tok {
                    if KEYWORDS.contains(&name.as_str()) {
                        tok = Token::Keyword(name.clone())
                    }
                }
                tokens.push(tok); // cut
                tok = new;
            }
        }
    }
    tokens.push(tok);
    Ok(
        tokens
            .into_iter()
            .filter(|t| match t {
                Token::Space(_) => false,
                _ => true,
            })
            .collect()
    )
}

mod tests {
    use super::*;
    use super::Token::*;

    #[test]
    fn test_minimal_program() -> Result<(), TokenizerError> {
        let text = "fn main() { }";
        let tokens = tokenize(text)?;
        assert_eq!(dump_tokens(&tokens), dump_tokens(&[
            keyword("fn"),
            ident("main"),
            Sym('('),
            Sym(')'),
            Sym('{'),
            Sym('}'),
        ]));
        Ok(())
    }

    #[test]
    fn test_multi_fn() -> Result<(), TokenizerError> {
        let text = "
        fn main(x: i32)
        {
            let x1: i32;
            x1 = add34(x);
            print(x1);
        }

        fn add34(y: i32) -> i32
        {
            return y + 34;
        }
        ";
        let tokens = tokenize(text)?;
        assert_eq!(dump_tokens(&tokens), dump_tokens(&[
            keyword("fn"), ident("main"), Sym('('), ident("x"), Sym(':'), keyword("i32"), Sym(')'),
            Sym('{'),
            keyword("let"), ident("x1"), Sym(':'), keyword("i32"), Sym(';'),
            ident("x1"), Sym('='), ident("add34"), Sym('('), ident("x"), Sym(')'), Sym(';'),
            ident("print"), Sym('('), ident("x1"), Sym(')'), Sym(';'),
            Sym('}'),
            keyword("fn"), ident("add34"), Sym('('), ident("y"), Sym(':'), keyword("i32"), Sym(')'),
            Sym('-'), Sym('>'), keyword("i32"),
            Sym('{'),
            keyword("return"), ident("y"), Sym('+'), literal("34"), Sym(';'),
            Sym('}'),
        ]));
        Ok(())
    }

    #[test]
    fn test_bad_char() {
        let text = "fn main()` { }";
        match tokenize(text) {
            Err(TokenizerError::UnrecognizedChar(9, '`')) => {}
            Err(other) => panic!("Got some other error: {:?}", other),
            Ok(tokens) => panic!("Got tokens: {:?}", tokens),
        };
    }
}
