use std::fmt::{self, Debug};
use crate::parser::parse_string_to_regex;

pub const CLASS_ESCAPE_CHARS: &[char] = &['[', ']', '-', '\\'];
pub const NON_CLASS_ESCAPE_CHARS: &[char] = &['[', ']', '-', '(', ')', '{', '}', '?', '*', '+', '|', '\\', '.'];

fn escape_regex_char(c: char, in_class: bool) -> String {
    let to_escape = if in_class {
        CLASS_ESCAPE_CHARS
    } else {
        NON_CLASS_ESCAPE_CHARS
    };

    if to_escape.contains(&c) {
        format!("\\{}", c)
    } else {
        c.to_string()
    }
}

/// A struct that represents a set of characters to be matched in a character class.
#[derive(Clone, PartialEq, Eq)]
pub enum CharRange {
    /// A single character (e.g., `a`).
    Single(char),
    /// A range of characters (e.g., `a-z`).
    Range(char, char),
}

impl ToString for CharRange {
    fn to_string(&self) -> String {
        match self {
            CharRange::Single(c) => escape_regex_char(*c, true),
            CharRange::Range(start, end) => format!("{}-{}", escape_regex_char(*start, true), escape_regex_char(*end, true)),
        }
    }
}

impl Debug for CharRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl CharRange {
    /// Returns `true` if the given character is in the range, otherwise returns `false`.
    fn contains(&self, c: &char) -> bool {
        match self {
            CharRange::Single(ch) => *ch == *c,
            CharRange::Range(start, end) => *start <= *c && *c <= *end,
        }
    }
}

/// A struct that represents the number of times a regex can match. If `max` is `None`, the regex must match exactly `min` times. If `max` is `Some(n)`, the regex must match between `min` and `n` times (inclusive).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Count {
    pub min: usize,
    pub max: Option<usize>,
}

impl ToString for Count {
    fn to_string(&self) -> String {
        if let Some(max) = self.max {
            format!("{{{},{}}}", self.min, max)
        } else {
            format!("{{{}}}", self.min)
        }
    }
}

impl Debug for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// A regular expression.
#[derive(Clone, PartialEq, Eq)]
pub enum Regex {
    /// A regex that does not match any strings.
    Empty,
    /// A regex that matches the empty string.
    Epsilon,
    /// A regex that matches a single character (e.g., `a`).
    Literal(char),
    /// A regex that matches a concatenation of two regexes (e.g., `ab`).
    Concat(Box<Regex>, Box<Regex>),
    /// A regex that matches an alternation of two regexes (e.g., `a|b`).
    Or(Box<Regex>, Box<Regex>),
    /// A regex that matches zero or one of the given regex (e.g., `a?`).
    ZeroOrOne(Box<Regex>),
    /// A regex that matches zero or more of the given regex (e.g., `a*`).
    ZeroOrMore(Box<Regex>),
    /// A regex that matches one or more of the given regex (e.g., `a+`).
    OneOrMore(Box<Regex>),
    /// A regex that matches any character in the given character class (e.g., `[a-z]`).
    Class(Vec<CharRange>),
    /// A regex that matches a given regex a specified number of times (e.g., `a{3}` or `a{3,5}`).
    Count(Box<Regex>, Count),
}

impl ToString for Regex {
    fn to_string(&self) -> String {
        match self {
            Regex::Empty => "∅".to_string(),
            Regex::Epsilon => "ε".to_string(),
            Regex::Literal(c) => escape_regex_char(*c, false),
            Regex::Concat(left, right) => format!("{}{}", left.to_string(), right.to_string()),
            Regex::Or(left, right) => format!("({}|{})", left.to_string(), right.to_string()),
            Regex::ZeroOrOne(inner) => format!("({})?", inner.to_string()),
            Regex::ZeroOrMore(inner) => format!("({})*", inner.to_string()),
            Regex::OneOrMore(inner) => format!("({})+", inner.to_string()),
            Regex::Class(ranges) => {
                let ranges_str = ranges.iter().map(|range| range.to_string()).collect::<Vec<String>>().join("");
                format!("[{}]", ranges_str)
            }
            Regex::Count(inner, quantifier) => {
                format!("({}){}", inner.to_string(), quantifier.to_string())
            },
        }
    }
}

