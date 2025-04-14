use std::fmt;
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
pub enum Token {
    #[regex(r"[^(){}\[\]|*+?\-\\,%@.]", |lex| lex.slice().chars().next().unwrap())]
    Literal(char),
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("|")]
    Pipe,
    #[token("*")]
    Star,
    #[token("+")]
    Plus,
    #[token("?")]
    Question,
    #[token("-")]
    Hyphen,
    #[token(r"\")]
    Backslash,
    #[token(",")]
    Comma,
    #[token("%")]
    Percent,
    #[token(".")]
    Dot,
    #[token("@")]
    At,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Token {
    pub fn as_char(&self) -> Result<char, ()> {
        match self {
            Token::Literal(c) => Ok(*c),
            Token::OpenParen => Ok('('),
            Token::CloseParen => Ok(')'),
            Token::OpenCurly => Ok('{'),
            Token::CloseCurly => Ok('}'),
            Token::OpenBracket => Ok('['),
            Token::CloseBracket => Ok(']'),
            Token::Pipe => Ok('|'),
            Token::Star => Ok('*'),
            Token::Plus => Ok('+'),
            Token::Question => Ok('?'),
            Token::Hyphen => Ok('-'),
            Token::Backslash => Ok('\\'),
            Token::Comma => Ok(','),
            Token::Percent => Ok('%'),
            Token::Dot => Ok('.'),
            Token::At => Ok('@'),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn lex_unescaped_literal() {
        let input = "a";
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::Literal('a'))));
    }

    #[test]
    fn lex_escaped_literal() {
        let input = r"\[";
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::Backslash)));
        assert_eq!(lexer.next(), Some(Ok(Token::OpenBracket)));
    }
}
