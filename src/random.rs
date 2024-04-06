use super::math;
use core::{
    iter::{repeat_with, FusedIterator},
    ops::Range,
};

/// Random number generator that replicates the behavior of
/// `java.util.Random` in Java.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct JavaRng {
    pub(crate) seed: i64,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) next_gaussian: Option<f64>,
}

impl JavaRng {
    /// Create a random number generator with the specified
    /// `seed`.
    #[inline]
    #[must_use]
    pub const fn with_seed(seed: i64) -> JavaRng {
        JavaRng {
            seed: initial_scramble(seed),
            next_gaussian: None,
        }
    }

    /// Create a random number generator with a seed of zero.
    #[inline]
    #[must_use]
    pub const fn new_zeroed() -> JavaRng {
        JavaRng::with_seed(0)
    }

    /// Create a random number generator using the current system time
    /// as a source of entropy.
    ///
    /// This is equivalent to what Java does
    /// create a `java.util.Random` object with no seed.
    #[inline]
    #[must_use]
    #[cfg(feature = "std")]
    pub fn new_nanos() -> JavaRng {
        JavaRng::with_seed(get_seed())
    }

    /// Create a random number generator.
    ///
    /// With the `std` feature enabled, it is equivalent to calling
    /// [`JavaRng::new_nanos`].
    ///
    /// With the `std` feature disabled, it is equivalent to calling
    /// [`JavaRng::new_zeroed`].
    #[inline]
    #[must_use]
    pub fn new() -> JavaRng {
        #[cfg(not(feature = "std"))]
        {
            JavaRng::new_zeroed()
        }

        #[cfg(feature = "std")]
        {
            JavaRng::new_nanos()
        }
    }
}

impl JavaRng {
    #[inline]
    #[must_use]
    pub(crate) fn next(&mut self, bits: u8) -> i32 {
        self.seed = next_seed(self.seed);

        (self.seed as u64 >> (48 - bits)) as i32
    }
}

impl JavaRng {
    #[inline]
    #[must_use]
    pub fn next_bytes(&mut self, bytes: &mut [u8]) {
        bytes.chunks_mut(4).for_each(|chunk| {
            let bytes = self.next_i32().to_le_bytes();

            chunk.copy_from_slice(&bytes[..chunk.len()])
        });
    }

    #[inline]
    #[must_use]
    pub fn next_bytes_signed(&mut self, bytes: &mut [i8]) {
        self.next_bytes(bytemuck::cast_slice_mut(bytes))
    }

    #[inline]
    #[must_use]
    pub fn next_i32(&mut self) -> i32 {
        self.next(32)
    }

