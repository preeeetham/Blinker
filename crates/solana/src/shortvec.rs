//! Solana's "compact-u16" (a.k.a. short_vec) length prefix: LEB128 limited to
//! three bytes and to values that fit in a u16.

use crate::error::DecodeError;

/// Appends `value` as compact-u16. Returns the number of bytes written.
#[inline]
pub fn encode(mut value: u16, out: &mut Vec<u8>) -> usize {
    let mut n = 0;
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        n += 1;
        if value == 0 {
            return n;
        }
    }
}

/// Encoded size of `value` without writing it.
#[inline]
pub const fn encoded_len(value: u16) -> usize {
    if value < 0x80 {
        1
    } else if value < 0x4000 {
        2
    } else {
        3
    }
}

/// Decodes a compact-u16 from the start of `buf`, returning `(value, bytes_used)`.
///
/// Rejects non-minimal encodings (e.g. `[0x80, 0x00]` for 0) and values above
/// `u16::MAX`, matching the validator's behaviour so a crafted transaction
/// can't decode differently here than on-chain.
pub fn decode(buf: &[u8]) -> Result<(u16, usize), DecodeError> {
    let mut value: u32 = 0;
    for i in 0..3 {
        let byte = *buf.get(i).ok_or(DecodeError::BadShortVec)?;
        let part = (byte & 0x7f) as u32;
        // A zero continuation byte after the first means a non-minimal encoding.
        if i > 0 && byte == 0 {
            return Err(DecodeError::BadShortVec);
        }
        value |= part << (7 * i);
        if byte & 0x80 == 0 {
            if value > u16::MAX as u32 {
                return Err(DecodeError::BadShortVec);
            }
            return Ok((value as u16, i + 1));
        }
        if i == 2 {
            // Third byte must terminate.
            return Err(DecodeError::BadShortVec);
        }
    }
    Err(DecodeError::BadShortVec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        // Vectors from solana-sdk's short_vec tests.
        let cases: &[(u16, &[u8])] = &[
            (0x0, &[0x0]),
            (0x7f, &[0x7f]),
            (0x80, &[0x80, 0x01]),
            (0xff, &[0xff, 0x01]),
            (0x100, &[0x80, 0x02]),
            (0x7fff, &[0xff, 0xff, 0x01]),
            (0xffff, &[0xff, 0xff, 0x03]),
        ];
        for &(v, bytes) in cases {
            let mut out = Vec::new();
            encode(v, &mut out);
            assert_eq!(out, bytes, "encode {v:#x}");
            assert_eq!(encoded_len(v), bytes.len());
            assert_eq!(decode(bytes).unwrap(), (v, bytes.len()), "decode {v:#x}");
        }
    }

    #[test]
    fn rejects_bad_encodings() {
        assert!(decode(&[0x80, 0x00]).is_err()); // alias of 0
        assert!(decode(&[0xff, 0xff, 0x04]).is_err()); // > u16::MAX
        assert!(decode(&[0x80, 0x80, 0x80, 0x01]).is_err()); // 4 bytes
        assert!(decode(&[0x80]).is_err()); // truncated
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn roundtrip_all() {
        let mut out = Vec::with_capacity(3);
        for v in 0..=u16::MAX {
            out.clear();
            encode(v, &mut out);
            assert_eq!(decode(&out).unwrap(), (v, out.len()));
        }
    }
}
