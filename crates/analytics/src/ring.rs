//! Fixed-size time-bucketed ring buffer.
//!
//! Bucket `i` holds the window `epoch` where `epoch % N == i`. Each bucket
//! remembers which epoch it holds, so stale buckets are lazily reset on
//! write and skipped on read — no background sweeper, no allocation.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bucket {
    pub views: u32,
    pub posts: u32,
    pub paid: u32,
    pub volume: u64,
}

impl Bucket {
    #[inline]
    pub fn add(&mut self, o: &Bucket) {
        self.views += o.views;
        self.posts += o.posts;
        self.paid += o.paid;
        self.volume += o.volume;
    }
}

#[derive(Clone)]
pub struct Ring<const N: usize> {
    width_ms: u64,
    epochs: [u64; N],
    buckets: [Bucket; N],
}

impl<const N: usize> Ring<N> {
    pub fn new(width_ms: u64) -> Self {
        Self { width_ms, epochs: [u64::MAX; N], buckets: [Bucket::default(); N] }
    }

    /// The bucket for `ts_ms`, or `None` if that window has already been
    /// overwritten by a newer one (an event older than the ring's span).
    #[inline]
    pub fn slot(&mut self, ts_ms: u64) -> Option<&mut Bucket> {
        let epoch = ts_ms / self.width_ms;
        let i = (epoch % N as u64) as usize;
        if self.epochs[i] != epoch {
            // Only move forward: a late event must not reset a newer window.
            if self.epochs[i] != u64::MAX && self.epochs[i] > epoch {
                return None;
            }
            self.epochs[i] = epoch;
            self.buckets[i] = Bucket::default();
        }
        Some(&mut self.buckets[i])
    }

    /// The last `N` windows ending at `now_ms`, oldest first, as
    /// `(window_start_ms, bucket)`. Missing windows are zero.
    pub fn series(&self, now_ms: u64) -> Vec<(u64, Bucket)> {
        let now_epoch = now_ms / self.width_ms;
        let start = now_epoch.saturating_sub(N as u64 - 1);
        (start..=now_epoch)
            .map(|e| {
                let i = (e % N as u64) as usize;
                let b = if self.epochs[i] == e { self.buckets[i] } else { Bucket::default() };
                (e * self.width_ms, b)
            })
            .collect()
    }

    /// Sum over windows newer than `now - span`.
    pub fn sum_since(&self, now_ms: u64, span_ms: u64) -> Bucket {
        let lo = now_ms.saturating_sub(span_ms) / self.width_ms;
        let hi = now_ms / self.width_ms;
        let mut acc = Bucket::default();
        for i in 0..N {
            let e = self.epochs[i];
            if e != u64::MAX && e > lo && e <= hi {
                acc.add(&self.buckets[i]);
            }
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_and_expires() {
        let mut r: Ring<4> = Ring::new(1000);
        r.slot(0).unwrap().views += 1;
        r.slot(1500).unwrap().views += 2;
        r.slot(3999).unwrap().views += 3;
        let s = r.series(3999);
        assert_eq!(s.iter().map(|(_, b)| b.views).collect::<Vec<_>>(), [1, 2, 0, 3]);
        // Epoch 4 reuses slot 0 and resets it.
        r.slot(4000).unwrap().views += 10;
        // An event from epoch 0 is now older than the ring: dropped.
        assert!(r.slot(10).is_none());
        let s = r.series(4000);
        assert_eq!(s.iter().map(|(_, b)| b.views).collect::<Vec<_>>(), [2, 0, 3, 10]);
        assert_eq!(r.sum_since(4000, 2000).views, 13);
    }
}
