#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::excessive_nesting,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
mod base62_tests;
mod boundary_tests;
mod concurrent_tests;
mod config_tests;
mod core_tests;
mod edge_case_tests;
mod extraction_tests;
mod sequence_tests;
pub mod test_utils;
mod timestamp_tests;
mod timing_tests;
