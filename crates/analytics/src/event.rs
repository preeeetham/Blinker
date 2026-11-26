//! Binary event codec.
//!
//! ```text
//! 0       1       2              10            22
//! | ver=1 | kind  | ts_ms u64 le | blink_id[12] | kind-specific payload
//! ```
//!
//! Fixed offsets keep decode branch-light. Decoding is strict (exact length
//! per kind, known version, ASCII id) because the bytes come off disk and
//! the fuzzer feeds it garbage.

use std::fmt;

use serde::{Deserialize, Serialize};

pub const VERSION: u8 = 1;
pub const ID_LEN: usize = 12;
const HEAD: usize = 1 + 1 + 8 + ID_LEN;

/// A 12-character base62 blink id, stored inline (no heap) so events and
/// map keys are `Copy`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlinkId([u8; ID_LEN]);

const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

impl BlinkId {
    /// Builds an id from 12 random bytes (rejection-free: `% 62` bias on 256
    /// values is < 1.6% and irrelevant for ids with 71 bits of entropy).
    pub fn from_random(bytes: [u8; ID_LEN]) -> Self {
        Self(bytes.map(|b| ALPHABET[(b % 62) as usize]))
    }

    pub fn parse(s: &str) -> Option<Self> {
        let b = s.as_bytes();
        if b.len() != ID_LEN || !b.iter().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        let mut out = [0u8; ID_LEN];
        out.copy_from_slice(b);
        Some(Self(out))
    }

    pub fn as_str(&self) -> &str {
        // Invariant: always ASCII alphanumeric.
        std::str::from_utf8(&self.0).unwrap_or("????????????")
    }

    pub fn as_bytes(&self) -> &[u8; ID_LEN] {
        &self.0
    }
}

impl fmt::Display for BlinkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for BlinkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlinkId({})", self.as_str())
    }
}

impl Serialize for BlinkId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BlinkId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str>>::deserialize(d)?;
        BlinkId::parse(&s).ok_or_else(|| serde::de::Error::custom("invalid blink id"))
    }
}

/// What a blink collects payments in. Volume is only summed across blinks
/// of the same asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Asset {
    Sol = 0,
    Spl = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// Blink metadata created (fee not yet paid).
    Created { asset: Asset },
    /// Creation fee confirmed on-chain; the blink is live.
    Activated { fee_lamports: u64 },
    /// Someone fetched the action (a wallet/unfurler rendered it).
    View,
    /// Someone requested a transaction for an amount (base units).
    ActionPost { amount: u64 },
    /// A payment through the blink was confirmed on-chain.
    Paid { amount: u64, payer: [u8; 32] },
}

impl EventKind {
    fn tag(&self) -> u8 {
        match self {
            EventKind::Created { .. } => 0,
            EventKind::Activated { .. } => 1,
            EventKind::View => 2,
            EventKind::ActionPost { .. } => 3,
            EventKind::Paid { .. } => 4,
        }
    }

