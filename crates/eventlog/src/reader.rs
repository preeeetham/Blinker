use std::fs::{self, File};
use std::io::{BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::segment::{read_full, segment_path};
use crate::{LogError, Result, HEADER_LEN, MAX_RECORD_LEN};

/// A tailing cursor over the log.
///
/// `next()` returns `Ok(None)` at the current end of the log rather than
/// blocking; call it again after the writer commits more records. A partial
/// record at the end (writer mid-flush) is treated as "not yet written" and
/// re-read from its start next time.
pub struct LogReader {
    dir: PathBuf,
    base: u64,
    file: BufReader<File>,
    pos: u64,
    offset: u64,
    buf: Vec<u8>,
}

impl LogReader {
    pub(crate) fn open(dir: &Path, offset: u64) -> Result<Self> {
        let mut bases: Vec<u64> = fs::read_dir(dir)?
            .filter_map(|e| e.ok()?.file_name().into_string().ok()?.strip_suffix(".log")?.parse().ok())
            .collect();
        bases.sort_unstable();
        let first = *bases.first().unwrap_or(&0);
        if offset < first {
            return Err(LogError::Truncated(offset));
        }
        // Segment containing `offset`: last base <= offset.
        let base = *bases.iter().rev().find(|&&b| b <= offset).unwrap_or(&first);
        let file = BufReader::with_capacity(128 << 10, File::open(segment_path(dir, base))?);
        let mut r = Self { dir: dir.to_path_buf(), base, file, pos: 0, offset: base, buf: Vec::with_capacity(512) };
        // Skip forward to the requested offset.
        while r.offset < offset {
            if r.next()?.is_none() {
                break;
            }
        }
        Ok(r)
    }

    /// Offset of the next record this reader will return.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Like `next`, but never reads at or past `end` — used by consumers to
    /// stay behind the writer's committed (durable) offset, since buffered
    /// bytes can reach the file before their commit's fsync.
    pub fn next_before(&mut self, end: u64) -> Result<Option<(u64, &[u8])>> {
        if self.offset >= end {
            return Ok(None);
        }
        self.next()
    }

    /// Next record as `(offset, payload)`. The payload slice is valid until
    /// the next call — no per-record allocation on the replay path. (That
    /// borrow is why this is a "lending" method rather than `Iterator`.)
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<(u64, &[u8])>> {
        loop {
            let mut header = [0u8; HEADER_LEN];
            let start = self.pos;
            if read_full(&mut self.file, &mut header)? {
                let len = u32::from_le_bytes(header[..4].try_into().unwrap()) as usize;
                let crc = u32::from_le_bytes(header[4..].try_into().unwrap());
                if len > MAX_RECORD_LEN {
                    return Err(LogError::Corrupt { offset: self.offset, path: segment_path(&self.dir, self.base) });
                }
                self.buf.resize(len, 0);
                if read_full(&mut self.file, &mut self.buf)? {
                    if crc32fast::hash(&self.buf) != crc {
                        return Err(LogError::Corrupt { offset: self.offset, path: segment_path(&self.dir, self.base) });
                    }
                    self.pos = start + (HEADER_LEN + len) as u64;
                    let off = self.offset;
                    self.offset += 1;
                    return Ok(Some((off, &self.buf)));
                }
            }
            // End of this file (or a partial record). Rewind to the record
            // start so a later call re-reads it once it's complete.
            self.file.seek(SeekFrom::Start(start))?;
            self.pos = start;
            // If a newer segment starts exactly where we are, move to it.
            let next_path = segment_path(&self.dir, self.offset);
            if self.offset != self.base && next_path.exists() {
                self.file = BufReader::with_capacity(128 << 10, File::open(next_path)?);
                self.base = self.offset;
                self.pos = 0;
                continue;
            }
            return Ok(None);
        }
    }
}
