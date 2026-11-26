use std::fmt;
use std::str::FromStr;

use curve25519_dalek::edwards::CompressedEdwardsY;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::error::DecodeError;

/// A 32-byte account address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Pubkey(pub [u8; 32]);

const PDA_MARKER: &[u8] = b"ProgramDerivedAddress";
pub const MAX_SEED_LEN: usize = 32;
pub const MAX_SEEDS: usize = 16;

impl Pubkey {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Parses a base58 address at compile time. Panics (at compile time when
    /// used in a `const`) on invalid input, so program IDs are checked once.
    pub const fn from_str_const(s: &str) -> Self {
        Self(match bs58::decode(s.as_bytes()).into_array_const::<32>() {
            Ok(a) => a,
            Err(_) => panic!("invalid base58 pubkey"),
        })
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.0).into_string()
    }

    /// True when the bytes decode to a point on the ed25519 curve, i.e. the
    /// address could have a private key. PDAs must be off-curve.
    pub fn is_on_curve(&self) -> bool {
        CompressedEdwardsY(self.0).decompress().is_some()
    }

    /// `create_program_address`: hash the seeds; fail if the result is on-curve.
    pub fn create_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Option<Pubkey> {
        if seeds.len() > MAX_SEEDS || seeds.iter().any(|s| s.len() > MAX_SEED_LEN) {
            return None;
        }
        let mut h = Sha256::new();
        for s in seeds {
            h.update(s);
        }
        h.update(program_id.0);
        h.update(PDA_MARKER);
        let candidate = Pubkey(h.finalize().into());
        (!candidate.is_on_curve()).then_some(candidate)
    }

    /// `find_program_address`: search bump seeds from 255 down for the first
    /// off-curve address.
    pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Option<(Pubkey, u8)> {
        if seeds.len() >= MAX_SEEDS {
            return None;
        }
        let bumps: [[u8; 1]; 256] = std::array::from_fn(|i| [i as u8]);
        let mut with_bump: Vec<&[u8]> = Vec::with_capacity(seeds.len() + 1);
        for bump in (0..=255u8).rev() {
            with_bump.clear();
            with_bump.extend_from_slice(seeds);
            with_bump.push(&bumps[bump as usize]);
            if let Some(pda) = Self::create_program_address(&with_bump, program_id) {
                return Some((pda, bump));
            }
        }
        None
    }
}

impl FromStr for Pubkey {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // A 32-byte value is at most 44 base58 chars; reject early.
        if s.is_empty() || s.len() > 44 {
            return Err(DecodeError::Base58);
        }
        let mut out = [0u8; 32];
        let n = bs58::decode(s).onto(&mut out).map_err(|_| DecodeError::Base58)?;
        if n != 32 {
            return Err(DecodeError::Length { expected: 32, got: n });
        }
        Ok(Pubkey(out))
    }
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_base58())
    }
}

impl fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pubkey({})", self.to_base58())
    }
}

impl Serialize for Pubkey {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_base58())
    }
}

impl<'de> Deserialize<'de> for Pubkey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str>>::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::programs::{ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID};

    #[test]
    fn base58_roundtrip() {
        let s = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
        let k: Pubkey = s.parse().unwrap();
        assert_eq!(k.to_string(), s);
        assert!("not-base58!".parse::<Pubkey>().is_err());
        assert!("1111".parse::<Pubkey>().is_err());
    }

    #[test]
    fn ata_matches_known_address() {
        // USDC mainnet ATA for a known wallet, cross-checked against
        // `solana find-program-derived-address`.
        let wallet: Pubkey = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse().unwrap();
        let mint: Pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".parse().unwrap();
        let (ata, _) = Pubkey::find_program_address(
            &[wallet.as_bytes(), TOKEN_PROGRAM_ID.as_bytes(), mint.as_bytes()],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        )
        .unwrap();
        assert!(!ata.is_on_curve());
        assert_eq!(ata.to_string(), "FGETo8T8wMcN2wCjav8VK6eh3dLk63evNDPxzLSJra8B");
    }

    #[test]
    fn system_program_is_on_curve_check() {
        // All-zero bytes decompress to a valid point.
        assert!(Pubkey([0; 32]).is_on_curve());
    }
}
