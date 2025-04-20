mod lexer;

use crate::derivatives::{CharRange, Count, Regex, CLASS_ESCAPE_CHARS, NON_CLASS_ESCAPE_CHARS};
use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use lexer::Token;
use logos::Logos;
use std::fmt::Write as _;
use std::{collections::HashMap, sync::LazyLock};

/// Represents a regex in a more convenient format for parsing. This is an intermediate representation before converting to the final `Regex` type.
#[derive(Clone)]
enum RegexRepresentation {
    Literal(char),
    Concat(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Optional(Box<Self>),
    Star(Box<Self>),
    Plus(Box<Self>),
    Class(Vec<CharRange>),
    Count(Box<Self>, Count),
}

impl RegexRepresentation {
    fn to_regex(&self) -> Regex {
        match self {
            Self::Literal(c) => Regex::Literal(*c),
            Self::Concat(left, right) => {
                Regex::Concat(Box::new(left.to_regex()), Box::new(right.to_regex()))
            }
            Self::Or(left, right) => {
                Regex::Or(Box::new(left.to_regex()), Box::new(right.to_regex()))
            }
            Self::Optional(inner) => inner.to_regex().optional(),
            Self::Star(inner) => inner.to_regex().star(),
            Self::Plus(inner) => inner.to_regex().plus(),
            Self::Class(ranges) => Regex::Class(ranges.clone()),
            Self::Count(inner, count) => Regex::Count(Box::new(inner.to_regex()), *count),
        }
    }
}

/// A map of special character sequences to their corresponding `RegexRepresentation`. For example, `\d` maps to `[0-9]`.
static SPECIAL_CHAR_SEQUENCES: LazyLock<HashMap<char, RegexRepresentation>> = LazyLock::new(|| {
    HashMap::from([
        // "\d" => [0-9]
        (
            'd',
            RegexRepresentation::Class(vec![CharRange::Range('0', '9')]),
        ),
        // "\w" => [a-zA-Z0-9_]
        (
            'w',
            RegexRepresentation::Class(vec![
                CharRange::Range('a', 'z'),
                CharRange::Range('A', 'Z'),
                CharRange::Range('0', '9'),
                CharRange::Single('_'),
            ]),
        ),
    ])
});

fn tokenize_string(input: &str) -> Result<Vec<Token>, String> {
    let lexer = Token::lexer(input);
    let tokens = lexer
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid token in input".to_string())?;

    if tokens.is_empty() {
        return Err("Empty input not allowed".to_string());
    }

    Ok(tokens)
}

/// Parses an unescaped character (e.g., `a`).
fn unescaped_char<'a, I>() -> impl Parser<'a, I, char, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    any()
        .filter(|token: &Token| {
            let c = token.as_char();
            !NON_CLASS_ESCAPE_CHARS.contains(&c)
        })
        .map(|token| token.as_char())
}

/// Parses an escaped character (e.g., `\[`).
fn escaped_char<'a, I>() -> impl Parser<'a, I, char, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    just(Token::Backslash)
        .then(any())
        .filter(|(_, token): &(_, Token)| {
            let c = token.as_char();
            NON_CLASS_ESCAPE_CHARS.contains(&c)
        })
        .map(|(_, token)| token.as_char())
}

/// Parses a special character sequence (e.g., `\d`).
fn special_char_sequence<'a, I>(
) -> impl Parser<'a, I, RegexRepresentation, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    just(Token::Backslash)
        .then(any().filter(|token| matches!(token, Token::Literal(_))))
        .filter(|(_, token)| {
            let c = token.as_char();
            SPECIAL_CHAR_SEQUENCES.contains_key(&c)
        })
        .map(|(_, token)| {
            let c = token.as_char();
            SPECIAL_CHAR_SEQUENCES[&c].clone()
        })
}

