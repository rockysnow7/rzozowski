#![deny(
    unsafe_code,
    clippy::undocumented_unsafe_blocks,
    clippy::multiple_unsafe_ops_per_block,
)]

#![warn(
    clippy::cognitive_complexity,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::format_push_string,
    clippy::missing_const_for_fn,
/*
 * The following lints (especially the first one) require a lot of documentation
 * to be written before being globally enabled, which is why they are left disabled
 * until the documentation has been written in a separate PR.
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
*/
    clippy::non_std_lazy_statics,
    clippy::option_if_let_else,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::semicolon_if_nothing_returned,
    clippy::similar_names,
    clippy::suspicious_operation_groupings,
    clippy::trivially_copy_pass_by_ref,
    clippy::uninlined_format_args,
    clippy::unnecessary_join,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_wraps,
    clippy::unseparated_literal_suffix,
    clippy::unused_self,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq,
    clippy::wildcard_dependencies,
    clippy::wildcard_imports,
    keyword_idents,
    missing_debug_implementations,
    noop_method_call,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
)]

//! *rzozowski* (ruh-zov-ski) is a Rust crate for reasoning about regular expressions in terms of Brzozowski derivatives.

mod derivatives;
mod parser;

pub use derivatives::{Regex, Count, CharRange};
