//! Analytics ingest throughput: decode + apply, the consumer's hot loop.
use blinker_analytics::{Asset, BlinkId, Engine, Event, EventKind};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;

fn events(n: usize, blinks: usize) -> Vec<Vec<u8>> {
    let ids: Vec<BlinkId> = (0..blinks).map(|i| BlinkId::from_random(((i as u128) * 0x9e3779b97f4a7c15).to_le_bytes()[..12].try_into().unwrap())).collect();
    let mut out: Vec<Vec<u8>> = ids.iter().map(|&b| Event { ts_ms: 0, blink: b, kind: EventKind::Created { asset: Asset::Sol } }.to_vec()).collect();
    for i in 0..n {
        let blink = ids[i % blinks];
        let kind = match i % 10 {
            0 => EventKind::Paid { amount: 1_000_000, payer: [(i % 251) as u8; 32] },
            1..=3 => EventKind::ActionPost { amount: 1_000_000 },
            _ => EventKind::View,
        };
        out.push(Event { ts_ms: 1_764_000_000_000 + i as u64, blink, kind }.to_vec());
    }
    out
}

fn ingest(c: &mut Criterion) {
    let recs = events(100_000, 1_000);
    let mut g = c.benchmark_group("analytics");
    g.throughput(Throughput::Elements(recs.len() as u64));
    g.sample_size(30);
    g.bench_function("decode+apply_100k_events_1k_blinks", |b| {
        b.iter(|| {
            let mut e = Engine::new();
            for r in &recs {
                e.apply_bytes(black_box(r)).unwrap();
            }
            black_box(e.events_applied())
        })
    });
    g.finish();

    let one = recs[1500].clone();
    c.bench_function("event_decode", |b| b.iter(|| Event::decode(black_box(&one)).unwrap()));
}

criterion_group!(benches, ingest);
criterion_main!(benches);
