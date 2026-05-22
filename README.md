# ❄️ SnowID Rust

[![Crates.io](https://img.shields.io/crates/v/snowid.svg)](https://crates.io/crates/snowid)
[![Documentation](https://docs.rs/snowid/badge.svg)](https://docs.rs/snowid)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> A Rust implementation of a Snowflake-like ID generator with 42-bit timestamp.

**Generate 64-bit unique identifiers that are:**

- ⚡️ Fast (~22ns per ID with default logical generation, ~722-841ns for 1,024-ID batches)
- 📈 Time-sorted
- 🔄 Monotonic
- 🔒 Thread-safe
- 🌐 Distributed-ready
- 🎯 Minimal dependencies (`base62` for encoding)

## 🧮 ID Structure

**Example ID**: 151819733950271234

**Default configuration:**

```text
|------------------------------------------|------------|------------|
|           TIMESTAMP (42 bits)            | NODE (10)  |  SEQ (12)  |
|------------------------------------------|------------|------------|
```

- Timestamp: 42 bits = 139 years from 2024-01-01 (1704067200000)
- Node ID: 10 bits = 1,024 nodes (valid range: 6-16 bits)
- Sequence: 12 bits = 4,096 IDs/ms/node

## 🎯 Quick Start

```toml
[dependencies]
snowid = "3.0.0"
```

```rust
use snowid::SnowID;

fn main() {
    let gen = SnowID::new(1).unwrap();
    let id = gen.generate();
    println!("Generated ID: {}", id);
}
```

`generate()` uses logical timestamp generation by default: it always returns an ID and advances the timestamp component
instead of waiting when the current millisecond's sequence range is exhausted.

## 🔠 Base62 Encoded IDs

Generate base62 encoded IDs (using characters 0-9, a-z, A-Z) for more compact and URL-friendly identifiers:

```rust
use snowid::SnowID;

fn main() {
    // Create a generator
    let gen = SnowID::new(1).unwrap();

    // Generate a base62 encoded ID
    let encoded_id = gen.generate_base62();
    println!("Base62 ID: {}", encoded_id); // Example: "2qPfVQh7Jw9"

    // Generate with raw value
    let (encoded_id, raw_id) = gen.generate_base62_with_raw();
    println!("Base62: {}, Raw: {}", encoded_id, raw_id);

    // Decode a base62 ID back to u64
    let decoded = gen.decode_base62(&encoded_id).unwrap();
    assert_eq!(decoded, raw_id);

    // Extract components from a base62 ID
    let (timestamp, node, sequence) = gen.decompose_base62(&encoded_id).unwrap();
    println!("Timestamp: {}, Node: {}, Sequence: {}", timestamp, node, sequence);
}
```

### Benefits of Base62 IDs

- 🔤 More compact representation (11 chars max vs 20 digits for u64)
- 🔗 URL-friendly (no special characters)
- 👁️ Human-readable and easier to share
- 🔄 Directly encodes the same 64-bit SnowID value

## 🔧 Configuration

```rust
use snowid::{SnowID, SnowIDConfig};

fn main() {
    // Create custom configuration
    let config = SnowIDConfig::builder()
        .epoch(1577836800000) // 2020-01-01 00:00:00 UTC
        .node_bits(8).unwrap()         // Supports 255 nodes
        .build();

    // Create generator with custom config
    let gen = SnowID::with_config(1, config).unwrap();
}
```

Epochs are validated when creating a generator. SnowID rejects epochs in the future and epochs old enough that the
42-bit timestamp field cannot represent the current time, preventing silent timestamp wrap or duplicate ID collisions.

### ℹ️ Available Methods

```rust
use snowid::{SnowID, base62};

fn main() {
    let gen = SnowID::new(1).unwrap();

    // Generate numeric IDs
    let id = gen.generate();  // Default logical mode: never waits; timestamp can run ahead under overload
    let maybe_id = gen.try_generate();  // Non-blocking: returns Err when current millisecond is exhausted

    // Reserve many IDs with one atomic update for throughput-oriented hot paths
    let mut batch = [0u64; 128];
    gen.generate_batch(&mut batch);  // Always fills; timestamp can run ahead under overload
    let written = gen.try_generate_batch(&mut batch);  // May return fewer than batch.len() without waiting

    // Generate Base62 encoded IDs (allocates String)
    let base62_id = gen.generate_base62();
    let (base62_id, raw_id) = gen.generate_base62_with_raw();

    // Zero-allocation Base62 generation (for hot paths)
    let (bytes, len) = gen.generate_base62_array();  // Returns [u8; 11] + length
    let mut buf = [0u8; base62::MAX_LEN];
    let (str_ref, raw_id) = gen.generate_base62_into(&mut buf);  // Returns &str + raw ID

    // Decode Base62 IDs
    let decoded = gen.decode_base62(&base62_id).unwrap();
    let (ts, node, seq) = gen.decompose_base62(&base62_id).unwrap();

    // Extract individual components from numeric IDs
    let timestamp = gen.extract.timestamp(id);  // Get timestamp from ID
    let node = gen.extract.node(id);           // Get node ID from ID
    let sequence = gen.extract.sequence(id);    // Get sequence from ID

    // Extract all components at once
    let (ts, node, seq) = gen.extract.decompose(id);

    // Configuration information
    let max_node = gen.config.max_node_id();          // Get maximum allowed node ID
    let node_bits = gen.config.node_bits();           // Get number of bits used for node ID
    let max_seq = gen.config.max_sequence_id();    // Get maximum sequence per millisecond
    let timestamp_bits = SnowID::TIMESTAMP_BITS; // Get number of bits used for timestamp (42)
}
```

### ⏳ Logical Overflow Handling

Default `generate()` does not wait on per-millisecond sequence exhaustion; it advances a logical timestamp and returns
immediately. IDs stay unique and monotonic for the generator, but the timestamp component can run ahead of wall-clock
time during sustained overload.

For callers that must never advance logical time, use `try_generate()`. It returns immediately with an error when the
current millisecond has exhausted its sequence values or when the generator's logical timestamp is ahead of wall-clock.
For burst-heavy workloads, use
`try_generate_batch(&mut [u64])`; it reserves a contiguous sequence range with one atomic state update and returns the
number of IDs written without waiting for the next millisecond.

## 📊 Performance & Comparisons

### Social Media Platform Configurations

| Platform  | Timestamp | Node Bits | Sequence Bits | Max Nodes | IDs/ms/node before logical rollover | Default `generate()` Time/ID |
|-----------|-----------|-----------|---------------|-----------|--------------------------------------|------------------------------|
| Twitter   | 41        | 10        | 12            | 1,024     | 4,096                                | ~22ns                        |
| Instagram | 41        | 13        | 10            | 8,192     | 1,024                                | ~22ns                        |
| Discord   | 42        | 10        | 12            | 1,024     | 4,096                                | ~22ns                        |

### Node vs Sequence Bits Trade-off

| Node Bits | Max Nodes | IDs/ms/node before logical rollover | Default `generate()` Time/ID |
|-----------|-----------|--------------------------------------|------------------------------|
| 6         | 64        | 65,536                               | ~22ns                        |
| 8         | 256       | 16,384                               | ~22ns                        |
| 10        | 1,024     | 4,096                                | ~22ns                        |
| 12        | 4,096     | 1,024                                | ~22ns                        |
| 14        | 16,384    | 256                                  | ~22ns                        |
| 16        | 65,536    | 64                                   | ~24ns                        |

Choose configuration based on your needs:

- More nodes → Increase node bits (max 16 bits = 65,536 nodes)
- More IDs per node → Increase sequence bits (min 6 node bits = 64 nodes)
- Total bits (node + sequence) is fixed at 22 bits

For default logical generation, `node_bits` controls how many IDs fit in a real millisecond before the generator rolls
forward to a logical timestamp. It no longer forces the hot path to wait. On a local Apple Silicon benchmark run
(`cargo bench --bench perf_hotspots -- "Hotspot Generate Capacity/node_bits/<N>"`), `node_bits=6`, `node_bits=10`, and
`node_bits=16` all measured around ~22ns per ID. Use `try_generate()` when callers prefer an immediate non-blocking
failure over logical timestamp advancement.

Shared-generator contention is the other major throughput factor. A single shared generator is lock-free, but all
threads still update one atomic state. In the same local benchmark run, 8 threads sharing one generator for 1,024 IDs
each took about 2.64ms, while 8 per-thread generators took about 145µs. If your topology allows it, prefer one
generator per thread, worker, or shard with distinct node IDs for peak throughput.

For burst generation, `generate_batch(&mut [u64])` reserves a logical timestamp range with one atomic update and fills
the whole buffer. `try_generate_batch(&mut [u64])` is available when callers prefer partial non-blocking reservation.
On the same machine, filling 1,024 IDs with `generate_batch()` took about 721.62-840.74ns, while calling `generate()`
1,024 times took about 24.25µs. That is roughly a 32x throughput improvement for callers that can consume IDs in batches.

Focused hotspot benchmarks are available for validating your target machine without running the full benchmark suite:

```bash
cargo bench --bench perf_hotspots -- "Hotspot Generate Capacity/node_bits/10"
cargo bench --bench perf_hotspots -- "Hotspot Logical Generation/generate/node_bits/16/batch/1024"
cargo bench --bench perf_hotspots -- "Hotspot Batch Reservation/generate_batch/1024"
cargo bench --bench perf_hotspots -- "Hotspot Batch Reservation/try_generate_batch/1024"
cargo bench --bench perf_hotspots -- "Hotspot Batch Reservation/generate_loop/1024"
cargo bench --bench perf_hotspots -- "Hotspot Shared Generator/threads/8/ops_per_thread/1024"
cargo bench --bench perf_hotspots -- "Hotspot Per Thread Generator/threads/8/ops_per_thread/1024"
```

### Int64 vs Base62 Performance

| Variant          | Time/ID | Size         | Notes                        |
|------------------|---------|--------------|------------------------------|
| Int64            | ~22 ns  | 18-20 digits | Fastest single-ID option     |
| Base62 (String)  | ~40 ns  | 10-11 chars  | Compact, URL-friendly        |
| Base62 (array)   | ~24 ns  | 10-11 chars  | Zero-allocation, hot paths   |
| Base62 (into)    | ~24 ns  | 10-11 chars  | Zero-allocation, reuse buffer|

These Base62 numbers include ID generation plus encoding from the targeted `ID Generation Comparison` benchmark.

Base62 encoding provides more compact, URL-friendly IDs. For hot paths, use `generate_base62_array()` or `generate_base62_into()` to avoid heap allocations.

## 🚀 Examples

Check out [examples](examples/) for:

- Basic usage
- Custom configuration
- Base62 encoding and decoding
- Performance comparisons between Int64 and Base62
- Distributed generation
- Performance benchmarks

## 🧪 Correctness Stress Testing

The test suite includes deterministic unit tests, high-load multithread tests, and property-based tests for ID
uniqueness, monotonicity, sequence bounds, logical timestamp rollover, and batch generation. For Stryker-style mutation
testing in Rust, see [MUTATION_TESTING.md](MUTATION_TESTING.md) for `cargo-mutants` commands and release-gate guidance.

## 📜 License

MIT - See [LICENSE](LICENSE) for details
