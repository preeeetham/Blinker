//! HyperLogLog cardinality sketch (Flajolet et al. 2007) with the small-range
//! linear-counting correction. With p = 10 it uses 1 KiB and has ~3.25%
//! standard error, versus 32 bytes per distinct payer for a `HashSet`.

pub const P: u32 = 10;
pub const M: usize = 1 << P;

#[derive(Clone)]
pub struct HyperLogLog {
    registers: Box<[u8; M]>,
}

impl Default for HyperLogLog {
    fn default() -> Self {
        Self { registers: Box::new([0; M]) }
    }
}

/// SplitMix64 finaliser: turns any 64-bit input into a well-mixed hash.
#[inline]
pub fn mix64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

impl HyperLogLog {
    /// Inserts a pre-hashed 64-bit value.
    #[inline]
    pub fn insert_hash(&mut self, h: u64) {
        let idx = (h >> (64 - P)) as usize;
        // Rank = leading zeros of the remaining bits + 1, capped.
        let rest = h << P | (1 << (P - 1));
        let rank = rest.leading_zeros() as u8 + 1;
        let r = &mut self.registers[idx];
        if rank > *r {
            *r = rank;
        }
    }

    /// Inserts a 32-byte key (a pubkey). Pubkeys are already uniform, but
    /// mixing two lanes guards against crafted vanity addresses.
    #[inline]
    pub fn insert_key(&mut self, key: &[u8; 32]) {
        let a = u64::from_le_bytes(key[0..8].try_into().unwrap());
        let b = u64::from_le_bytes(key[8..16].try_into().unwrap());
        self.insert_hash(mix64(a ^ mix64(b)));
    }

    pub fn estimate(&self) -> u64 {
        let m = M as f64;
        let alpha = 0.7213 / (1.0 + 1.079 / m);
        let mut sum = 0.0;
        let mut zeros = 0;
        for &r in self.registers.iter() {
            sum += 2f64.powi(-(r as i32));
            if r == 0 {
                zeros += 1;
            }
        }
        let raw = alpha * m * m / sum;
        if raw <= 2.5 * m && zeros > 0 {
            (m * (m / zeros as f64).ln()).round() as u64
        } else {
            raw.round() as u64
        }
    }

    pub const fn memory_bytes() -> usize {
        M
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accuracy() {
        for n in [0u64, 1, 10, 100, 1_000, 10_000, 100_000] {
            let mut h = HyperLogLog::default();
            for i in 0..n {
                h.insert_hash(mix64(i));
                h.insert_hash(mix64(i)); // duplicates must not count
            }
            let est = h.estimate() as f64;
            let err = if n == 0 { est } else { (est - n as f64).abs() / n as f64 };
            assert!(err < 0.1, "n={n} est={est} err={err}");
        }
    }
}
