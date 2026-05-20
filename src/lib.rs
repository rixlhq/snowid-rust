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
#![warn(
    clippy::cognitive_complexity,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::excessive_nesting,
    clippy::large_enum_variant,
    clippy::large_stack_frames,
    clippy::type_repetition_in_bounds,
    clippy::verbose_bit_mask,
    clippy::needless_borrow,
    clippy::redundant_clone,
    clippy::redundant_closure,
    clippy::match_same_arms,
    clippy::single_match_else,
    clippy::manual_unwrap_or,
    clippy::manual_map,
    clippy::unnecessary_fold,
    clippy::branches_sharing_code,
    clippy::needless_return,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::unnecessary_to_owned,
    clippy::inefficient_to_string,
    clippy::implicit_clone
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
