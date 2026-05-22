#![allow(clippy::unwrap_used, clippy::panic, clippy::cast_possible_truncation)]

use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use snowid::{SnowID, SnowIDConfig, base62};

fn criterion_config() -> Criterion {
    Criterion::default().sample_size(20).measurement_time(Duration::from_secs(2))
}

fn generate_batch(generator: &SnowID, iterations: usize) -> u64 {
    let mut last = 0u64;
    for _ in 0..iterations {
        last = generator.generate();
    }
    last
}

fn generate_unbounded_batch(generator: &SnowID, iterations: usize) -> u64 {
    let mut last = 0u64;
    for _ in 0..iterations {
        last = generator.generate_unbounded();
    }
    last
}

fn reserve_batch(generator: &SnowID, ids: &mut [u64]) -> u64 {
    let written = generator.try_generate_batch(ids);
    ids.iter().take(written).fold(written as u64, |checksum, id| checksum ^ id)
}

fn fill_batch(generator: &SnowID, ids: &mut [u64]) -> u64 {
    generator.generate_batch(ids);
    ids.iter().fold(ids.len() as u64, |checksum, id| checksum ^ id)
}

fn join_checksum(handles: Vec<thread::JoinHandle<u64>>) -> u64 {
    let mut checksum = 0u64;
    for handle in handles {
        checksum ^= handle.join().unwrap();
    }
    checksum
}

fn shared_generator_batch(generator: Arc<SnowID>, thread_count: usize) -> u64 {
    let mut handles = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        let generator = Arc::clone(&generator);
        handles.push(thread::spawn(move || generate_batch(&generator, 1024)));
    }
    join_checksum(handles)
}

fn per_thread_generator_batch(thread_count: usize) -> u64 {
    let mut handles = Vec::with_capacity(thread_count);
    for node_id in 0..thread_count {
        handles.push(thread::spawn(move || {
            let generator = SnowID::new(node_id as u16).unwrap();
            generate_batch(&generator, 1024)
        }));
    }
    join_checksum(handles)
}

pub fn time_source_cost(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Time Source");

    group.bench_function("system_time_unix_ms", |b| {
        b.iter(|| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            black_box(now)
        });
    });

    group.bench_function("duration_since_custom_epoch", |b| {
        let epoch = SnowIDConfig::default().epoch();
        b.iter(|| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            black_box(now - epoch)
        });
    });

    group.finish();
}

pub fn generation_by_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Generate Capacity");

    for node_bits in [6, 10, 16] {
        group.bench_function(format!("node_bits/{node_bits}"), |b| {
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().build();
            let generator = SnowID::with_config(1, config).unwrap();
            b.iter(|| black_box(generator.generate()));
        });
    }

    group.finish();
}

pub fn generation_burst_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Generate Burst Capacity");

    for node_bits in [6, 10, 16] {
        group.bench_function(format!("node_bits/{node_bits}/batch/1024"), |b| {
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().build();
            let generator = SnowID::with_config(1, config).unwrap();
            b.iter(|| black_box(generate_batch(&generator, 1024)));
        });
    }

    group.finish();
}

pub fn unbounded_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Unbounded Generation");

    for node_bits in [10, 16] {
        group.bench_function(format!("generate_unbounded/node_bits/{node_bits}/batch/1024"), |b| {
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().build();
            let generator = SnowID::with_config(1, config).unwrap();
            b.iter(|| black_box(generate_unbounded_batch(&generator, 1024)));
        });
    }

    group.finish();
}

pub fn batch_reservation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Batch Reservation");

    for batch in [64usize, 256, 1024] {
        group.bench_function(format!("try_generate_batch/{batch}"), |b| {
            b.iter_batched(
                || {
                    let config = SnowIDConfig::builder().node_bits(6).unwrap().build();
                    (SnowID::with_config(1, config).unwrap(), vec![0u64; batch])
                },
                |(generator, mut ids)| black_box(reserve_batch(&generator, black_box(&mut ids))),
                BatchSize::SmallInput,
            );
        });

        group.bench_function(format!("generate_batch/{batch}"), |b| {
            b.iter_batched(
                || {
                    let config = SnowIDConfig::builder().node_bits(6).unwrap().build();
                    (SnowID::with_config(1, config).unwrap(), vec![0u64; batch])
                },
                |(generator, mut ids)| black_box(fill_batch(&generator, black_box(&mut ids))),
                BatchSize::SmallInput,
            );
        });

        group.bench_function(format!("generate_loop/{batch}"), |b| {
            b.iter_batched(
                || {
                    let config = SnowIDConfig::builder().node_bits(6).unwrap().build();
                    SnowID::with_config(1, config).unwrap()
                },
                |generator| black_box(generate_batch(&generator, batch)),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

pub fn generator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Generator Creation");

    group.bench_function("default_new", |b| {
        b.iter(|| black_box(SnowID::new(black_box(1)).unwrap()));
    });

    group.bench_function("with_default_config", |b| {
        let config = SnowIDConfig::default();
        b.iter(|| black_box(SnowID::with_config(black_box(1), config).unwrap()));
    });

    group.finish();
}

pub fn shared_generator_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Shared Generator");

    for thread_count in [2usize, 4, 8] {
        group.bench_function(format!("threads/{thread_count}/ops_per_thread/1024"), |b| {
            b.iter_batched(
                || Arc::new(SnowID::new(1).unwrap()),
                |generator| black_box(shared_generator_batch(generator, thread_count)),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

pub fn per_thread_generator(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Per Thread Generator");

    for thread_count in [2usize, 4, 8] {
        group.bench_function(format!("threads/{thread_count}/ops_per_thread/1024"), |b| {
            b.iter(|| black_box(per_thread_generator_batch(thread_count)));
        });
    }

    group.finish();
}

pub fn extraction_shapes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Extraction");
    let generator = SnowID::new(1).unwrap();
    let id = generator.generate();

    group.bench_function("timestamp_only", |b| {
        b.iter(|| black_box(generator.extract.timestamp(black_box(id))));
    });

    group.bench_function("node_only", |b| {
        b.iter(|| black_box(generator.extract.node(black_box(id))));
    });

    group.bench_function("sequence_only", |b| {
        b.iter(|| black_box(generator.extract.sequence(black_box(id))));
    });

    group.bench_function("three_individual_calls", |b| {
        b.iter(|| {
            let id = black_box(id);
            black_box((generator.extract.timestamp(id), generator.extract.node(id), generator.extract.sequence(id)))
        });
    });

    group.bench_function("decompose", |b| {
        b.iter(|| black_box(generator.extract.decompose(black_box(id))));
    });

    group.finish();
}

pub fn base62_buffer_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hotspot Base62 Boundary");
    let generator = SnowID::new(1).unwrap();

    group.bench_function("into_fresh_stack_buffer", |b| {
        b.iter(|| {
            let mut buf = [0u8; base62::MAX_LEN];
            let (_, raw) = generator.generate_base62_into(&mut buf);
            black_box(raw)
        });
    });

    group.bench_function("into_reused_buffer", |b| {
        let mut buf = [0u8; base62::MAX_LEN];
        b.iter(|| {
            let (_, raw) = generator.generate_base62_into(&mut buf);
            black_box(raw)
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets =
        time_source_cost,
        generation_by_capacity,
        generation_burst_capacity,
        unbounded_generation,
        batch_reservation,
        generator_creation,
        shared_generator_contention,
        per_thread_generator,
        extraction_shapes,
        base62_buffer_reuse
}
criterion_main!(benches);
