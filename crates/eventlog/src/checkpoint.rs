use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// A consumer's committed read position, stored crash-safely with the
/// write-temp → fsync → rename → fsync-dir pattern, so a crash leaves either
/// the old or the new value, never a half-written file.
pub struct Checkpoint {
    path: PathBuf,
}

impl Checkpoint {
    pub fn new(dir: impl AsRef<Path>, consumer: &str) -> Self {
        Self { path: dir.as_ref().join(format!("{consumer}.offset")) }
    }

    pub fn load(&self) -> std::io::Result<Option<u64>> {
        match fs::read_to_string(&self.path) {
            Ok(s) => Ok(s.trim().parse().ok()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn store(&self, offset: u64) -> std::io::Result<()> {
        let tmp = self.path.with_extension("offset.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            write!(f, "{offset}")?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &self.path)?;
        if let Some(dir) = self.path.parent() {
            crate::sync_dir(dir)?;
        }
        Ok(())
    }
}
