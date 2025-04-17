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
        write!(f, "{self:?}")
    }
}

impl Token {
    pub fn as_char(&self) -> char {
        match self {
            Token::Literal(c) => *c,
            Token::OpenParen => '(',
            Token::CloseParen => ')',
            Token::OpenCurly => '{',
            Token::CloseCurly => '}',
            Token::OpenBracket => '[',
            Token::CloseBracket => ']',
            Token::Pipe => '|',
            Token::Star => '*',
            Token::Plus => '+',
            Token::Question => '?',
            Token::Hyphen => '-',
            Token::Backslash => '\\',
            Token::Comma => ',',
            Token::Percent => '%',
            Token::Dot => '.',
            Token::At => '@',
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
