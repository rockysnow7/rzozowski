#![warn(unused_crate_dependencies)]

mod derivatives;
mod parser;

pub use derivatives::Regex;
pub use parser::parse_string_to_regex as parse;