impl Debug for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Regex {
    fn is_nullable_(&self) -> bool {
        match self {
            Regex::Empty => false,
            Regex::Epsilon => true,
            Regex::Literal(_) => false,
            Regex::Concat(left, right) => left.is_nullable_() && right.is_nullable_(),
            Regex::Or(left, right) => left.is_nullable_() || right.is_nullable_(),
            Regex::ZeroOrOne(_) | Regex::ZeroOrMore(_) => true,
            Regex::OneOrMore(inner) => inner.is_nullable_(),
            Regex::Class(_) => false,
            Regex::Count(inner, quantifier) => {
                if quantifier.min == 0 {
                    true
                } else {
                    inner.is_nullable_()
                }
            },
        }
    }

    /// If the regex is nullable, returns `Regex::Epsilon`, otherwise returns `Regex::Empty`.
    pub fn is_nullable(&self) -> Regex {
        if self.is_nullable_() {
            Regex::Epsilon
        } else {
            Regex::Empty
        }
    }

    /// Returns the Brzozowski derivative of the regex with respect to a given character.
    pub fn derivative(&self, c: char) -> Regex {
        match self {
            Regex::Empty | Regex::Epsilon => Regex::Empty,
            Regex::Literal(ch) => {
                if *ch == c {
                    Regex::Epsilon
                } else {
                    Regex::Empty
                }
            },
            Regex::Concat(left, right) => {
                Regex::Or(
                    Box::new(Regex::Concat(
                        Box::new(left.derivative(c)),
                        right.clone(),
                    ).simplify()),
                    Box::new(Regex::Concat(
                        Box::new(left.is_nullable()),
                        Box::new(right.derivative(c)),
                    ).simplify()),
                )
            },
            Regex::Or(left, right) => {
                Regex::Or(
                    Box::new(left.derivative(c)),
                    Box::new(right.derivative(c)),
                )
            },
            Regex::ZeroOrOne(inner) => {
                Regex::Or(
                    inner.clone(),
                    Box::new(Regex::Epsilon),
                ).derivative(c)
            },
            Regex::ZeroOrMore(inner) => {
                Regex::Concat(
                    Box::new(inner.derivative(c)),
                    Box::new(Regex::ZeroOrMore(inner.clone()).simplify()),
                )
            },
            Regex::OneOrMore(inner) => {
                Regex::Concat(
                    inner.clone(),
                    Box::new(Regex::ZeroOrMore(inner.clone()).simplify()),
                ).derivative(c)
            },
            Regex::Class(ranges) => {
                for range in ranges {
                    if range.contains(&c) {
                        return Regex::Epsilon;
                    }
                }
                Regex::Empty
            },
            Regex::Count(inner, count) => {
                let new_count = Count {
                    min: count.min.saturating_sub(1),
                    max: count.max.map(|max| max.saturating_sub(1)),
                };

                Regex::Concat(
                    Box::new(inner.derivative(c)),
                    Box::new(Regex::Count(inner.clone(), new_count)),
                )
            }
        }.simplify()
    }

