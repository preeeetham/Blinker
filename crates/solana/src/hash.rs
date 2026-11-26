use std::fmt;
use std::str::FromStr;

use crate::error::DecodeError;

/// A 32-byte blockhash.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Hash(pub [u8; 32]);

impl FromStr for Hash {
    type Err = DecodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let k: crate::Pubkey = s.parse()?;
        Ok(Hash(k.0))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&bs58::encode(self.0).into_string())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({self})")
    }
}
