use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DecodeError {
    #[error("unexpected end of input at byte {0}")]
    Eof(usize),
    #[error("compact-u16 is malformed or not minimally encoded")]
    BadShortVec,
    #[error("unsupported message version {0}")]
    UnsupportedVersion(u8),
    #[error("account index {0} out of range")]
    AccountIndex(u8),
    #[error("{0} trailing bytes after transaction")]
    Trailing(usize),
    #[error("transaction exceeds {max} bytes")]
    TooLarge { max: usize },
    #[error("invalid header: {0}")]
    Header(&'static str),
    #[error("invalid base58")]
    Base58,
    #[error("invalid base64")]
    Base64,
    #[error("expected {expected} bytes, got {got}")]
    Length { expected: usize, got: usize },
}

/// Bounds-checked cursor over a byte slice. Every read returns `Eof` instead
/// of panicking, which is what keeps the decoder fuzz-clean.
pub(crate) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn u8(&mut self) -> Result<u8, DecodeError> {
        let b = *self.buf.get(self.pos).ok_or(DecodeError::Eof(self.pos))?;
        self.pos += 1;
        Ok(b)
    }

    pub fn peek(&self) -> Option<u8> {
        self.buf.get(self.pos).copied()
    }

    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let end = self.pos.checked_add(n).ok_or(DecodeError::Eof(self.pos))?;
        let s = self.buf.get(self.pos..end).ok_or(DecodeError::Eof(self.pos))?;
        self.pos = end;
        Ok(s)
    }

    pub fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let s = self.bytes(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(s);
        Ok(out)
    }

    pub fn short_len(&mut self) -> Result<usize, DecodeError> {
        let (len, used) = crate::shortvec::decode(&self.buf[self.pos..])?;
        self.pos += used;
        Ok(len as usize)
    }

    /// Reads a compact-u16 length, then checks that `len * elem_size` bytes
    /// are actually present before the caller allocates anything.
    pub fn short_len_checked(&mut self, elem_size: usize) -> Result<usize, DecodeError> {
        let len = self.short_len()?;
        if len.saturating_mul(elem_size) > self.remaining() {
            return Err(DecodeError::Eof(self.pos));
        }
        Ok(len)
    }
}
