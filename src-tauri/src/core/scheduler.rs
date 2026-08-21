//! Choosing what to study and in which order.
//!
//! Today: the order of one classic run. The «when is this card due next»
//! calculation (SM-2 / FSRS) lands here in M17, next to the same generator.
//!
//! Randomness is seeded rather than taken from the system, so a run can be
//! replayed exactly in a test. The generator is a few lines of arithmetic
//! instead of a dependency — nothing here needs cryptographic quality, only
//! «different every time and the same for the same seed».

/// A seeded pseudo-random generator (SplitMix64).
///
/// Small, fast, and good enough for shuffling a deck: it passes the usual
/// statistical batteries, and it has no state to speak of beyond one number.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `[0, bound)`, without the bias a plain `%` would add.
    ///
    /// Values from the last, incomplete stretch of the 64-bit range are
    /// thrown away and redrawn; with a bound as small as a deck size that
    /// practically never happens even once.
    pub fn below(&mut self, bound: u64) -> u64 {
        if bound <= 1 {
            return 0;
        }

        let limit = u64::MAX - (u64::MAX % bound) - 1;
        loop {
            let value = self.next_u64();
            if value <= limit {
                return value % bound;
            }
        }
    }
}

/// Shuffles in place, Fisher–Yates, using `seed`.
///
/// Every ordering is equally likely, and the same seed always gives the same
/// one — which is what makes a run reproducible in a test.
pub fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut rng = Rng::new(seed);

    for index in (1..items.len()).rev() {
        let swap_with = rng.below(index as u64 + 1) as usize;
        items.swap(index, swap_with);
    }
}
