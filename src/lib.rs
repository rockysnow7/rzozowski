#![warn(unused_crate_dependencies)]

//! *rzozowski* (ruh-zov-ski) is a Rust crate for reasoning about regular expressions in terms of Brzozowski derivatives.

mod derivatives;
mod parser;

pub use derivatives::{Regex, Count, CharRange};
