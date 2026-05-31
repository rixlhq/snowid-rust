//! # `SnowID`
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
pub use generator::{SnowID, TryGenerateError};