    #[inline]
    #[must_use]
    pub fn next_i32_bounded(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");

        let max = bound - 1;

        if bound & max == 0 {
            return ((self.next(31) as i64).wrapping_mul(bound as i64) >> 31) as i32;
        }

        loop {
            let bits = self.next(31);
            let rem = bits % bound;

            if bits.wrapping_sub(rem).wrapping_add(max) >= 0 {
                break rem;
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn next_i32_ranged(&mut self, range: Range<i32>) -> i32 {
        let Range {
            start: origin,
            end: bound,
        } = range;

        let len = bound.wrapping_sub(origin);

        if origin >= bound {
            self.next_i32()
        } else if len > 0 {
            self.next_i32_bounded(len) + origin
        } else {
            loop {
                let r = self.next_i32();

                if (origin..bound).contains(&r) {
                    break r;
                }
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn i32_iter(&mut self) -> impl Iterator<Item = i32> + FusedIterator + '_ {
        repeat_with(|| self.next_i32())
    }

    #[inline]
    #[must_use]
    pub fn i32_iter_bounded(
        &mut self,
        bound: i32,
    ) -> impl Iterator<Item = i32> + FusedIterator + '_ {
        repeat_with(move || self.next_i32_bounded(bound))
    }

    #[inline]
    #[must_use]
    pub fn i32_iter_ranged(
        &mut self,
        range: Range<i32>,
    ) -> impl Iterator<Item = i32> + FusedIterator + '_ {
        repeat_with(move || self.next_i32_ranged(range.clone()))
    }

    #[inline]
    #[must_use]
    pub fn next_u32(&mut self) -> u32 {
        self.next_i32() as u32
    }

    #[inline]
    #[must_use]
    pub fn next_i64(&mut self) -> i64 {
        let upper = (self.next_i32() as i64) << 32;
        let lower = self.next_i32() as i64;

        upper.wrapping_add(lower)
    }

    #[inline]
    #[must_use]
    pub fn next_i64_ranged(&mut self, range: Range<i64>) -> i64 {
        let Range {
            start: origin,
            end: bound,
        } = range;

        let len = bound.wrapping_sub(origin);
        let max = len.wrapping_sub(1);

        if origin >= bound {
            self.next_i64()
        } else if len & max == 0 {
            (self.next_i64() & max).wrapping_add(origin)
        } else if len > 0 {
            loop {
                let bits = (self.next_u64() >> 1) as i64;
                let rem = bits % len;

                if bits.wrapping_add(max).wrapping_sub(rem) >= 0 {
                    break rem + origin;
                }
            }
        } else {
            loop {
                let r = self.next_i64();

                if (origin..bound).contains(&r) {
                    break r;
                }
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn i64_iter(&mut self) -> impl Iterator<Item = i64> + FusedIterator + '_ {
        repeat_with(|| self.next_i64())
    }

    #[inline]
    #[must_use]
    pub fn i64_iter_ranged(
        &mut self,
        range: Range<i64>,
    ) -> impl Iterator<Item = i64> + FusedIterator + '_ {
        repeat_with(move || self.next_i64_ranged(range.clone()))
    }

    #[inline]
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.next_i64() as u64
    }

    #[inline]
    #[must_use]
    pub fn next_bool(&mut self) -> bool {
        self.next(1) != 0
    }

    #[inline]
    #[must_use]
    pub fn next_f32(&mut self) -> f32 {
        (self.next(24) as f32) * consts::FLOAT_UNIT
    }

    #[inline]
    #[must_use]
    pub fn next_f64(&mut self) -> f64 {
        let upper = (self.next(26) as i64) << 27;
        let lower = self.next(27) as i64;

        (upper.wrapping_add(lower) as f64) * consts::DOUBLE_UNIT
    }

    #[inline]
    #[must_use]
    pub fn next_f64_ranged(&mut self, range: Range<f64>) -> f64 {
        let Range {
            start: origin,
            end: bound,
        } = range;

        let mut r = self.next_f64();

        if origin < bound {
            r = math::mul_add(r, bound - origin, origin);

            if r >= bound {
                r = f64::from_bits(r.to_bits().wrapping_sub(1));
            }
        }

        r
    }

    #[inline]
    #[must_use]
    pub fn f64_iter(&mut self) -> impl Iterator<Item = f64> + FusedIterator + '_ {
        repeat_with(|| self.next_f64())
    }

    #[inline]
    #[must_use]
    pub fn f64_iter_ranged(
        &mut self,
        range: Range<f64>,
    ) -> impl Iterator<Item = f64> + FusedIterator + '_ {
        repeat_with(move || self.next_f64_ranged(range.clone()))
    }

    #[inline]
    #[must_use]
    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(next) = self.next_gaussian.take() {
            return next;
        }

        let (v1, v2) = repeat_with(|| {
            let v1 = math::mul_add(2., self.next_f64(), -1.);
            let v2 = math::mul_add(2., self.next_f64(), -1.);
            let s = (v1 * v1) + (v2 * v2);

            (v1, v2, s)
        })
        .find(|(.., s)| *s < 1. && *s != 0.)
        .map(|(v1, v2, s)| {
            let multiplier = math::sqrt(-2. * math::ln(s) / s);

            (v1 * multiplier, v2 * multiplier)
        })
        .expect("failed to generate next gaussian values");

        self.next_gaussian = Some(v2);

        v1
    }
}

impl Default for JavaRng {
    fn default() -> Self {
        JavaRng::new()
    }
}

#[inline]
#[must_use]
const fn initial_scramble(seed: i64) -> i64 {
    (seed ^ consts::MULTIPLIER) & consts::MASK
}

#[inline]
#[must_use]
const fn next_seed(seed: i64) -> i64 {
    seed.wrapping_mul(consts::MULTIPLIER)
        .wrapping_add(consts::ADDEND)
        & consts::MASK
}

#[cfg(feature = "std")]
fn get_seed() -> i64 {
    use core::sync::atomic::{AtomicI64, Ordering};
    use std::time::SystemTime;

    let uniquifier = {
        static NEXT_UNIQUIFIER: AtomicI64 = AtomicI64::new(consts::FIRST_UNIQUIFIER);
        let mut prev = NEXT_UNIQUIFIER.load(Ordering::Relaxed);

        loop {
            let next = prev.wrapping_mul(consts::UNIQUIFIER_MULTIPLIER);

            match NEXT_UNIQUIFIER.compare_exchange_weak(
                prev,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(uniquifier) => break uniquifier,
                Err(next_prev) => prev = next_prev,
            }
        }
    };

    let current_nanos = {
        let time = SystemTime::now();

        let duration = match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration,
            Err(error) => error.duration(),
        };

        // We do not really care if it's lossy at this point.
        duration.as_nanos() as i64
    };

    uniquifier ^ current_nanos
}

#[doc(hidden)]
pub mod consts {
    pub const FLOAT_UNIT: f32 = 5.9604645E-8;
    pub const DOUBLE_UNIT: f64 = 1.1102230246251565E-16;

    pub const MULTIPLIER: i64 = 0x5DEECE66D;
    pub const ADDEND: i64 = 0xB;
    pub const MASK: i64 = (1 << 48) - 1;

    pub const FIRST_UNIQUIFIER: i64 = 8682522807148012;
    pub const UNIQUIFIER_MULTIPLIER: i64 = 181783497276652981;
}