/// Parses a literal (e.g., `a`, `\[`, `\d`).
fn literal<'a, I>() -> impl Parser<'a, I, RegexRepresentation, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    special_char_sequence()
        .boxed()
        .or(escaped_char().map(RegexRepresentation::Literal))
        .or(unescaped_char().map(RegexRepresentation::Literal))
}

/// Parses an unescaped character that is not a special character sequence (e.g., `a`, `0`, `_`).
fn class_unescaped_char<'a, I>() -> impl Parser<'a, I, char, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    any()
        .filter(|token| {
            matches!(
                token,
                Token::Literal(_) | Token::Percent | Token::Plus | Token::Dot | Token::At
            )
        })
        .filter(|token| !CLASS_ESCAPE_CHARS.contains(&token.as_char()))
        .map(|token| token.as_char())
}

/// Parses an escaped character that is not a special character sequence (e.g., `\[`, `\]`, `\-`).
fn class_escaped_char<'a, I>() -> impl Parser<'a, I, char, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    just(Token::Backslash)
        .then(any())
        .filter(|(_, token): &(_, Token)| {
            let c = token.as_char();
            CLASS_ESCAPE_CHARS.contains(&c)
        })
        .map(|(_, token)| token.as_char())
}

/// Parses a class character.
fn class_char<'a, I>() -> impl Parser<'a, I, char, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    class_escaped_char().or(class_unescaped_char())
}

/// Parses a single class character into a `CharRange`.
fn class_range_single<'a, I>() -> impl Parser<'a, I, CharRange, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    class_char().map(CharRange::Single)
}

/// Parses a character range (e.g., `a-z`, `\--0`) into a `CharRange`.
fn class_range_range<'a, I>() -> impl Parser<'a, I, CharRange, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    class_char()
        .then_ignore(just(Token::Hyphen))
        .then(class_char())
        .map(|(start, end)| CharRange::Range(start, end))
}

/// Parses a character range (e.g., `a-z`, `a-zA-Z0-9`, `a-zA`).
fn class_range<'a, I>() -> impl Parser<'a, I, CharRange, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    class_range_range().or(class_range_single())
}

/// Parses a character class (e.g., `[a-z]`, `[a-zA-Z0-9]`, `[a-zA]`, `[\--0]`).
fn class<'a, I>() -> impl Parser<'a, I, RegexRepresentation, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    class_range()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket))
        .map(RegexRepresentation::Class)
}

/// Parses a parenthesized expression (e.g., `(a)`, `(a|b)`, `(a*)`, `(a+)`, `(a?)`).
fn parenthesized<'a, I>(
    regex: impl Parser<'a, I, RegexRepresentation, extra::Err<Rich<'a, Token>>>,
) -> impl Parser<'a, I, RegexRepresentation, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    regex.delimited_by(just(Token::OpenParen), just(Token::CloseParen))
}

#[derive(Clone)]
enum RepetitionKind {
    ZeroOrOne,
    ZeroOrMore,
    OneOrMore,
    Count(Count),
}

/// Parses a digit (e.g., `3`).
fn parse_digit<'a, I>() -> impl Parser<'a, I, char, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    any()
        .filter(|token| matches!(token, Token::Literal(_)))
        .filter(|token| {
            let c = token.as_char();
            c.is_ascii_digit()
        })
        .map(|token| token.as_char())
}

/// Parses a number (e.g., `3`, `42`).
fn parse_number<'a, I>() -> impl Parser<'a, I, usize, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    parse_digit()
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|digits| digits.iter().collect::<String>().parse::<usize>().unwrap())
}

/// Parses a `Count::Exact` (e.g., `{3}`).
fn parse_count_exact<'a, I>() -> impl Parser<'a, I, Count, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    just(Token::OpenCurly)
        .ignore_then(parse_number())
        .then_ignore(just(Token::CloseCurly))
        .map(Count::Exact)
}

