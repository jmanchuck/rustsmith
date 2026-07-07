//! Dependency-free PRNG with a rand-like API surface.
//!
//! Replaces the `rand` crate so the workspace builds with std only. The
//! generator only needs reproducible, well-mixed randomness — not
//! cryptographic quality — so this uses xoshiro256** seeded via SplitMix64.
//!
//! Integer range sampling uses modulo reduction. The tiny modulo bias is
//! irrelevant for fuzzing and keeps sampling branch-free and fast.

pub struct SmithRng {
    s: [u64; 4],
}

/// Alias so existing `StdRng::seed_from_u64` call sites keep working.
pub type StdRng = SmithRng;

impl SmithRng {
    pub fn seed_from_u64(seed: u64) -> Self {
        // SplitMix64 to spread a 64-bit seed over 256 bits of state.
        let mut x = seed;
        let mut next = move || {
            x = x.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        };
        let s = [next(), next(), next(), next()];
        SmithRng { s }
    }
}

impl Rng for SmithRng {
    fn next_u64(&mut self) -> u64 {
        // xoshiro256**
        let result = self.s[1]
            .wrapping_mul(5)
            .rotate_left(7)
            .wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }
}

pub trait Rng {
    fn next_u64(&mut self) -> u64;

    fn next_u128(&mut self) -> u128 {
        ((self.next_u64() as u128) << 64) | self.next_u64() as u128
    }

    fn gen<T: RandGen>(&mut self) -> T
    where
        Self: Sized,
    {
        T::rand_gen(self)
    }

    fn gen_range<R: SampleRange>(&mut self, range: R) -> R::Output
    where
        Self: Sized,
    {
        range.sample_from(self)
    }

    fn gen_bool(&mut self, p: f64) -> bool
    where
        Self: Sized,
    {
        self.gen::<f64>() < p
    }
}

impl<R: Rng + ?Sized> Rng for &mut R {
    fn next_u64(&mut self) -> u64 {
        (**self).next_u64()
    }
}

/// Replacement for `rand`'s `Distribution<T> for Standard` / `rng.gen::<T>()`.
pub trait RandGen: Sized {
    fn rand_gen<R: Rng>(rng: &mut R) -> Self;
}

macro_rules! impl_randgen_int {
    ($($t:ty),+) => {
        $(impl RandGen for $t {
            fn rand_gen<R: Rng>(rng: &mut R) -> Self {
                rng.next_u64() as $t
            }
        })+
    };
}

impl_randgen_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize);

impl RandGen for u128 {
    fn rand_gen<R: Rng>(rng: &mut R) -> Self {
        rng.next_u128()
    }
}

impl RandGen for i128 {
    fn rand_gen<R: Rng>(rng: &mut R) -> Self {
        rng.next_u128() as i128
    }
}

impl RandGen for bool {
    fn rand_gen<R: Rng>(rng: &mut R) -> Self {
        rng.next_u64() & 1 == 1
    }
}

impl RandGen for f32 {
    fn rand_gen<R: Rng>(rng: &mut R) -> Self {
        // 24 explicit mantissa bits -> uniform in [0, 1)
        (rng.next_u64() >> 40) as f32 * (1.0 / (1u64 << 24) as f32)
    }
}

impl RandGen for f64 {
    fn rand_gen<R: Rng>(rng: &mut R) -> Self {
        // 53 bits -> uniform in [0, 1)
        (rng.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// Replacement for `rand`'s `SampleRange` so `rng.gen_range(a..b)` and
/// `rng.gen_range(a..=b)` keep working for ints and floats.
pub trait SampleRange {
    type Output;
    fn sample_from<R: Rng>(self, rng: &mut R) -> Self::Output;
}

macro_rules! impl_samplerange_uint {
    ($($t:ty),+) => {
        $(
            impl SampleRange for std::ops::Range<$t> {
                type Output = $t;
                fn sample_from<R: Rng>(self, rng: &mut R) -> $t {
                    assert!(self.start < self.end, "gen_range: empty range");
                    let span = (self.end - self.start) as u128;
                    self.start + (rng.next_u128() % span) as $t
                }
            }
            impl SampleRange for std::ops::RangeInclusive<$t> {
                type Output = $t;
                fn sample_from<R: Rng>(self, rng: &mut R) -> $t {
                    let (lo, hi) = (*self.start(), *self.end());
                    assert!(lo <= hi, "gen_range: empty range");
                    let span = ((hi - lo) as u128).wrapping_add(1);
                    if span == 0 {
                        // Full u128 domain
                        return (rng.next_u128() as $t).wrapping_add(lo);
                    }
                    lo + (rng.next_u128() % span) as $t
                }
            }
        )+
    };
}

impl_samplerange_uint!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_samplerange_int {
    ($($t:ty),+) => {
        $(
            impl SampleRange for std::ops::Range<$t> {
                type Output = $t;
                fn sample_from<R: Rng>(self, rng: &mut R) -> $t {
                    assert!(self.start < self.end, "gen_range: empty range");
                    let span = (self.end as i128 - self.start as i128) as u128;
                    (self.start as i128 + (rng.next_u128() % span) as i128) as $t
                }
            }
            impl SampleRange for std::ops::RangeInclusive<$t> {
                type Output = $t;
                fn sample_from<R: Rng>(self, rng: &mut R) -> $t {
                    let (lo, hi) = (*self.start(), *self.end());
                    assert!(lo <= hi, "gen_range: empty range");
                    let span = (hi as i128 - lo as i128) as u128 + 1;
                    (lo as i128 + (rng.next_u128() % span) as i128) as $t
                }
            }
        )+
    };
}

impl_samplerange_int!(i8, i16, i32, i64);

impl SampleRange for std::ops::Range<i128> {
    type Output = i128;
    fn sample_from<R: Rng>(self, rng: &mut R) -> i128 {
        assert!(self.start < self.end, "gen_range: empty range");
        let span = self.end.wrapping_sub(self.start) as u128;
        self.start.wrapping_add((rng.next_u128() % span) as i128)
    }
}

impl SampleRange for std::ops::RangeInclusive<i128> {
    type Output = i128;
    fn sample_from<R: Rng>(self, rng: &mut R) -> i128 {
        let (lo, hi) = (*self.start(), *self.end());
        assert!(lo <= hi, "gen_range: empty range");
        let span = (hi.wrapping_sub(lo) as u128).wrapping_add(1);
        if span == 0 {
            return rng.next_u128() as i128;
        }
        lo.wrapping_add((rng.next_u128() % span) as i128)
    }
}

impl SampleRange for std::ops::Range<f32> {
    type Output = f32;
    fn sample_from<R: Rng>(self, rng: &mut R) -> f32 {
        self.start + f32::rand_gen(rng) * (self.end - self.start)
    }
}

impl SampleRange for std::ops::Range<f64> {
    type Output = f64;
    fn sample_from<R: Rng>(self, rng: &mut R) -> f64 {
        self.start + f64::rand_gen(rng) * (self.end - self.start)
    }
}

/// Replacement for `rand::seq::SliceRandom::choose`.
pub trait SliceChoose {
    type Item;
    fn choose<R: Rng>(&self, rng: &mut R) -> Option<&Self::Item>;
}

impl<T> SliceChoose for [T] {
    type Item = T;
    fn choose<R: Rng>(&self, rng: &mut R) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self[rng.gen_range(0..self.len())])
        }
    }
}