    /// Simplifies the regex.
    pub fn simplify(&self) -> Regex {
        match self {
            Regex::Empty => Regex::Empty,
            Regex::Epsilon => Regex::Epsilon,
            Regex::Literal(c) => Regex::Literal(*c),
            Regex::Concat(left, right) => {
                let left_simplified = left.simplify();
                let right_simplified = right.simplify();

                // r∅ = ∅r = ∅
                if left_simplified == Regex::Empty || right_simplified == Regex::Empty {
                    return Regex::Empty;
                }

                // εr = rε = r
                if left_simplified == Regex::Epsilon {
                    return right_simplified;
                }
                if right_simplified == Regex::Epsilon {
                    return left_simplified;
                }

                Regex::Concat(
                    Box::new(left_simplified),
                    Box::new(right_simplified),
                )
            },
            Regex::Or(left, right) => {
                let left_simplified = left.simplify();
                let right_simplified = right.simplify();

                // r ∪ ∅ = ∅ ∪ r = r
                if left_simplified == Regex::Empty {
                    return right_simplified;
                }
                if right_simplified == Regex::Empty {
                    return left_simplified;
                }

                // r ∪ r = r
                if left_simplified == right_simplified {
                    return left_simplified;
                }

                Regex::Or(
                    Box::new(left_simplified),
                    Box::new(right_simplified),
                )
            },
            Regex::ZeroOrOne(inner) => {
                Regex::ZeroOrOne(Box::new(inner.simplify()))
            },
            Regex::ZeroOrMore(inner) => {
                let inner_simplified = inner.simplify();

                // ∅* = ε* = ε
                if **inner == Regex::Empty || **inner == Regex::Epsilon {
                    return Regex::Epsilon;
                }

                // (r*)* = r*
                if let Regex::ZeroOrMore(_) = inner_simplified {
                    return inner_simplified;
                }

                Regex::ZeroOrMore(Box::new(inner.simplify()))
            },
            Regex::OneOrMore(inner) => {
                let inner_simplified = inner.simplify();

                // (ε)+ = ε
                if inner_simplified == Regex::Epsilon {
                    return Regex::Epsilon;
                }

                Regex::OneOrMore(Box::new(inner_simplified))
            },
            Regex::Class(ranges) => {
                let mut new_ranges = Vec::new();
                let mut changed = false;
                for range in ranges {
                    if let CharRange::Range(start, end) = range {
                        if start == end {
                            new_ranges.push(CharRange::Single(*start));
                            changed = true;
                        } else {
                            new_ranges.push(range.clone());
                        }
                    } else {
                        new_ranges.push(range.clone());
                    }
                }

                if changed {
                    return Regex::Class(new_ranges).simplify();
                }

                if ranges.len() == 1 {
                    if let CharRange::Single(c) = ranges[0] {
                        return Regex::Literal(c);
                    }
                }

                let mut new_ranges = ranges.clone();
                new_ranges.sort_unstable_by_key(|r| match r {
                    CharRange::Single(c) => *c,
                    CharRange::Range(start, _) => *start,
                });
                Regex::Class(new_ranges)
            },
            Regex::Count(inner, count) => {
                let inner_simplified = inner.simplify();

                // ∅{n} = ∅
                if inner_simplified == Regex::Empty {
                    return Regex::Empty;
                }
                // ε{n} = ε
                if inner_simplified == Regex::Epsilon {
                    return Regex::Epsilon;
                }

                // r{n,n} = r{n}
                if let Some(max) = count.max {
                    if count.min == max {
                        return Regex::Count(
                            Box::new(inner_simplified),
                            Count { min: count.min, max: None },
                        ).simplify();
                    }
                }

                // r{0} = ε
                if count.min == 0 && count.max.is_none() {
                    return Regex::Epsilon;
                }
                // r{1} = r
                if count.min == 1 && count.max.is_none() {
                    return inner_simplified;
                }

                Regex::Count(Box::new(inner_simplified), *count)
            },
        }
    }

    /// Returns `true` if the regex matches the given string, otherwise returns `false`.
    pub fn matches(&self, s: &str) -> bool {
        let mut current = self.clone();
        for c in s.chars() {
            current = current.derivative(c);
        }
        current.is_nullable_()
    }

    /// Tries to parse a string into a `Regex`.
    pub fn new(s: &str) -> Result<Regex, String> {
        parse_string_to_regex(s)
    }
}

mod tests {
    use super::*;

    // comprehensive derivative tests
    #[test]
    fn test_derivative_empty() {
        let regex = Regex::Empty;
        assert_eq!(regex.derivative('a'), Regex::Empty);
    }

    #[test]
    fn test_derivative_epsilon() {
        let regex = Regex::Epsilon;
        assert_eq!(regex.derivative('a'), Regex::Empty);
    }

    #[test]
    fn test_derivative_literal_match() {
        let regex = Regex::Literal('a');
        assert_eq!(regex.derivative('a'), Regex::Epsilon);
    }

    #[test]
    fn test_derivative_literal_no_match() {
        let regex = Regex::Literal('a');
        assert_eq!(regex.derivative('b'), Regex::Empty);
    }