/// Parses a `Count::Range` (e.g., `{3,5}`).
fn parse_count_range<'a, I>() -> impl Parser<'a, I, Count, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    just(Token::OpenCurly)
        .ignore_then(parse_number())
        .then_ignore(just(Token::Comma))
        .then(parse_number())
        .then_ignore(just(Token::CloseCurly))
        .map(|(min, max)| Count::Range(min, max))
}

/// Parses a `Count::AtLeast` (e.g., `{3,}`).
fn parse_count_at_least<'a, I>() -> impl Parser<'a, I, Count, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    just(Token::OpenCurly)
        .ignore_then(parse_number())
        .then_ignore(just(Token::Comma))
        .then_ignore(just(Token::CloseCurly))
        .map(Count::AtLeast)
}

/// Parses a count (e.g., `{3}`, `{3,5}`, `{3,}`).
fn parse_count<'a, I>() -> impl Parser<'a, I, Count, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    parse_count_range()
        .or(parse_count_at_least())
        .or(parse_count_exact())
}

/// Parses an optional repetition operation (e.g., `*`, `+`, `?`, `{3}`, `{3,5}`, or nothing).
fn parse_repetition<'a, I>(
) -> impl Parser<'a, I, Option<RepetitionKind>, extra::Err<Rich<'a, Token>>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    let simple_repetition = select! {
        Token::Star => Some(RepetitionKind::ZeroOrMore),
        Token::Plus => Some(RepetitionKind::OneOrMore),
        Token::Question => Some(RepetitionKind::ZeroOrOne),
    };

    let count_repetition = parse_count().map(RepetitionKind::Count).map(Some);

    count_repetition
        .or(simple_repetition)
        .or(empty().map(|_| None))
        .boxed()
}

fn parser<'a, I>() -> impl Parser<'a, I, RegexRepresentation, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    recursive(|regex| {
        let atom = literal()
            .boxed()
            .or(class().boxed())
            .or(parenthesized(regex).boxed());

        let repetition = atom
            .then(parse_repetition())
            .map(|(atom, repetition)| match repetition {
                Some(RepetitionKind::ZeroOrOne) => RegexRepresentation::Optional(Box::new(atom)),
                Some(RepetitionKind::ZeroOrMore) => RegexRepresentation::Star(Box::new(atom)),
                Some(RepetitionKind::OneOrMore) => RegexRepresentation::Plus(Box::new(atom)),
                Some(RepetitionKind::Count(count)) => {
                    RegexRepresentation::Count(Box::new(atom), count)
                }
                None => atom,
            });

        let concatenation = repetition
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|regexes| {
                regexes
                    .into_iter()
                    .reduce(|acc, regex| {
                        RegexRepresentation::Concat(Box::new(acc), Box::new(regex))
                    })
                    .unwrap()
            });

        #[allow(clippy::let_and_return)]
        let alternation = concatenation
            .separated_by(just(Token::Pipe))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|regexes| {
                regexes
                    .into_iter()
                    .reduce(|acc, regex| RegexRepresentation::Or(Box::new(acc), Box::new(regex)))
                    .unwrap()
            });

        alternation
    })
}

