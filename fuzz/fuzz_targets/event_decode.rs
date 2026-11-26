//! Event records come off disk; a corrupt or hostile record must be
//! rejected cleanly, and every accepted record must round-trip exactly.
#![no_main]
use blinker_analytics::{Engine, Event};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(e) = Event::decode(data) {
        assert_eq!(e.to_vec(), data);
    }
    // The engine must survive any byte sequence.
    let mut eng = Engine::new();
    for chunk in data.chunks(23) {
        let _ = eng.apply_bytes(chunk);
    }
    let _ = eng.global(0);
});
