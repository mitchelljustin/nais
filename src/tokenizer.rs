
use std::fmt;
use std::fmt::Formatter;

const KEYWORDS: &[&str] = &[
    "fn",
    "if",
    "while",
    "let",
    "return"
];

#[derive(Debug)]
enum TokenizeError {
    UnrecognizedChar(usize, char),
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum TokChar {
    Space,
    Letter,
    Digit,

    RParen,
    LParen,
    LBrac,
    RBrac,

    Semi,

    Eq,

    Plus,
    Minus,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum TokenType {
    Space,
    Ident,
    Keyword,
    Literal,

    RParen,
    LParen,
    LBrac,
    RBrac,

    Semi,

    Eq,

    Plus,
    Minus,

    EqEq,
}

#[derive(PartialEq, Clone)]
struct Token { ty: TokenType, val: String }

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} \"{}\"", self.ty, self.val)
    }
}

fn dump_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| format!("{:?}", t))
        .collect::<Vec<_>>()
        .join("\n")
}

fn tokenize(text: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut tokens = Vec::new();
    enum Decision {
        Append,
        UpdateType(TokenType),
        Cut,
    }
    use Decision::*;
    let mut tok = Token { ty: TokenType::Space, val: String::new() };
    for (i, ch) in text.chars().enumerate() {
        let ch_ty = match ch {
            '(' => TokChar::LParen,
            ')' => TokChar::RParen,
            '{' => TokChar::LBrac,
            '}' => TokChar::RBrac,
            ';' => TokChar::Semi,
            '=' => TokChar::Eq,
            '+' => TokChar::Plus,
            '-' => TokChar::Minus,
            '\t' | '\n' | '\x0C' | '\r' | ' ' => TokChar::Space,
            '0'..='9' => TokChar::Digit,
            'a'..='z' | 'A'..='Z' => TokChar::Letter,
            _ => return Err(TokenizeError::UnrecognizedChar(i, ch)),
        };
        let decision = match (tok.ty, ch_ty) {
            // Same type
            (TokenType::Space, TokChar::Space)    => Append,

            (TokenType::Ident, TokChar::Letter)   => Append,
            (TokenType::Ident, TokChar::Digit)    => Append,

            (TokenType::Literal, TokChar::Digit)  => Append,

            (TokenType::Eq, TokChar::Eq)          => UpdateType(TokenType::EqEq),

            (_, _)                     => Cut,
        };
        match decision {
            Append => {
                tok.val.push(ch);
            },
            UpdateType(new_ty) => {
                tok.ty = new_ty;
            },
            Cut => {
                if tok.ty == TokenType::Ident && KEYWORDS.contains(&tok.val.as_str()) {
                    tok.ty = TokenType::Keyword;
                }
                tokens.push(tok);
                let ty = match ch_ty {
                    TokChar::Space => TokenType::Space,
                    TokChar::Letter => TokenType::Ident,
                    TokChar::Digit => TokenType::Literal,

                    TokChar::LParen => TokenType::LParen,
                    TokChar::RParen => TokenType::RParen,
                    TokChar::LBrac => TokenType::LBrac,
                    TokChar::RBrac => TokenType::RBrac,
                    TokChar::Semi => TokenType::Semi,
                    TokChar::Eq => TokenType::Eq,
                    TokChar::Plus => TokenType::Plus,
                    TokChar::Minus => TokenType::Minus,
                };
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
    fn test_simple_main() {
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
    fn test_complex_main() -> Result<(), TokenizeError> {
        let text = "
        fn main(x)
        {
            let x1 = x + 34;
            print(x1);
        }";
        let tokens = match tokenize(text) {
            Err(e) => panic!("Tokenizer error: {:?}", e),
            Ok(tokens) => tokens
        };
        assert_eq!(dump_tokens(&tokens), gen_dump(&[
            (Keyword, "fn"), (Ident, "main"), (LParen, "("), (Ident, "x"), (RParen, ")"),
            (LBrac, "{"),
            (Keyword, "let"), (Ident, "x1"), (Eq, "="), (Ident, "x"), (Plus, "+"), (Literal, "34"), (Semi, ";"),
            (Ident, "print"), (LParen, "("), (Ident, "x1"), (RParen, ")"), (Semi, ";"),
            (RBrac, "}"),
        ]));
        Ok(())
    }

    #[test]
    fn test_multi_fn() -> Result<(), TokenizeError> {
        let text = "
        fn main(x)
        {
            let x1 = add34(x);
            print(x1);
        }

        fn add34(y)
        {
            return y + 34;
        }
        ";
        let tokens = match tokenize(text) {
            Err(e) => panic!("Tokenizer error: {:?}", e),
            Ok(tokens) => tokens
        };
        assert_eq!(dump_tokens(&tokens), gen_dump(&[
            (Keyword, "fn"), (Ident, "main"), (LParen, "("), (Ident, "x"), (RParen, ")"),
            (LBrac, "{"),
            (Keyword, "let"), (Ident, "x1"), (Eq, "="), (Ident, "add34"), (LParen, "("), (Ident, "x"), (RParen, ")"), (Semi, ";"),
            (Ident, "print"), (LParen, "("), (Ident, "x1"), (RParen, ")"), (Semi, ";"),
            (RBrac, "}"),

            (Keyword, "fn"), (Ident, "add34"), (LParen, "("), (Ident, "y"), (RParen, ")"),
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