    fn payload_len(tag: u8) -> Option<usize> {
        Some(match tag {
            2 => 0,
            0 => 1,
            1 | 3 => 8,
            4 => 8 + 32,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            EventKind::Created { .. } => "created",
            EventKind::Activated { .. } => "activated",
            EventKind::View => "view",
            EventKind::ActionPost { .. } => "action_post",
            EventKind::Paid { .. } => "paid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    pub ts_ms: u64,
    pub blink: BlinkId,
    pub kind: EventKind,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CodecError {
    #[error("record too short")]
    Short,
    #[error("unknown version {0}")]
    Version(u8),
    #[error("unknown kind {0}")]
    Kind(u8),
    #[error("bad length for kind")]
    Length,
    #[error("invalid blink id")]
    Id,
}

/// Largest encoded event; lets callers use a stack buffer.
pub const MAX_ENCODED: usize = HEAD + 40;

impl Event {
    /// Encodes into a stack buffer, returning the used prefix length.
    pub fn encode(&self, out: &mut [u8; MAX_ENCODED]) -> usize {
        out[0] = VERSION;
        out[1] = self.kind.tag();
        out[2..10].copy_from_slice(&self.ts_ms.to_le_bytes());
        out[10..HEAD].copy_from_slice(&self.blink.0);
        match self.kind {
            EventKind::View => HEAD,
            EventKind::Created { asset } => {
                out[HEAD] = asset as u8;
                HEAD + 1
            }
            EventKind::Activated { fee_lamports: v } | EventKind::ActionPost { amount: v } => {
                out[HEAD..HEAD + 8].copy_from_slice(&v.to_le_bytes());
                HEAD + 8
            }
            EventKind::Paid { amount, payer } => {
                out[HEAD..HEAD + 8].copy_from_slice(&amount.to_le_bytes());
                out[HEAD + 8..HEAD + 40].copy_from_slice(&payer);
                HEAD + 40
            }
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = [0u8; MAX_ENCODED];
        let n = self.encode(&mut buf);
        buf[..n].to_vec()
    }

    pub fn decode(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() < HEAD {
            return Err(CodecError::Short);
        }
        if b[0] != VERSION {
            return Err(CodecError::Version(b[0]));
        }
        let tag = b[1];
        let want = EventKind::payload_len(tag).ok_or(CodecError::Kind(tag))?;
        if b.len() != HEAD + want {
            return Err(CodecError::Length);
        }
        let ts_ms = u64::from_le_bytes(b[2..10].try_into().unwrap());
        let mut id = [0u8; ID_LEN];
        id.copy_from_slice(&b[10..HEAD]);
        if !id.iter().all(|c| c.is_ascii_alphanumeric()) {
            return Err(CodecError::Id);
        }
        let u64_at = |i: usize| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
        let kind = match tag {
            0 => EventKind::Created {
                asset: match b[HEAD] {
                    0 => Asset::Sol,
                    1 => Asset::Spl,
                    _ => return Err(CodecError::Length),
                },
            },
            1 => EventKind::Activated { fee_lamports: u64_at(HEAD) },
            2 => EventKind::View,
            3 => EventKind::ActionPost { amount: u64_at(HEAD) },
            4 => {
                let mut payer = [0u8; 32];
                payer.copy_from_slice(&b[HEAD + 8..HEAD + 40]);
                EventKind::Paid { amount: u64_at(HEAD), payer }
            }
            _ => unreachable!(),
        };
        Ok(Event { ts_ms, blink: BlinkId(id), kind })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all_kinds() {
        let blink = BlinkId::parse("abcDEF012345").unwrap();
        for kind in [
            EventKind::Created { asset: Asset::Sol },
            EventKind::Created { asset: Asset::Spl },
            EventKind::Activated { fee_lamports: 1_000_000 },
            EventKind::View,
            EventKind::ActionPost { amount: 42 },
            EventKind::Paid { amount: 7, payer: [9; 32] },
        ] {
            let e = Event { ts_ms: 1_764_000_000_000, blink, kind };
            assert_eq!(Event::decode(&e.to_vec()).unwrap(), e);
        }
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(Event::decode(&[]), Err(CodecError::Short));
        let mut v = Event { ts_ms: 1, blink: BlinkId::parse("aaaaaaaaaaaa").unwrap(), kind: EventKind::View }.to_vec();
        v.push(0);
        assert_eq!(Event::decode(&v), Err(CodecError::Length));
        v.pop();
        v[10] = b'!';
        assert_eq!(Event::decode(&v), Err(CodecError::Id));
        v[1] = 99;
        assert_eq!(Event::decode(&v), Err(CodecError::Kind(99)));
    }

    #[test]
    fn ids() {
        let id = BlinkId::from_random([255; 12]);
        assert!(BlinkId::parse(id.as_str()).is_some());
        assert!(BlinkId::parse("short").is_none());
        assert!(BlinkId::parse("has space!!!").is_none());
    }
}
