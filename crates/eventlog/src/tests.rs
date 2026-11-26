use super::*;

fn cfg(segment_bytes: u64) -> Config {
    Config { segment_bytes, sync: SyncPolicy::Never, retention_bytes: None }
}

fn collect(log: &Log, from: u64) -> Vec<(u64, Vec<u8>)> {
    let mut r = log.reader(from).unwrap();
    let mut out = vec![];
    while let Some((o, p)) = r.next().unwrap() {
        out.push((o, p.to_vec()));
    }
    out
}

#[test]
fn append_commit_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(dir.path(), cfg(1 << 20)).unwrap();
    for i in 0..100u32 {
        assert_eq!(log.append(&i.to_le_bytes()).unwrap(), i as u64);
    }
    assert_eq!(log.commit().unwrap(), 100);
    let recs = collect(&log, 0);
    assert_eq!(recs.len(), 100);
    assert_eq!(recs[42], (42, 42u32.to_le_bytes().to_vec()));
    assert_eq!(collect(&log, 90).len(), 10);
}

#[test]
fn rolls_segments_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut log = Log::open(dir.path(), cfg(256)).unwrap();
        for i in 0..200u32 {
            log.append(&[i as u8; 20]).unwrap();
        }
        log.commit().unwrap();
        assert!(log.stats().segments > 5);
    }
    let log = Log::open(dir.path(), cfg(256)).unwrap();
    assert_eq!(log.next_offset(), 200);
    let recs = collect(&log, 0);
    assert_eq!(recs.len(), 200);
    assert!(recs.iter().enumerate().all(|(i, (o, p))| *o == i as u64 && p[0] == i as u8));
    // Seek into the middle of a later segment.
    assert_eq!(collect(&log, 150)[0].0, 150);
}

#[test]
fn recovers_from_torn_write() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut log = Log::open(dir.path(), cfg(1 << 20)).unwrap();
        for i in 0..10u8 {
            log.append(&[i; 32]).unwrap();
        }
        log.commit().unwrap();
    }
    // Simulate a crash mid-write: a header promising 32 bytes, then 5 bytes.
    let path = segment::segment_path(dir.path(), 0);
    let mut f = OpenOptions::new().append(true).open(&path).unwrap();
    f.write_all(&32u32.to_le_bytes()).unwrap();
    f.write_all(&0u32.to_le_bytes()).unwrap();
    f.write_all(&[1, 2, 3, 4, 5]).unwrap();
    drop(f);

    let mut log = Log::open(dir.path(), cfg(1 << 20)).unwrap();
    assert_eq!(log.next_offset(), 10);
    assert_eq!(log.stats().recovered_truncated_bytes, 13);
    // New appends land cleanly after the cut.
    log.append(b"after").unwrap();
    log.commit().unwrap();
    let recs = collect(&log, 0);
    assert_eq!(recs.len(), 11);
    assert_eq!(recs[10].1, b"after");
}

#[test]
fn detects_bit_flip_as_torn_tail() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut log = Log::open(dir.path(), cfg(1 << 20)).unwrap();
        for i in 0..5u8 {
            log.append(&[i; 16]).unwrap();
        }
        log.commit().unwrap();
    }
    let path = segment::segment_path(dir.path(), 0);
    let mut bytes = fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff; // corrupt final record's payload
    fs::write(&path, bytes).unwrap();
    let log = Log::open(dir.path(), cfg(1 << 20)).unwrap();
    assert_eq!(log.next_offset(), 4);
}

#[test]
fn reader_tails_new_commits() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(dir.path(), cfg(128)).unwrap();
    let mut r = log.reader(0).unwrap();
    assert!(r.next().unwrap().is_none());
    for round in 0..5u8 {
        for _ in 0..10 {
            log.append(&[round; 24]).unwrap();
        }
        log.commit().unwrap();
        let mut n = 0;
        while let Some((_, p)) = r.next().unwrap() {
            assert_eq!(p[0], round);
            n += 1;
        }
        assert_eq!(n, 10, "round {round}");
    }
    assert_eq!(r.offset(), 50);
}

#[test]
fn retention_drops_oldest_segments() {
    let dir = tempfile::tempdir().unwrap();
    let mut log =
        Log::open(dir.path(), Config { segment_bytes: 100, sync: SyncPolicy::Never, retention_bytes: Some(300) })
            .unwrap();
    for _ in 0..100 {
        log.append(&[0; 42]).unwrap();
    }
    log.commit().unwrap();
    assert!(log.stats().bytes <= 400);
    assert!(log.first_offset() > 0);
    assert!(matches!(log.reader(0), Err(LogError::Truncated(0))));
    assert!(!collect(&log, log.first_offset()).is_empty());
}

#[test]
fn checkpoint_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let cp = Checkpoint::new(dir.path(), "analytics");
    assert_eq!(cp.load().unwrap(), None);
    cp.store(12345).unwrap();
    assert_eq!(cp.load().unwrap(), Some(12345));
}
