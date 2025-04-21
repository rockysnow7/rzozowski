use logos::Logos;
use std::fmt;

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
        write!(f, "{self:?}")
    }
}

impl Token {
    pub const fn as_char(&self) -> char {
        match self {
            Self::Literal(c) => *c,
            Self::OpenParen => '(',
            Self::CloseParen => ')',
            Self::OpenCurly => '{',
            Self::CloseCurly => '}',
            Self::OpenBracket => '[',
            Self::CloseBracket => ']',
            Self::Pipe => '|',
            Self::Star => '*',
            Self::Plus => '+',
            Self::Question => '?',
            Self::Hyphen => '-',
            Self::Backslash => '\\',
            Self::Comma => ',',
            Self::Percent => '%',
            Self::Dot => '.',
            Self::At => '@',
        }
    }
}

mod tests {
    // Not quite sure why this triggers here, possibly the include is too "broad"
    // The code fails to compile without the use statement, yet clippy isn't happy about it being
    // there.
    #[allow(unused_imports)]
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
