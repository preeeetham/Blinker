//! Append throughput vs. group-commit batch size and fsync policy.
//! The interesting number is how batch size amortises the fsync cost.
use blinker_eventlog::{Config, Log, SyncPolicy};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

const RECORD: [u8; 64] = [0xab; 64]; // ~size of an encoded analytics event

fn append(c: &mut Criterion) {
    let mut g = c.benchmark_group("append");
    for (policy_name, sync) in [("no_fsync", SyncPolicy::Never), ("fsync_per_commit", SyncPolicy::EveryCommit)] {
        for batch in [1usize, 16, 256] {
            let dir = tempfile::tempdir().unwrap();
            let mut log = Log::open(dir.path(), Config { segment_bytes: 64 << 20, sync, retention_bytes: Some(256 << 20) }).unwrap();
            g.throughput(Throughput::Elements(batch as u64));
            g.bench_with_input(BenchmarkId::new(policy_name, batch), &batch, |b, &batch| {
                b.iter(|| {
                    for _ in 0..batch {
                        log.append(black_box(&RECORD)).unwrap();
                    }
                    log.commit().unwrap()
                })
            });
        }
    }
    g.finish();
}

fn replay(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(dir.path(), Config { segment_bytes: 8 << 20, sync: SyncPolicy::Never, retention_bytes: None }).unwrap();
    const N: u64 = 100_000;
    for _ in 0..N {
        log.append(&RECORD).unwrap();
    }
    log.commit().unwrap();
    let mut g = c.benchmark_group("replay");
    g.throughput(Throughput::Elements(N));
    g.sample_size(20);
    g.bench_function("100k_records", |b| {
        b.iter(|| {
            let mut r = log.reader(0).unwrap();
            let mut sum = 0u64;
            while let Some((_, p)) = r.next().unwrap() {
                sum += p.len() as u64;
            }
            black_box(sum)
        })
    });
    g.finish();
}

criterion_group!(benches, append, replay);
criterion_main!(benches);