/// Replacement for `rand::distributions::WeightedIndex` (u32 weights only).
pub struct WeightedIndex {
    cumulative: Vec<u64>,
    total: u64,
}

impl WeightedIndex {
    pub fn new<I: IntoIterator<Item = u32>>(weights: I) -> Result<Self, &'static str> {
        let mut cumulative = Vec::new();
        let mut total: u64 = 0;
        for w in weights {
            total += w as u64;
            cumulative.push(total);
        }
        if total == 0 {
            return Err("WeightedIndex: all weights are zero");
        }
        Ok(WeightedIndex { cumulative, total })
    }

    pub fn sample<R: Rng>(&self, rng: &mut R) -> usize {
        let x = rng.gen_range(0..self.total);
        // First index whose cumulative weight exceeds x
        self.cumulative.partition_point(|&c| c <= x)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deterministic_from_seed() {
        let mut a = SmithRng::seed_from_u64(7);
        let mut b = SmithRng::seed_from_u64(7);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = SmithRng::seed_from_u64(1);
        let mut b = SmithRng::seed_from_u64(2);
        let same = (0..100).filter(|_| a.next_u64() == b.next_u64()).count();
        assert!(same < 5);
    }

    #[test]
    fn gen_range_bounds() {
        let mut rng = SmithRng::seed_from_u64(42);
        for _ in 0..10_000 {
            let x = rng.gen_range(-5i32..5);
            assert!((-5..5).contains(&x));
            let y = rng.gen_range(0u128..=u128::MAX);
            let _ = y;
            let z = rng.gen_range(0.0f32..1.0);
            assert!((0.0..1.0).contains(&z));
            let w = rng.gen_range(3u8..=3);
            assert_eq!(w, 3);
        }
    }

    #[test]
    fn gen_range_hits_extremes() {
        let mut rng = SmithRng::seed_from_u64(1);
        let mut lo_seen = false;
        let mut hi_seen = false;
        for _ in 0..1000 {
            match rng.gen_range(0u8..4) {
                0 => lo_seen = true,
                3 => hi_seen = true,
                _ => {}
            }
        }
        assert!(lo_seen && hi_seen);
    }

    #[test]
    fn weighted_index_respects_zero_weights() {
        let dist = WeightedIndex::new(vec![0u32, 5, 0, 5]).unwrap();
        let mut rng = SmithRng::seed_from_u64(9);
        let mut counts = [0usize; 4];
        for _ in 0..10_000 {
            counts[dist.sample(&mut rng)] += 1;
        }
        assert_eq!(counts[0], 0);
        assert_eq!(counts[2], 0);
        assert!(counts[1] > 4000 && counts[3] > 4000);
    }

    #[test]
    fn slice_choose() {
        let mut rng = SmithRng::seed_from_u64(3);
        let empty: [u8; 0] = [];
        assert!(empty.choose(&mut rng).is_none());
        let items = [10, 20, 30];
        for _ in 0..100 {
            assert!(items.contains(items.choose(&mut rng).unwrap()));
        }
    }

    #[test]
    fn gen_bool_probability() {
        let mut rng = SmithRng::seed_from_u64(5);
        let hits = (0..10_000).filter(|_| rng.gen_bool(0.3)).count();
        assert!(hits > 2500 && hits < 3500, "hits: {}", hits);
    }
}