    #[test]
    fn test_derivative_concat_first_char() {
        let regex = Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Literal('b'))
        );
        assert_eq!(regex.derivative('a'), Regex::Literal('b'));
    }

    #[test]
    fn test_derivative_concat_nullable_first() {
        let regex = Regex::Concat(
            Box::new(Regex::ZeroOrOne(Box::new(Regex::Literal('a')))),
            Box::new(Regex::Literal('b'))
        );
        assert_eq!(regex.derivative('b'), Regex::Epsilon);
    }

    #[test]
    fn test_derivative_or_left_match() {
        let regex = Regex::Or(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Literal('b'))
        );
        assert_eq!(regex.derivative('a'), Regex::Epsilon);
    }

    #[test]
    fn test_derivative_or_right_match() {
        let regex = Regex::Or(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Literal('b'))
        );
        assert_eq!(regex.derivative('b'), Regex::Epsilon);
    }

    #[test]
    fn test_derivative_or_no_match() {
        let regex = Regex::Or(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Literal('b'))
        );
        assert_eq!(regex.derivative('c'), Regex::Empty);
    }

    #[test]
    fn test_derivative_zero_or_one() {
        let regex = Regex::ZeroOrOne(Box::new(Regex::Literal('a')));
        assert_eq!(regex.derivative('a'), Regex::Epsilon);
        assert_eq!(regex.derivative('b'), Regex::Empty);
    }

    #[test]
    fn test_derivative_zero_or_more_match() {
        let regex = Regex::ZeroOrMore(Box::new(Regex::Literal('a')));
        let result = regex.derivative('a');
        assert_eq!(result, Regex::ZeroOrMore(Box::new(Regex::Literal('a'))));
    }

    #[test]
    fn test_derivative_zero_or_more_no_match() {
        let regex = Regex::ZeroOrMore(Box::new(Regex::Literal('a')));
        assert_eq!(regex.derivative('b'), Regex::Empty);
    }

    #[test]
    fn test_derivative_one_or_more_match() {
        let regex = Regex::OneOrMore(Box::new(Regex::Literal('a')));
        let result = regex.derivative('a');
        assert_eq!(result, Regex::ZeroOrMore(Box::new(Regex::Literal('a'))));
    }

    #[test]
    fn test_derivative_one_or_more_no_match() {
        let regex = Regex::OneOrMore(Box::new(Regex::Literal('a')));
        assert_eq!(regex.derivative('b'), Regex::Empty);
    }

    #[test]
    fn test_derivative_class_match() {
        let regex = Regex::Class(vec![
            CharRange::Single('a'),
            CharRange::Range('c', 'e')
        ]);
        assert_eq!(regex.derivative('a'), Regex::Epsilon);
        assert_eq!(regex.derivative('d'), Regex::Epsilon);
    }

    #[test]
    fn test_derivative_class_no_match() {
        let regex = Regex::Class(vec![
            CharRange::Single('a'),
            CharRange::Range('c', 'e')
        ]);
        assert_eq!(regex.derivative('b'), Regex::Empty);
        assert_eq!(regex.derivative('f'), Regex::Empty);
    }

    #[test]
    fn test_derivative_count_match() {
        let regex = Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 2, max: Some(3) }
        );
        let result = regex.derivative('a');
        assert_eq!(result, Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 1, max: Some(2) }
        ));
    }

    #[test]
    fn test_derivative_count_no_match() {
        let regex = Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 2, max: Some(3) }
        );
        assert_eq!(regex.derivative('b'), Regex::Empty);
    }

    #[test]
    fn test_derivative_complex_pattern() {
        // Pattern: a(b|c)*d
        let regex = Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Concat(
                Box::new(Regex::ZeroOrMore(Box::new(Regex::Or(
                    Box::new(Regex::Literal('b')),
                    Box::new(Regex::Literal('c'))
                )))),
                Box::new(Regex::Literal('d'))
            ))
        );
        
        // Take derivative with respect to 'a'
        let d1 = regex.derivative('a');
        assert_eq!(d1, Regex::Concat(
            Box::new(Regex::ZeroOrMore(Box::new(Regex::Or(
                Box::new(Regex::Literal('b')),
                Box::new(Regex::Literal('c'))
            )))),
            Box::new(Regex::Literal('d'))
        ));
        
        // Take derivative with respect to 'b'
        let d2 = d1.derivative('b');
        assert_eq!(d2, Regex::Concat(
            Box::new(Regex::ZeroOrMore(Box::new(Regex::Or(
                Box::new(Regex::Literal('b')),
                Box::new(Regex::Literal('c'))
            )))),
            Box::new(Regex::Literal('d'))
        ));
        
        // Take derivative with respect to 'd'
        let d3 = d2.derivative('d');
        assert_eq!(d3, Regex::Epsilon);
    }

    // comprehensive simplify tests
    #[test]
    fn test_simplify_empty() {
        let regex = Regex::Empty;
        assert_eq!(regex.simplify(), Regex::Empty);
    }
    
    #[test]
    fn test_simplify_epsilon() {
        let regex = Regex::Epsilon;
        assert_eq!(regex.simplify(), Regex::Epsilon);
    }
    
    #[test]
    fn test_simplify_literal() {
        let regex = Regex::Literal('a');
        assert_eq!(regex.simplify(), Regex::Literal('a'));
    }
    
    #[test]
    fn test_simplify_concat_with_empty() {
        // r∅ = ∅
        let regex = Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Empty)
        );
        assert_eq!(regex.simplify(), Regex::Empty);
        
        // ∅r = ∅
        let regex = Regex::Concat(
            Box::new(Regex::Empty),
            Box::new(Regex::Literal('a'))
        );
        assert_eq!(regex.simplify(), Regex::Empty);
    }
    
    #[test]
    fn test_simplify_concat_with_epsilon() {
        // rε = r
        let regex = Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Epsilon)
        );
        assert_eq!(regex.simplify(), Regex::Literal('a'));
        
        // εr = r
        let regex = Regex::Concat(
            Box::new(Regex::Epsilon),
            Box::new(Regex::Literal('a'))
        );
        assert_eq!(regex.simplify(), Regex::Literal('a'));
    }
    
    #[test]
    fn test_simplify_or_with_empty() {
        // r ∪ ∅ = r
        let regex = Regex::Or(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Empty)
        );
        assert_eq!(regex.simplify(), Regex::Literal('a'));
        
        // ∅ ∪ r = r
        let regex = Regex::Or(
            Box::new(Regex::Empty),
            Box::new(Regex::Literal('a'))
        );
        assert_eq!(regex.simplify(), Regex::Literal('a'));
    }
    
    #[test]
    fn test_simplify_or_with_same() {
        // r ∪ r = r
        let regex = Regex::Or(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Literal('a'))
        );
        assert_eq!(regex.simplify(), Regex::Literal('a'));
    }
    
    #[test]
    fn test_simplify_zero_or_more() {
        // ∅* = ε
        let regex = Regex::ZeroOrMore(Box::new(Regex::Empty));
        assert_eq!(regex.simplify(), Regex::Epsilon);

        // ε* = ε
        let regex = Regex::ZeroOrMore(Box::new(Regex::Epsilon));
        assert_eq!(regex.simplify(), Regex::Epsilon);

        // (r*)* = r*
        let inner = Regex::ZeroOrMore(Box::new(Regex::Literal('a')));
        let regex = Regex::ZeroOrMore(Box::new(inner));
        assert_eq!(regex.simplify(), Regex::ZeroOrMore(Box::new(Regex::Literal('a'))));
    }
    
    #[test]
    fn test_simplify_one_or_more() {
        // ε+ = ε
        let regex = Regex::OneOrMore(Box::new(Regex::Epsilon));
        assert_eq!(regex.simplify(), Regex::Epsilon);
    }
    
    #[test]
    fn test_simplify_class() {
        // Single char class to literal
        let regex = Regex::Class(vec![CharRange::Single('a')]);
        assert_eq!(regex.simplify(), Regex::Literal('a'));
        
        // Range with same start and end becomes single
        let regex = Regex::Class(vec![CharRange::Range('a', 'a')]);
        assert_eq!(regex.simplify(), Regex::Literal('a'));
        
        // Test sorting
        let regex = Regex::Class(vec![
            CharRange::Single('c'),
            CharRange::Single('a'),
            CharRange::Range('d', 'f')
        ]);
        assert_eq!(regex.simplify(), Regex::Class(vec![
            CharRange::Single('a'),
            CharRange::Single('c'),
            CharRange::Range('d', 'f')
        ]));
    }
    
    #[test]
    fn test_simplify_count() {
        // ∅{n} = ∅
        let regex = Regex::Count(
            Box::new(Regex::Empty),
            Count { min: 2, max: Some(3) }
        );
        assert_eq!(regex.simplify(), Regex::Empty);
        
        // ε{n} = ε
        let regex = Regex::Count(
            Box::new(Regex::Epsilon),
            Count { min: 2, max: Some(3) }
        );
        assert_eq!(regex.simplify(), Regex::Epsilon);
        
        // r{n,n} = r{n}
        let regex = Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 2, max: Some(2) }
        );
        assert_eq!(regex.simplify(), Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 2, max: None }
        ));
        
        // r{0} = ε
        let regex = Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 0, max: None }
        );
        assert_eq!(regex.simplify(), Regex::Epsilon);
        
        // r{1} = r
        let regex = Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 1, max: None }
        );
        assert_eq!(regex.simplify(), Regex::Literal('a'));
    }
    
    #[test]
    fn test_complex_simplification() {
        // (a|∅)(ε|b*)
        let regex = Regex::Concat(
            Box::new(Regex::Or(
                Box::new(Regex::Literal('a')),
                Box::new(Regex::Empty)
            )),
            Box::new(Regex::Or(
                Box::new(Regex::Epsilon),
                Box::new(Regex::ZeroOrMore(Box::new(Regex::Literal('b'))))
            ))
        );
        
        // Should simplify to a(ε|b*) which further simplifies to a
        let simplified = regex.simplify();
        assert_eq!(simplified, Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Or(
                Box::new(Regex::Epsilon),
                Box::new(Regex::ZeroOrMore(Box::new(Regex::Literal('b'))))
            ))
        ));
    }

    // matches tests
    #[test]
    fn test_matches_literal() {
        let regex = Regex::Literal('a');
        assert!(regex.matches("a"));
        assert!(!regex.matches("b"));
    }

    #[test]
    fn test_matches_concat() {
        let regex = Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Literal('b'))
        );
        assert!(regex.matches("ab"));
        assert!(!regex.matches("a"));
        assert!(!regex.matches("b"));
    }

    #[test]
    fn test_matches_or() {
        let regex = Regex::Or(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Literal('b'))
        );
        assert!(regex.matches("a"));
        assert!(regex.matches("b"));
        assert!(!regex.matches("c"));
    }

    #[test]
    fn test_matches_zero_or_more() {
        let regex = Regex::ZeroOrMore(Box::new(Regex::Literal('a')));
        assert!(regex.matches(""));
        assert!(regex.matches("a"));
        assert!(regex.matches("aa"));
        assert!(!regex.matches("b"));
    }

    #[test]
    fn test_matches_complex() {
        let regex = Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::ZeroOrMore(Box::new(Regex::Literal('b')))
        )); // ab*
        assert!(regex.matches("a"));
        assert!(regex.matches("ab"));
        assert!(regex.matches("abb"));
        assert!(!regex.matches("b"));
        assert!(!regex.matches("aa"));
    }

    #[test]
    fn test_matches_count_range() {
        let regex = Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 2, max: Some(3) }
        );
        assert!(!regex.matches(""));
        assert!(!regex.matches("a"));
        assert!(regex.matches("aa"));
        assert!(regex.matches("aaa"));
        assert!(!regex.matches("aaaa"));
    }
    
    #[test]
    fn test_matches_count_single() {
        let regex = Regex::Count(
            Box::new(Regex::Literal('a')),
            Count { min: 2, max: None }
        );

        assert!(!regex.matches(""));
        assert!(!regex.matches("a"));
        assert!(regex.matches("aa"));
        assert!(!regex.matches("aaa"));
    }

    #[test]
    fn test_matches_class() {
        let regex = Regex::Class(vec![CharRange::Single('a'), CharRange::Single('b')]);
        assert!(regex.matches("a"));
        assert!(regex.matches("b"));
        assert!(!regex.matches("c"));
    }
}
