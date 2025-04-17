use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use once_cell::sync::Lazy;
use regex;
use rzozowski;
use std::hint::black_box;

struct TestPattern {
    name: &'static str,
    pattern: &'static str,
    valid_string: String,
    invalid_string: String,
}

static TEST_PATTERNS: Lazy<[TestPattern; 14]> = Lazy::new(|| {
    [
        TestPattern {
            name: "concatenation",
            pattern: "abcdef",
            valid_string: "abcdef".to_string(),
            invalid_string: "abcde".to_string(),
        },
        TestPattern {
            name: "alternation",
            pattern: "a|b",
            valid_string: "a".to_string(),
            invalid_string: "c".to_string(),
        },
        TestPattern {
            name: "star",
            pattern: "a*",
            valid_string: "aaaa".to_string(),
            invalid_string: "b".to_string(),
        },
        TestPattern {
            name: "plus",
            pattern: "a+",
            valid_string: "aaaa".to_string(),
            invalid_string: "".to_string(),
        },
        TestPattern {
            name: "question",
            pattern: "a?",
            valid_string: "a".to_string(),
            invalid_string: "b".to_string(),
        },
        TestPattern {
            name: "class",
            pattern: "[a-z]",
            valid_string: "j".to_string(),
            invalid_string: "1".to_string(),
        },
        TestPattern {
            name: "special_char_sequences",
            pattern: r"\d\w",
            valid_string: "1_".to_string(),
            invalid_string: "a_".to_string(),
        },
        TestPattern {
            name: "count",
            pattern: "a{2,270}",
            valid_string: "a".repeat(269),
            invalid_string: "a".repeat(271),
        },
        TestPattern {
            name: "nested_star",
            pattern: "(a*b*c*)*d+",
            valid_string: "aaabbabbacacbacbcadddd".to_string(),
            invalid_string: "abcabcabccccc".to_string(),
        },
        TestPattern {
            name: "deep_nesting",
            pattern: "((a|b|c)(d|e|f)(g|h|i))+",
            valid_string: "adgbdhceia".to_string(),
            invalid_string: "adgbdhceid".to_string(),
        },
        TestPattern {
            name: "complex_count",
            pattern: "(a{2,5}b{3,7}c{1,9}){2,4}",
            valid_string: "aaabbbcaabbbbbbccaaaaabbbcc".to_string(),
            invalid_string: "aaabbbcaabbbbbbccaaaaabbbccaaaaaa".to_string(),
        },
        TestPattern {
            name: "complex_class",
            pattern: r"[a-zA-Z0-9_][a-z]{5,10}\d{3,6}",
            valid_string: "7knmavpp1234".to_string(),
            invalid_string: "_abcde12".to_string(),
        },
        TestPattern {
            name: "exponential_plus",
            pattern: "(a+)+b",
            valid_string: "aaaaaaaaaaab".to_string(),
            invalid_string: "aaaaaaaaaa".to_string(),
        },
        TestPattern {
            name: "email",
            pattern: r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z][a-zA-Z]+",
            valid_string: "test@example.com".to_string(),
            invalid_string: "test@example".to_string(),
        },
    ]
});

fn bench_regex_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("regex_parse");

    for pattern in TEST_PATTERNS.iter() {
        group.bench_with_input(
            BenchmarkId::new("rzozowski", pattern.name),
            pattern.pattern,
            |b, pat| {
                b.iter(|| {
                    black_box(match rzozowski::Regex::new(pat) {
                        Ok(re) => re,
                        Err(e) => {
                            eprintln!("Pattern: {pat}");
                            panic!("{e}");
                        }
                    })
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("regex", pattern.name),
            pattern.pattern,
            |b, pat| b.iter(|| black_box(regex::Regex::new(pat).unwrap())),
        );
    }

    group.finish();
}

fn bench_regex_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("regex_matches");

    for pattern in TEST_PATTERNS.iter() {
        let re = rzozowski::Regex::new(pattern.pattern).unwrap();
        group.bench_function(BenchmarkId::new("rzozowski-valid", pattern.name), |b| {
            b.iter(|| {
                black_box(re.matches(&pattern.valid_string));
            })
        });
        group.bench_function(BenchmarkId::new("rzozowski-invalid", pattern.name), |b| {
            b.iter(|| {
                black_box(re.matches(&pattern.invalid_string));
            })
        });

        let re = regex::Regex::new(pattern.pattern).unwrap();
        group.bench_function(BenchmarkId::new("regex-valid", pattern.name), |b| {
            b.iter(|| {
                black_box(re.is_match(&pattern.valid_string));
            })
        });
        group.bench_function(BenchmarkId::new("regex-invalid", pattern.name), |b| {
            b.iter(|| {
                black_box(re.is_match(&pattern.invalid_string));
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_regex_parse, bench_regex_matches);
criterion_main!(benches);
