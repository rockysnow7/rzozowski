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
            Self::Literal(c) => Ok(*c),
            Self::OpenParen => Ok('('),
            Self::CloseParen => Ok(')'),
            Self::OpenCurly => Ok('{'),
            Self::CloseCurly => Ok('}'),
            Self::OpenBracket => Ok('['),
            Self::CloseBracket => Ok(']'),
            Self::Pipe => Ok('|'),
            Self::Star => Ok('*'),
            Self::Plus => Ok('+'),
            Self::Question => Ok('?'),
            Self::Hyphen => Ok('-'),
            Self::Backslash => Ok('\\'),
            Self::Comma => Ok(','),
            Self::Percent => Ok('%'),
            Self::Dot => Ok('.'),
            Self::At => Ok('@'),
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
