const KEYWORDS: &[&str] = &[
    "fn",
    "i32",
    "let",
    "return",
    "struct",
    "if",
    "while",

    // FUTURE
    "const",
    "f32",
    "u8",
];

const MULTI_CHAR_SYMS: &[&str] = &[
    "->",
    "==",
];

const HEX_CHARS: &[char] = &[
    'a',
    'b',
    'c',
    'd',
    'e',
    'f',
];

#[derive(Debug)]
pub enum TokenizerError {
    UnrecognizedChar(usize, char),
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Token {
    Unknown(String),

    Space(String),
    Ident(String),
    Keyword(String),
    Literal(String),
    Sym(String),

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

pub fn sym(s: &str) -> Token {
    Token::Sym(s.to_string())
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

    fn push(&mut self, ch: char) {
        if let Some(val) = self.val_mut() {
            val.push(ch);
        }
    }
}

impl From<char> for Token {
    fn from(ch: char) -> Self {
        (match ch {
            '\t' | '\n' | '\x0C' | '\r' | ' ' =>
                Token::Space,
            '0'..='9' =>
                Token::Literal,
            'a'..='z' | 'A'..='Z' =>
                Token::Ident,
            '(' | ')' | '{' | '}' | '[' | ']' |
            '=' | '<' | '>' |
            '+' | '-' | '*' | '/' |
            ':' | ';' | ',' | '!' | '.' =>
                Token::Sym,
            _ =>
                Token::Unknown,
        })(ch.to_string())
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
        match (&tok, &Token::from(ch)) {
            (_, Token::Unknown(_)) =>
                return Err(TokenizerError::UnrecognizedChar(i, ch)),

            (Token::Space(_), Token::Space(_)) |
            (Token::Ident(_), Token::Ident(_)) |
            (Token::Ident(_), Token::Literal(_)) |
            (Token::Literal(_), Token::Literal(_))
            => tok.push(ch), // append

            (Token::Literal(s1), Token::Ident(s2))
            if (s1 == "0" && s2 == "x") || HEX_CHARS.contains(&ch)
            => tok.push(ch),

            (Token::Sym(s1), Token::Sym(s2)) => {
                let multi_char_sym = s1.clone() + s2;
                if MULTI_CHAR_SYMS.contains(&multi_char_sym.as_str()) {
                    tok = Token::Sym(multi_char_sym);
                } else {
                    tokens.push(tok);
                    tok = Token::Sym(s2.clone());
                }
            }
            (Token::Ident(name), new) => {
                if KEYWORDS.contains(&name.as_str()) {
                    tok = Token::Keyword(name.clone())
                }
                tokens.push(tok);
                tok = new.clone();
            }
            (_, new) => {
                tokens.push(tok);
                tok = new.clone();
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

    #[test]
    fn test_minimal_program() -> Result<(), TokenizerError> {
        let text = "fn main() { }";
        let tokens = tokenize(text)?;
        assert_eq!(dump_tokens(&tokens), dump_tokens(&[
            keyword("fn"),
            ident("main"),
            sym("("),
            sym(")"),
            sym("{"),
            sym("}"),
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
            keyword("fn"), ident("main"), sym("("), ident("x"), sym(":"), keyword("i32"), sym(")"),
            sym("{"),
                keyword("let"), ident("x1"), sym(":"), keyword("i32"), sym(";"),
                ident("x1"), sym("="), ident("add34"), sym("("), ident("x"), sym(")"), sym(";"),
                ident("print"), sym("("), ident("x1"), sym(")"), sym(";"),
            sym("}"),
            keyword("fn"), ident("add34"), sym("("), ident("y"), sym(":"), keyword("i32"), sym(")"),
            sym("->"), keyword("i32"),
            sym("{"),
                keyword("return"), ident("y"), sym("+"), literal("34"), sym(";"),
            sym("}"),
        ]));
        Ok(())
    }

    #[test]
    fn test_hex_literal() -> Result<(), TokenizerError> {
        let text = "0x1234abcdef hello 0x123g";
        let tokens = tokenize(text)?;
        assert_eq!(tokens, &[
            literal("0x1234abcdef"),
            ident("hello"),
            literal("0x123"),
            ident("g"),
        ]);
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
