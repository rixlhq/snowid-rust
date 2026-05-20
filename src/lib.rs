//! # SnowID
//!
//! A Rust implementation of a Snowflake-like ID generator with 42-bit timestamp.
//!
//! Generate 64-bit unique identifiers that are:
//! - ⚡️ Fast (~325ns per ID)
//! - 📈 Time-sorted
//! - 🔄 Monotonic
//! - 🔒 Thread-safe
//! - 🌐 Distributed-ready

#![forbid(unsafe_code)]
// Clippy lint configuration — keeps code small, readable, and AI-slop-free.
// These mirror our JS/TS (oxlint) and Go (golangci-lint) rules:
//   - max 3 params, max 50 lines/func, max depth 3, max complexity 10
//   - no long lines, no repetitive code, no oversized types
//   - no unwrap/expect/panic (use proper error types)
//   - no wildcard imports, no placeholder names

// === Restriction lints (deny) — always wrong in production code ===
// AI models love unwrap(), panic(), todo!(), dbg!(), println!() — forbid them.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::wildcard_imports
)]
// === Pedantic lints (warn) — catch AI verbosity and redundancy ===
#![warn(
    clippy::redundant_closure_for_method_calls,
    clippy::cloned_instead_of_copied,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::checked_conversions,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::redundant_type_annotations,
    clippy::manual_let_else,
    clippy::needless_continue,
    clippy::option_as_ref_cloned,
    clippy::is_digit_ascii_radix,
    clippy::str_to_string,
    clippy::trivially_copy_pass_by_ref,
    clippy::pattern_type_mismatch,
    clippy::match_like_matches_macro
)]
// === Complexity lints (warn) — prevent AI from writing overly complex code ===
#![warn(
    clippy::cognitive_complexity,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::excessive_nesting,
    clippy::large_enum_variant,
    clippy::large_stack_frames,
    clippy::type_repetition_in_bounds,
    clippy::verbose_bit_mask,
    clippy::branches_sharing_code
)]
// === Style lints (warn) — eliminate AI slop patterns ===
#![warn(
    clippy::needless_borrow,
    clippy::redundant_clone,
    clippy::redundant_closure,
    clippy::match_same_arms,
    clippy::single_match_else,
    clippy::manual_unwrap_or,
    clippy::manual_map,
    clippy::unnecessary_fold,
    clippy::needless_return,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::unnecessary_to_owned,
    clippy::inefficient_to_string,
    clippy::implicit_clone,
    clippy::bool_comparison,
    clippy::clone_on_copy,
    clippy::search_is_some,
    clippy::filter_next,
    clippy::boxed_local,
    clippy::box_default,
    clippy::redundant_else,
    clippy::manual_range_contains,
    clippy::manual_non_exhaustive,
    clippy::from_over_into,
    clippy::useless_conversion,
    clippy::double_must_use,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::self_named_constructors,
    clippy::enum_glob_use
)]

pub mod base62;
mod config;
mod error;
mod extractor;
mod generator;

#[cfg(test)]
pub mod tests;

// Re-export main types
pub use config::SnowIDConfig;
pub use error::SnowIDError;
pub use extractor::SnowIDExtractor;
pub use generator::SnowID;

// Re-export base62 types at crate root for backward compatibility
pub use base62::DecodeError as Base62DecodeError;
pub use base62::MAX_LEN as BASE62_MAX_LEN;
pub use base62::{decode as base62_decode, encode as base62_encode};
pub use base62::{encode_array as base62_encode_array, encode_into as base62_encode_into};
