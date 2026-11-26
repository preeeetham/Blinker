use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

use crate::{HEADER_LEN, MAX_RECORD_LEN};

#[derive(Debug, Clone)]
pub(crate) struct SegmentMeta {
    pub base: u64,
    pub len: u64,
    /// Known for sealed segments; `None` for the active one.
    #[allow(dead_code)]
    pub records: Option<u64>,
}

pub(crate) fn segment_path(dir: &Path, base: u64) -> PathBuf {
    dir.join(format!("{base:020}.log"))
}

pub(crate) struct Scan {
    pub records: u64,
    /// Byte length up to the end of the last intact record.
    pub valid_len: u64,
}

/// Walks a segment validating every frame. Stops at the first short read,
/// oversized length or CRC mismatch — everything after that is a torn tail.
pub(crate) fn scan_segment(path: &Path) -> io::Result<Scan> {
    let mut r = BufReader::with_capacity(256 << 10, File::open(path)?);
    let mut header = [0u8; HEADER_LEN];
    let mut payload = Vec::with_capacity(512);
    let mut scan = Scan { records: 0, valid_len: 0 };
    loop {
        if !read_full(&mut r, &mut header)? {
            return Ok(scan);
        }
        let len = u32::from_le_bytes(header[..4].try_into().unwrap()) as usize;
        let crc = u32::from_le_bytes(header[4..].try_into().unwrap());
        if len > MAX_RECORD_LEN {
            return Ok(scan);
        }
        payload.resize(len, 0);
        if !read_full(&mut r, &mut payload)? || crc32fast::hash(&payload) != crc {
            return Ok(scan);
        }
        scan.records += 1;
        scan.valid_len += (HEADER_LEN + len) as u64;
    }
}

/// `read_exact` that reports a clean or partial EOF as `Ok(false)`.
pub(crate) fn read_full(r: &mut impl Read, buf: &mut [u8]) -> io::Result<bool> {
    let mut filled = 0;
    while filled < buf.len() {
        match r.read(&mut buf[filled..]) {
            Ok(0) => return Ok(false),
            Ok(n) => filled += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(true)
}
