#![warn(unused_crate_dependencies)]

//! *rzozowski* (ruh-zov-ski) is a Rust crate for reasoning about regular expressions in terms of Brzozowski derivatives.

#![warn(
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::uninlined_format_args,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_join,
    clippy::unnecessary_wraps,
    clippy::format_push_string,
    clippy::non_std_lazy_statics
)]

mod derivatives;
mod parser;

pub use derivatives::{Regex, Count, CharRange};
