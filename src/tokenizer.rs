use std::fmt;
use std::fmt::Formatter;

const KEYWORDS: &[&str] = &[
    "fn",
    "if",
    "while",
    "let",
    "return",
    "i32",
];

#[derive(Debug)]
pub enum TokenizeError {
    UnrecognizedChar(usize, char),
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum CharType {
    Unknown,

    Space,
    Letter,
    Digit,

    RParen,
    LParen,
    LBrac,
    RBrac,
    LSqBrac,
    RSqBrac,

    GreaterThan,

    Colon,
    Semi,
    Comma,

    Eq,

    Plus,
    Minus,
}

impl From<char> for CharType {
    fn from(ch: char) -> Self {
        match ch {
            '(' => CharType::LParen,
            ')' => CharType::RParen,
            '{' => CharType::LBrac,
            '}' => CharType::RBrac,
            '[' => CharType::LSqBrac,
            ']' => CharType::RSqBrac,
            '>' => CharType::GreaterThan,
            ':' => CharType::Colon,
            ';' => CharType::Semi,
            ',' => CharType::Comma,
            '=' => CharType::Eq,
            '+' => CharType::Plus,
            '-' => CharType::Minus,
            '\t' | '\n' | '\x0C' | '\r' | ' ' => CharType::Space,
            '0'..='9' => CharType::Digit,
            'a'..='z' | 'A'..='Z' => CharType::Letter,
            _ => return CharType::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    Unknown,

    Space,
    Ident,
    Keyword,
    Literal,

    RParen,
    LParen,
    LBrac,
    RBrac,
    LSqBrac,
    RSqBrac,

    RArrow,

    Colon,
    Semi,
    Comma,

    Eq,

    Plus,
    Minus,

    EqEq,
}


impl From<CharType> for TokenType {
    fn from(ch_ty: CharType) -> Self {
        match ch_ty {
            CharType::Space => TokenType::Space,
            CharType::Letter => TokenType::Ident,
            CharType::Digit => TokenType::Literal,
            CharType::LParen => TokenType::LParen,
            CharType::RParen => TokenType::RParen,
            CharType::LBrac => TokenType::LBrac,
            CharType::RBrac => TokenType::RBrac,
            CharType::LSqBrac => TokenType::LSqBrac,
            CharType::RSqBrac => TokenType::RSqBrac,
            CharType::Colon => TokenType::Colon,
            CharType::Semi => TokenType::Semi,
            CharType::Comma => TokenType::Comma,
            CharType::Eq => TokenType::Eq,
            CharType::Plus => TokenType::Plus,
            CharType::Minus => TokenType::Minus,
            _ => TokenType::Unknown,
        }
    }
}


#[derive(PartialEq, Clone)]
pub struct Token {
    pub(crate) ty: TokenType,
    pub(crate) val: String,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} \"{}\"", self.ty, self.val)
    }
}

pub fn dump_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| format!("{:?}", t))
        .collect::<Vec<_>>()
        .join("\n")
}


enum Decision {
    Append,
    UpdateType(TokenType),
    Cut,
}

fn decide_token(tok_ty: TokenType, ch_ty: CharType) -> Decision {
    use Decision::*;
    match (tok_ty, ch_ty) {
        // Same type
        (TokenType::Space, CharType::Space) => Append,

        (TokenType::Ident, CharType::Letter) => Append,
        (TokenType::Ident, CharType::Digit) => Append,

        (TokenType::Literal, CharType::Digit) => Append,

        (TokenType::Eq, CharType::Eq) => UpdateType(TokenType::EqEq),
        (TokenType::Minus, CharType::GreaterThan) => UpdateType(TokenType::RArrow),

        (_, _) => Cut,
    }
}

pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut tokens = Vec::new();

    use Decision::*;
    let mut tok = Token { ty: TokenType::Space, val: String::new() };
    for (i, ch) in text.chars().enumerate() {
        let ch_ty = match CharType::from(ch) {
            CharType::Unknown => return Err(TokenizeError::UnrecognizedChar(i, ch)),
            ch_ty => ch_ty,
        };
        let decision = decide_token(tok.ty, ch_ty);
        match decision {
            Append => {
                tok.val.push(ch);
            }
            UpdateType(new_ty) => {
                tok.val.push(ch);
                tok.ty = new_ty;
            }
            Cut => {
                if tok.ty == TokenType::Ident && KEYWORDS.contains(&tok.val.as_str()) {
                    tok.ty = TokenType::Keyword;
                }
                tokens.push(tok);
                let ty = TokenType::from(ch_ty);
                tok = Token { ty, val: ch.to_string() };
            }
        }
    }
    tokens.push(tok);
    Ok(
        tokens
            .into_iter()
            .filter(|t| t.ty != TokenType::Space)
            .collect()
    )
}

mod tests {
    use super::*;
    use super::TokenType::*;

    fn make_tokens(toks: &[(TokenType, &str)]) -> Vec<Token> {
        toks.into_iter()
            .map(|(ty, val)| Token { ty: *ty, val: val.to_string() })
            .collect()
    }

    fn gen_dump(toks: &[(TokenType, &str)]) -> String {
        dump_tokens(&make_tokens(toks))
    }

    #[test]
    fn test_minimal_program() {
        let text = "fn main() { }";
        let tokens = match tokenize(text) {
            Err(e) => panic!("Tokenizer error: {:?}", e),
            Ok(tokens) => tokens
        };
        assert_eq!(dump_tokens(&tokens), gen_dump(&[
            (Keyword, "fn"),
            (Ident, "main"),
            (LParen, "("),
            (RParen, ")"),
            (LBrac, "{"),
            (RBrac, "}"),
        ]));
    }

    #[test]
    fn test_multi_fn() -> Result<(), TokenizeError> {
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
        let tokens = match tokenize(text) {
            Err(e) => panic!("Tokenizer error: {:?}", e),
            Ok(tokens) => tokens
        };
        assert_eq!(dump_tokens(&tokens), gen_dump(&[
            (Keyword, "fn"), (Ident, "main"), (LParen, "("), (Ident, "x"), (Colon, ":"), (Keyword, "i32"), (RParen, ")"),
            (LBrac, "{"),
            (Keyword, "let"), (Ident, "x1"), (Colon, ":"), (Keyword, "i32"), (Semi, ";"),
            (Ident, "x1"), (Eq, "="), (Ident, "add34"), (LParen, "("), (Ident, "x"), (RParen, ")"), (Semi, ";"),
            (Ident, "print"), (LParen, "("), (Ident, "x1"), (RParen, ")"), (Semi, ";"),
            (RBrac, "}"),
            (Keyword, "fn"), (Ident, "add34"), (LParen, "("), (Ident, "y"), (Colon, ":"), (Keyword, "i32"), (RParen, ")"),
                (RArrow, "->"), (Keyword, "i32"),
            (LBrac, "{"),
            (Keyword, "return"), (Ident, "y"), (Plus, "+"), (Literal, "34"), (Semi, ";"),
            (RBrac, "}"),
        ]));
        Ok(())
    }

    #[test]
    fn test_bad_char() {
        let text = "fn main()` { }";
        match tokenize(text) {
            Err(TokenizeError::UnrecognizedChar(9, '`')) => {}
            Err(other) => panic!("Got some other error: {:?}", other),
            Ok(tokens) => panic!("Got tokens: {:?}", tokens),
        };
    }
}
