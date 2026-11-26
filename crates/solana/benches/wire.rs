//! Wire-format hot paths: building the unsigned tx returned by every action
//! POST, and decoding fetched transactions in the payment verifier.
use blinker_solana::transaction::VersionedTransaction;
use blinker_solana::{programs, Hash, Message, Pubkey, Transaction};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

fn build_sol(payer: &Pubkey, to: &Pubkey) -> Transaction {
    let ixs = [programs::memo("blinker:v1:abcdefghijkl"), programs::system_transfer(payer, to, 1_000_000)];
    Transaction::new_unsigned(Message::new(&ixs, payer, Hash([7; 32])))
}

fn build_spl(payer: &Pubkey, to: &Pubkey, mint: &Pubkey) -> Transaction {
    let tp = programs::TOKEN_PROGRAM_ID;
    let src = programs::associated_token_address(payer, mint, &tp);
    let dst = programs::associated_token_address(to, mint, &tp);
    let ixs = [
        programs::create_ata_idempotent(payer, to, mint, &tp),
        programs::token_transfer_checked(&tp, &src, mint, &dst, payer, 1_500_000, 6),
        programs::memo("blinker:v1:abcdefghijkl"),
    ];
    Transaction::new_unsigned(Message::new(&ixs, payer, Hash([7; 32])))
}

fn benches(c: &mut Criterion) {
    let payer: Pubkey = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse().unwrap();
    let to: Pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".parse().unwrap();
    let mint: Pubkey = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".parse().unwrap();

    let mut g = c.benchmark_group("build");
    g.bench_function("sol_transfer+memo", |b| b.iter(|| black_box(build_sol(black_box(&payer), &to)).to_base64()));
    g.bench_function("spl_transfer+ata+memo", |b| b.iter(|| black_box(build_spl(black_box(&payer), &to, &mint)).to_base64()));
    // What the server actually does: recipient ATA cached, payer ATA derived.
    let tp = programs::TOKEN_PROGRAM_ID;
    let dst = programs::associated_token_address(&to, &mint, &tp);
    g.bench_function("spl_transfer+ata+memo (cached dst ATA)", |b| {
        b.iter(|| {
            let payer = black_box(&payer);
            let src = programs::associated_token_address(payer, &mint, &tp);
            let ixs = [
                programs::create_ata_idempotent_with(payer, &to, &dst, &mint, &tp),
                programs::token_transfer_checked(&tp, &src, &mint, &dst, payer, 1_500_000, 6),
                programs::memo("blinker:v1:abcdefghijkl"),
            ];
            Transaction::new_unsigned(Message::new(&ixs, payer, Hash([7; 32]))).to_base64()
        })
    });
    g.bench_function("find_ata_pda", |b| {
        b.iter(|| programs::associated_token_address(black_box(&payer), &mint, &programs::TOKEN_PROGRAM_ID))
    });
    g.finish();

    let mut g = c.benchmark_group("decode");
    for (name, tx) in [("sol", build_sol(&payer, &to)), ("spl", build_spl(&payer, &to, &mint))] {
        let bytes = tx.serialize();
        g.throughput(Throughput::Bytes(bytes.len() as u64));
        g.bench_with_input(BenchmarkId::new("versioned_tx", name), &bytes, |b, bytes| {
            b.iter(|| VersionedTransaction::decode(black_box(bytes)).unwrap())
        });
    }
    g.finish();
}

criterion_group!(wire, benches);
criterion_main!(wire);
