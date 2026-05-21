# Changelog

## [2.1.0](https://github.com/rixlhq/snowid-rust/compare/v2.0.3...v2.1.0) (2026-05-21)


### Features

* use strong CAS in cas_state to avoid spurious failures under contention ([952ed05](https://github.com/rixlhq/snowid-rust/commit/952ed05df8f133c2159a85afe0572762185d2314))


### Bug Fixes

* add must_use attribute to config/extractor functions ([d3d4d04](https://github.com/rixlhq/snowid-rust/commit/d3d4d04bdd0a69ef9cd7a049f4551c76661a7969))
* apply subagent clippy fixes for pattern_type_mismatch and const fn ([a713f54](https://github.com/rixlhq/snowid-rust/commit/a713f54f9bc3858e0dd6def74a5e35bdc3b34cdd))
* apply use_self and wildcard_enum_match_arm lints ([25449a4](https://github.com/rixlhq/snowid-rust/commit/25449a4a4132ed91fdfc0c4556fdcc4c3b3dd1a0))
* avoid sleeping after failed CAS on new millisecond claim in slow path ([6665fd8](https://github.com/rixlhq/snowid-rust/commit/6665fd822e5a4e4f0bcf67b2391cd4143e688c82))
* inline format args in SnowIDError display ([b93921d](https://github.com/rixlhq/snowid-rust/commit/b93921d9fd79f8e0ca722f0e17d73b752ea75a50))
* mark pure config/extractor functions as const ([ba24e36](https://github.com/rixlhq/snowid-rust/commit/ba24e36d6de9ee43629ab12f8524b6421e8299fc))
* resolve clippy warnings in base62.rs and time.rs ([a84533b](https://github.com/rixlhq/snowid-rust/commit/a84533b06c40e695f3caedfcc79e9d12e0a8ba8e))
* resolve clippy warnings in config/mod.rs ([a237705](https://github.com/rixlhq/snowid-rust/commit/a237705fb19b075169dfebb72567f46afeae9535))
* retry immediately on same-millisecond CAS contention in slow path ([b6e07ae](https://github.com/rixlhq/snowid-rust/commit/b6e07aee84bea9c333546b9a00c785cae5765d62))


### Performance Improvements

* **generator:** minimize nesting and resolve all remaining Clippy warnings across tests and examples ([833382c](https://github.com/rixlhq/snowid-rust/commit/833382c4dbe6e01b0691c80346a41b2862c891f0))
* **generator:** revert to compare_exchange_weak and optimize CAS sequence increment ([40424dd](https://github.com/rixlhq/snowid-rust/commit/40424dd6914da42d9ee1d16e5d687a0e0461d4de))

## [2.0.3](https://github.com/qeeqez/snowid-rust/compare/v2.0.2...v2.0.3) (2026-03-31)


### Bug Fixes

* **deps:** bump base62 from 2.2.3 to 2.2.4 in the dependencies group ([cc3eee2](https://github.com/qeeqez/snowid-rust/commit/cc3eee2eba2bf96403bc188fdfc3dd0fb2e7e8d4))

## [2.0.2](https://github.com/qeeqez/snowid-rust/compare/v2.0.1...v2.0.2) (2026-02-26)


### Bug Fixes

* get back release please builds ([2dde501](https://github.com/qeeqez/snowid-rust/commit/2dde50103126e9c817ff0b551b3af1d8748b9892))

## [2.0.0](https://github.com/qeeqez/snowid-rust/compare/v1.0.1...v2.0.0) (2026-01-30)


### ⚠ BREAKING CHANGES

* **perf:** optimize generator with combined atomic state and precomputed fields
* 1.0.0
* removed thiserror dependency

### release

* 1.0.0 ([fec917c](https://github.com/qeeqez/snowid-rust/commit/fec917c29455a25626a789c33e21bb042d410921))


### Features

* **perf:** optimize generate() with inline timestamp advancement ([693bbc2](https://github.com/qeeqez/snowid-rust/commit/693bbc2a0e5e4715bd04045aa71469b4d609d8d1))
* **perf:** optimize generator with combined atomic state and precomputed fields ([f081941](https://github.com/qeeqez/snowid-rust/commit/f081941eb0822805a783cfd7c8542537e3b29155))
* performance optimizations and zero-allocation base62 API ([f4090fc](https://github.com/qeeqez/snowid-rust/commit/f4090fc5905c0f385bda6042d98da9649fd24e9b))


### Bug Fixes

* replace absurd u16::MAX comparison with config max value ([6d91c8b](https://github.com/qeeqez/snowid-rust/commit/6d91c8b47fc6afb9f5a3f2b5dd1ba79c3c921019))
* revert coarsetime optimization to fix timestamp accuracy ([0179c89](https://github.com/qeeqez/snowid-rust/commit/0179c8919eeeec33baa549ada77c7d0d0611055f))
* silence false positive dead_code warnings for test-used methods ([766ec68](https://github.com/qeeqez/snowid-rust/commit/766ec6844a899b46ddfb578b747b2966f0f112f7))
* update readme ([58fbcc4](https://github.com/qeeqez/snowid-rust/commit/58fbcc4afa3096511a5e2239c61fdf735e6a66c4))


### Performance Improvements

* optimize ID generation with advanced techniques and rust 2024 features (~5-10% faster) ([23dcc2f](https://github.com/qeeqez/snowid-rust/commit/23dcc2fea9aa34fbe1bfc21d2bc9e4ca11c61b35))

## [1.0.1](https://github.com/qeeqez/snowid-rust/compare/v1.0.0...v1.0.1) (2026-01-29)


### Bug Fixes

* update readme ([58fbcc4](https://github.com/qeeqez/snowid-rust/commit/58fbcc4afa3096511a5e2239c61fdf735e6a66c4))

## [1.0.0](https://github.com/qeeqez/snowid-rust/compare/v0.3.0...v1.0.0) (2026-01-29)

### ⚠ BREAKING CHANGES

*   **deps:** Removed `thiserror` dependency to reduce binary size and compile times. Error types now implement `std::error::Error` directly.
*   **api:** Base62 encoding now encourages zero-allocation patterns.

### Features

*   **base62:** Added zero-allocation APIs `base62_encode_array` and `base62_encode_into` for high-performance encoding without heap allocation.
*   **core:** Integrated `coarsetime` for ~20x faster time queries (hybrid monotonic/wall-clock approach).
*   **concurrency:** Improved thread-safety and performance using optimized lock-free patterns for high-contention scenarios.
*   **config:** Enhanced spin-wait configuration for finer control over latency during sequence overflow.
*   **modernization:** Updated code to use Rust 2024 edition features.

### Performance Improvements

*   **optimization:** Significant reduction in generation latency (sub-350ns/op) via hot-path optimizations.
*   **memory:** Elimination of heap allocations in core generation paths.

### Bug Fixes

*   **ci:** Fixed formatting issues and streamlined CI workflows.
*   **deps:** Updated `chrono`, `base62`, and `criterion` to latest stable versions.

### Miscellaneous

*   **ci:** Migrated to `release-please` for automated semantic versioning and changelog generation.
*   **docs:** Comprehensive documentation updates including new zero-allocation examples and benchmark results.
