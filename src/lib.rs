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
//   - modern Rust idioms only (no deprecated patterns)

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
// === Compiler-level lints (deny) — enforce modern Rust ===
#![deny(
    unused_qualifications,          // foo::bar instead of use foo::bar; bar
    trivial_casts,                  // i as i32 when i is already i32
    trivial_numeric_casts,          // redundant numeric casts
    redundant_semicolons,           // unnecessary trailing ;
    unused_allocation,              // unnecessary Box/Rc/Vec allocations
    impl_trait_redundant_captures   // use<...> on impl Trait that doesn't need it
)]
// === Pedantic lints (warn) — catch AI verbosity and redundancy ===
#![warn(
    clippy::redundant_closure_for_method_calls,
    clippy::cloned_instead_of_copied,
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
// === Modern Rust lints (warn) — enforce idiomatic modern Rust ===
// These catch old patterns that have better modern equivalents.
#![warn(
    // --- Modern idioms ---
    clippy::redundant_field_names,          // Point { x } not Point { x: x }
    clippy::redundant_static_lifetimes,     // &'static str when lifetime elides
    clippy::use_self,                       // Self instead of type name in impl
    clippy::option_if_let_else,             // if let Some(x) = o { x } else { y }
    clippy::if_then_some_else_none,         // map_or for Option/Result
    clippy::match_bool,                     // match on bool → if/else
    clippy::match_wildcard_for_single_variants, // _ match on single-variant enum
    clippy::needless_match,                 // match that could be simplified
    clippy::single_char_pattern,            // matching on char instead of str
    clippy::wildcard_enum_match_arm,        // wildcard match on enum

    // --- Iterator modernization ---
    clippy::explicit_counter_loop,          // for i in 0..x → enumerate()
    clippy::explicit_into_iter_loop,        // into_iter() when iter() suffices
    clippy::explicit_iter_loop,             // iter() when & suffices
    clippy::needless_collect,               // collect() when iterator suffices

    // --- Type/cast modernization ---
    clippy::fn_to_numeric_cast,             // use into/from instead of `as`
    clippy::fn_to_numeric_cast_with_truncation,
    clippy::ref_option_ref,                 // &Option<&T> → Option<&T>
    clippy::borrow_deref_ref,               // &*x when deref/coercion works

    // --- Unnecessary operations ---
    clippy::needless_pass_by_value,         // pass by ref when value is small
    clippy::unused_unit,                    // unnecessary () return
    clippy::unused_self,                    // self parameter that isn't used
    clippy::redundant_pub_crate,            // pub(crate) when private suffices

    // --- Safety ---
    clippy::undocumented_unsafe_blocks,     // unsafe blocks need safety comment
    clippy::transmute_undefined_repr,       // transmute between different layouts

    // --- Performance ---
    clippy::mem_forget,                     // mem::forget is usually wrong
    clippy::useless_vec,                    // vec![x; 1] instead of [x]
    clippy::vec_init_then_push,             // init empty vec then push → collect

    // --- Naming ---
    clippy::struct_excessive_bools,         // structs with too many bools
    clippy::separated_literal_suffix,       // 42_u32 not 42u32

    // --- Misc modernization ---
    clippy::or_fun_call,                    // or(fun()) → unwrap_or_else(|_| fun())
    clippy::range_minus_one,                // x..=y-1 → x..y
    clippy::range_plus_one,                 // x..y+1 → x..=y
    clippy::neg_cmp_op_on_partial_ord,      // negated comparison on PartialOrd
    clippy::unneeded_field_pattern,         // ref x, ref y in match when Copy
    clippy::verbose_file_reads,             // read_to_string when read_to_end suffices
    clippy::string_lit_as_bytes,            // b"foo" instead of "foo".as_bytes()
    clippy::zero_sized_map_values,          // map with zero-sized value types
    clippy::wrong_transmute                 // transmute between same-sized types
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
