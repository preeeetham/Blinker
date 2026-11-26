//! An append-only, segmented commit log — the "broker" at the centre of
//! Blinker's event pipeline.
//!
//! ```text
//! data/events/
//!   00000000000000000000.log   records [0, 81_234)
//!   00000000000000081234.log   records [81_234, ...)   <- active segment
//! ```
//!
//! Each record is framed as `len:u32le | crc32:u32le | payload`. Offsets are
//! logical record numbers, so a segment's file name is the offset of its
//! first record and readers can seek without an index file.
//!
//! Durability: the writer buffers appends and makes them durable in
//! `commit()`, which flushes and `fdatasync`s once for the whole batch
//! (group commit). On open, the active segment is scanned and any torn tail
//! — a partial or CRC-failing record left by a crash mid-write — is
//! truncated away, so the log always ends on a record boundary.

mod checkpoint;
mod reader;
mod segment;

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub use checkpoint::Checkpoint;
pub use reader::LogReader;
use segment::{scan_segment, segment_path, SegmentMeta};

/// `len` + `crc`.
pub const HEADER_LEN: usize = 8;
/// Upper bound on one record; protects readers from absurd length fields.
pub const MAX_RECORD_LEN: usize = 1 << 20;

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("record of {0} bytes exceeds MAX_RECORD_LEN")]
    RecordTooLarge(usize),
    #[error("corrupt record at offset {offset} in {path}")]
    Corrupt { offset: u64, path: PathBuf },
    #[error("offset {0} is before the oldest retained record")]
    Truncated(u64),
}

pub type Result<T> = std::result::Result<T, LogError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPolicy {
    /// `fdatasync` on every `commit()` — no acknowledged write is ever lost.
    EveryCommit,
    /// Flush to the page cache on commit, `fdatasync` at most this often.
    /// Trades a bounded loss window for far fewer disk flushes.
    Interval(Duration),
    /// Never fsync (tests, benches).
    Never,
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Roll to a new segment once the active one reaches this size.
    pub segment_bytes: u64,
    pub sync: SyncPolicy,
    /// Delete the oldest segments once the total exceeds this. `None` keeps all.
    pub retention_bytes: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self { segment_bytes: 64 << 20, sync: SyncPolicy::EveryCommit, retention_bytes: Some(1 << 30) }
    }
}

/// Counters exposed on `/metrics`.
#[derive(Debug, Clone, Default)]
pub struct LogStats {
    pub segments: usize,
    pub bytes: u64,
    pub first_offset: u64,
    pub next_offset: u64,
    pub commits: u64,
    pub fsyncs: u64,
    pub last_fsync_micros: u64,
    pub recovered_truncated_bytes: u64,
}

pub struct Log {
    dir: PathBuf,
    cfg: Config,
    segments: Vec<SegmentMeta>,
    active: BufWriter<File>,
    active_len: u64,
    next_offset: u64,
    last_sync: Instant,
    dirty: bool,
    stats: LogStats,
}

impl Log {
    pub fn open(dir: impl AsRef<Path>, cfg: Config) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;

