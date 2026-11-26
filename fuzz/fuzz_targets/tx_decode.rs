//! Untrusted transaction bytes (from RPC or the relay endpoint) must never
//! panic or over-allocate, and anything that decodes as legacy must
//! re-serialize to the exact same bytes (canonical encoding).
#![no_main]
use blinker_solana::parse::{actions, resolve_keys};
use blinker_solana::{Transaction, VersionedMessage, VersionedTransaction};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(tx) = VersionedTransaction::decode(data) else { return };
    // Parsing instructions must be total over any decoded message.
    let keys = resolve_keys(&tx.message, &[], &[]);
    for _ in actions(tx.message.inner(), &keys) {}
    if let VersionedMessage::Legacy(m) = tx.message {
        let re = Transaction { signatures: tx.signatures, message: m }.serialize();
        assert_eq!(re, data, "legacy decode/encode is not canonical");
    }
});