/// Tries to parse a given string into a `Regex` object.
pub fn parse_string_to_regex(input: &str) -> Result<Regex, String> {
    let tokens = tokenize_string(input).map_err(|_| "Failed to tokenize input".to_string())?;

    if tokens.is_empty() {
        return Err("Empty input not allowed".to_string());
    }

    let result = parser().parse(Stream::from_iter(tokens)).into_result();

    match result {
        Ok(regex) => Ok(regex.to_regex().simplify()),
        Err(errors) => {
            let mut error_message = String::new();
            for error in errors {
                let span = error.span();
                let found = error
                    .found()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "end of input".to_string());
                let expected = error.expected().map(|t| t.to_string()).collect::<Vec<_>>();

                let _ = writeln!(
                    error_message,
                    "Error at position {}: found {}, expected one of: {}",
                    span.start,
                    found,
                    expected.join(", ")
                );
            }

            Err(error_message)
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
    fn parse_literal() {
        let regex = parse_string_to_regex("d").unwrap();
        assert_eq!(regex, Regex::Literal('d'));
    }

    #[test]
    fn parse_literal_escaped() {
        let regex = parse_string_to_regex(r"\+").unwrap();
        assert_eq!(regex, Regex::Literal('+'));
    }

    #[test]
    fn parse_literal_parenthesized() {
        let regex = parse_string_to_regex("(a)").unwrap();
        assert_eq!(regex, Regex::Literal('a'));
    }

    #[test]
    fn parse_character_class_simple() {
        let regex = parse_string_to_regex("[a-z]").unwrap();
        assert_eq!(regex, Regex::Class(vec![CharRange::Range('a', 'z')]));
    }

    #[test]
    fn parse_character_class_long() {
        let regex = parse_string_to_regex("[a-zA-Z0-9]").unwrap();
        assert_eq!(
            regex,
            Regex::Class(vec![
                CharRange::Range('a', 'z'),
                CharRange::Range('A', 'Z'),
                CharRange::Range('0', '9'),
            ])
            .simplify()
        );
    }

    #[test]
    fn parse_character_class_mixed() {
        let regex = parse_string_to_regex("[a-zA]").unwrap();
        assert_eq!(
            regex,
            Regex::Class(vec![CharRange::Range('a', 'z'), CharRange::Single('A'),]).simplify()
        );
    }

    #[test]
    fn parse_special_character_sequence() {
        let regex = parse_string_to_regex(r"\d").unwrap();
        assert_eq!(regex, Regex::Class(vec![CharRange::Range('0', '9')]));
    }

    #[test]
    fn parse_character_class_escaped_characters() {
        let regex = parse_string_to_regex(r"[\--0]").unwrap();
        assert_eq!(regex, Regex::Class(vec![CharRange::Range('-', '0')]));
    }

    #[test]
    fn parse_repetition_star() {
        let regex = parse_string_to_regex("a*").unwrap();
        assert_eq!(regex, Regex::Literal('a').star());
    }

    #[test]
    fn parse_repetition_plus() {
        let regex = parse_string_to_regex("a+").unwrap();
        assert_eq!(regex, Regex::Literal('a').plus());
    }

    #[test]
    fn parse_repetition_question() {
        let regex = parse_string_to_regex("a?").unwrap();
        assert_eq!(regex, Regex::Literal('a').optional());
    }

    #[test]
    fn parse_repetition_count_exact() {
        let regex = parse_string_to_regex("a{3}").unwrap();
        assert_eq!(
            regex,
            Regex::Count(Box::new(Regex::Literal('a')), Count::Exact(3))
        );
    }

    #[test]
    fn parse_repetition_count_range() {
        let regex = parse_string_to_regex("a{3,5}").unwrap();
        assert_eq!(
            regex,
            Regex::Count(Box::new(Regex::Literal('a')), Count::Range(3, 5))
        );
    }

    #[test]
    fn parse_repetition_count_at_least() {
        let regex = parse_string_to_regex("a{3,}").unwrap();
        assert_eq!(
            regex,
            Regex::Count(Box::new(Regex::Literal('a')), Count::AtLeast(3))
        );
    }

    #[test]
    fn parse_concatenation() {
        let regex = parse_string_to_regex("ab").unwrap();
        assert_eq!(
            regex,
            Regex::Concat(Box::new(Regex::Literal('a')), Box::new(Regex::Literal('b')))
        );
    }

    #[test]
    fn parse_concatenation_three() {
        let regex = parse_string_to_regex("abc").unwrap();
        assert_eq!(
            regex,
            Regex::Concat(
                Box::new(Regex::Concat(
                    Box::new(Regex::Literal('a')),
                    Box::new(Regex::Literal('b')),
                )),
                Box::new(Regex::Literal('c')),
            )
        );
    }

    #[test]
    fn parse_concatenation_complex() {
        let regex = parse_string_to_regex("a(bc)*d[a-z]").unwrap();

        let bc = Regex::Concat(Box::new(Regex::Literal('b')), Box::new(Regex::Literal('c')));
        let star = bc.star();
        let a_bc_star = Regex::Concat(Box::new(Regex::Literal('a')), Box::new(star));
        let a_bc_star_d = Regex::Concat(Box::new(a_bc_star), Box::new(Regex::Literal('d')));
        let class = Regex::Class(vec![CharRange::Range('a', 'z')]);
        let a_bc_star_d_class = Regex::Concat(Box::new(a_bc_star_d), Box::new(class));

        assert_eq!(regex, a_bc_star_d_class);
    }

    #[test]
    fn parse_alternation() {
        let regex = parse_string_to_regex("a|b").unwrap();
        assert_eq!(
            regex,
            Regex::Or(Box::new(Regex::Literal('a')), Box::new(Regex::Literal('b')))
        );
    }

    #[test]
    fn parse_alternation_three() {
        let regex = parse_string_to_regex("a|b|c").unwrap();

        assert_eq!(
            regex,
            Regex::Or(
                Box::new(Regex::Or(
                    Box::new(Regex::Literal('a')),
                    Box::new(Regex::Literal('b')),
                )),
                Box::new(Regex::Literal('c')),
            )
        );
    }

    #[test]
    fn parse_alternation_complex() {
        let regex = parse_string_to_regex("a*|(bc)?").unwrap();

        let a_star = Regex::Literal('a').star();
        let bc = Regex::Concat(Box::new(Regex::Literal('b')), Box::new(Regex::Literal('c')));
        let bc_optional = bc.optional();
        let a_star_or_bc_optional = Regex::Or(Box::new(a_star), Box::new(bc_optional));

        assert_eq!(regex, a_star_or_bc_optional);
    }

    #[test]
    fn parse_empty_character_class() {
        let regex = parse_string_to_regex("[]").unwrap();
        assert_eq!(regex, Regex::Class(vec![]));
    }

    #[test]
    fn parse_nested_parentheses() {
        let regex = parse_string_to_regex("((a|b)*c)+").unwrap();
        let a_or_b_star =
            Regex::Or(Box::new(Regex::Literal('a')), Box::new(Regex::Literal('b'))).star();
        let a_or_b_star_c = Regex::Concat(Box::new(a_or_b_star), Box::new(Regex::Literal('c')));
        let a_or_b_star_c_plus = a_or_b_star_c.plus();

        assert_eq!(regex, a_or_b_star_c_plus);
    }

    #[test]
    fn parse_unicode() {
        let regex = parse_string_to_regex("💕+").unwrap();
        assert_eq!(regex, Regex::Literal('💕').plus());
    }

    #[test]
    fn parse_hyphen() {
        let regex = parse_string_to_regex("a-z").unwrap();
        assert_eq!(regex, Regex::Concat(
            Box::new(Regex::Concat(
                Box::new(Regex::Literal('a')),
                Box::new(Regex::Literal('-')),
            )),
            Box::new(Regex::Literal('z')),
        ));
    }

    #[test]
    fn parse_invalid_syntax() {
        // test incomplete count
        let result = parse_string_to_regex("a{");
        assert!(result.is_err());

        // test incomplete character class
        let result = parse_string_to_regex("[a-");
        assert!(result.is_err());

        // test incomplete parentheses
        let result = parse_string_to_regex("(a");
        assert!(result.is_err());

        // test empty sequence
        let result = parse_string_to_regex("");
        assert!(result.is_err());

        // test empty alternation
        let result = parse_string_to_regex("|");
        assert!(result.is_err());
    }

    #[test]
    fn parse_email() {
        let pattern = r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}";
        let regex = parse_string_to_regex(pattern);
        println!("Error: {:?}", regex);
        assert!(regex.is_ok());
    }
}