        let mut bases: Vec<u64> = fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().into_string().ok()?;
                name.strip_suffix(".log")?.parse::<u64>().ok()
            })
            .collect();
        bases.sort_unstable();

        let mut segments: Vec<SegmentMeta> = Vec::with_capacity(bases.len().max(1));
        for (i, &base) in bases.iter().enumerate() {
            let path = segment_path(&dir, base);
            let len = fs::metadata(&path)?.len();
            // Sealed segments' record counts come from the next segment's base.
            let next = bases.get(i + 1).copied();
            segments.push(SegmentMeta { base, len, records: next.map(|n| n - base) });
        }

        let mut recovered = 0;
        if segments.is_empty() {
            File::create(segment_path(&dir, 0))?;
            segments.push(SegmentMeta { base: 0, len: 0, records: None });
        }

        // Recover the active (last) segment: count records, cut torn tail.
        let last = segments.last_mut().unwrap();
        let path = segment_path(&dir, last.base);
        let scan = scan_segment(&path)?;
        if scan.valid_len < last.len {
            recovered = last.len - scan.valid_len;
            let f = OpenOptions::new().write(true).open(&path)?;
            f.set_len(scan.valid_len)?;
            f.sync_all()?;
            last.len = scan.valid_len;
        }
        let next_offset = last.base + scan.records;

        let file = OpenOptions::new().append(true).open(&path)?;
        let active_len = last.len;
        let mut log = Self {
            dir,
            cfg,
            segments,
            active: BufWriter::with_capacity(256 << 10, file),
            active_len,
            next_offset,
            last_sync: Instant::now(),
            dirty: false,
            stats: LogStats { recovered_truncated_bytes: recovered, ..Default::default() },
        };
        log.refresh_stats();
        Ok(log)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Offset the next appended record will get.
    pub fn next_offset(&self) -> u64 {
        self.next_offset
    }

    pub fn first_offset(&self) -> u64 {
        self.segments.first().map(|s| s.base).unwrap_or(0)
    }

    /// Buffers one record. Not durable — nor visible to readers — until
    /// `commit()`. Returns the record's offset.
    pub fn append(&mut self, payload: &[u8]) -> Result<u64> {
        if payload.len() > MAX_RECORD_LEN {
            return Err(LogError::RecordTooLarge(payload.len()));
        }
        if self.active_len >= self.cfg.segment_bytes {
            self.roll()?;
        }
        let mut header = [0u8; HEADER_LEN];
        header[..4].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        header[4..].copy_from_slice(&crc32fast::hash(payload).to_le_bytes());
        self.active.write_all(&header)?;
        self.active.write_all(payload)?;
        self.active_len += (HEADER_LEN + payload.len()) as u64;
        let off = self.next_offset;
        self.next_offset += 1;
        self.dirty = true;
        Ok(off)
    }

    /// Group commit: flush the buffer and (per policy) fsync once for every
    /// record appended since the last commit. Returns the committed end offset.
    pub fn commit(&mut self) -> Result<u64> {
        if !self.dirty {
            return Ok(self.next_offset);
        }
        self.active.flush()?;
        let should_sync = match self.cfg.sync {
            SyncPolicy::EveryCommit => true,
            SyncPolicy::Interval(d) => self.last_sync.elapsed() >= d,
            SyncPolicy::Never => false,
        };
        if should_sync {
            let t = Instant::now();
            // fdatasync: skips the metadata-only flush fsync would do.
            self.active.get_ref().sync_data()?;
            self.stats.fsyncs += 1;
            self.stats.last_fsync_micros = t.elapsed().as_micros() as u64;
            self.last_sync = Instant::now();
        }
        self.dirty = false;
        self.stats.commits += 1;
        self.refresh_stats();
        Ok(self.next_offset)
    }

    fn roll(&mut self) -> Result<()> {
        self.active.flush()?;
        self.active.get_ref().sync_data()?;
        let last = self.segments.last_mut().unwrap();
        last.len = self.active_len;
        last.records = Some(self.next_offset - last.base);

        let base = self.next_offset;
        let path = segment_path(&self.dir, base);
        let file = OpenOptions::new().create_new(true).append(true).open(&path)?;
        // Make the new directory entry durable before writing into it.
        sync_dir(&self.dir)?;
        self.active = BufWriter::with_capacity(256 << 10, file);
        self.active_len = 0;
        self.segments.push(SegmentMeta { base, len: 0, records: None });
        self.enforce_retention()?;
        Ok(())
    }

    fn enforce_retention(&mut self) -> Result<()> {
        let Some(limit) = self.cfg.retention_bytes else { return Ok(()) };
        let mut total: u64 = self.segments.iter().map(|s| s.len).sum::<u64>() + self.active_len;
        // Never delete the active segment.
        while total > limit && self.segments.len() > 1 {
            let old = self.segments.remove(0);
            fs::remove_file(segment_path(&self.dir, old.base))?;
            total -= old.len;
        }
        Ok(())
    }

    fn refresh_stats(&mut self) {
        let sealed: u64 = self.segments[..self.segments.len() - 1].iter().map(|s| s.len).sum();
        self.stats.segments = self.segments.len();
        self.stats.bytes = sealed + self.active_len;
        self.stats.first_offset = self.first_offset();
        self.stats.next_offset = self.next_offset;
    }

    pub fn stats(&self) -> LogStats {
        self.stats.clone()
    }

    /// A reader starting at `offset`. Readers use their own file handles,
    /// so they can run on other threads while the writer appends. Bound reads
    /// with `LogReader::next_before(committed)` to only observe durable records.
    pub fn reader(&self, offset: u64) -> Result<LogReader> {
        LogReader::open(&self.dir, offset)
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        let _ = self.active.flush();
        let _ = self.active.get_ref().sync_data();
    }
}

/// fsync the directory so a newly created/renamed file survives a crash.
pub(crate) fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
