#[cfg(feature = "std")]
mod stdmath {
    #[inline(always)]
    pub(crate) fn mul_add(x: f64, y: f64, z: f64) -> f64 {
        ::std::primitive::f64::mul_add(x, y, z)
    }

    #[inline(always)]
    pub(crate) fn ln(x: f64) -> f64 {
        ::std::primitive::f64::ln(x)
    }

    #[inline(always)]
    pub(crate) fn sqrt(x: f64) -> f64 {
        ::std::primitive::f64::sqrt(x)
    }
}

#[cfg(feature = "std")]
pub(crate) use stdmath::*;

#[cfg(all(not(feature = "std"), feature = "libm"))]
mod libm {
    #[inline(always)]
    pub(crate) fn mul_add(x: f64, y: f64, z: f64) -> f64 {
        ::libm::fma(x, y, z)
    }

    #[inline(always)]
    pub(crate) fn ln(x: f64) -> f64 {
        ::libm::log(x)
    }

    #[inline(always)]
    pub(crate) fn sqrt(x: f64) -> f64 {
        ::libm::sqrt(x)
    }
}

#[cfg(all(not(feature = "std"), feature = "libm"))]
pub(crate) use libm::*;

#[allow(dead_code)]
mod fallback {
    use core::num::FpCategory;

    #[inline(always)]
    pub(crate) fn mul_add(x: f64, y: f64, z: f64) -> f64 {
        (x * y) + z
    }

    const MANTISSA_MASK: u64 = (1 << 52) - 1;
    const EXP: u64 = 1023 << 52;

    const COEFFICIENTS: [f64; 5] = [
        -0.081615808498122389,
        0.64514236358772081,
        -2.1206751311142673,
        4.0700907918522011,
        -2.5128546239033374,
    ];

    /// Inspiration: https://gist.github.com/LingDong-/7e4c4cae5cbbc44400a05fba65f06f23
    #[inline(always)]
    pub(crate) fn log2(mut x: f64) -> f64 {
        match x.classify() {
            FpCategory::Normal if x.is_sign_positive() => {
                let mut bits = x.to_bits();

                let exp = (((bits >> 52) & 0x7ff) as i32).wrapping_sub(0x3ff);

                bits &= MANTISSA_MASK;
                bits |= EXP;

                x = f64::from_bits(bits);

                let mut u = COEFFICIENTS[0];

                for coeff in COEFFICIENTS.into_iter().skip(1) {
                    u *= x;
                    u += coeff;
                }

                u + exp.abs() as f64
            }
            FpCategory::Subnormal | FpCategory::Normal => f64::NAN,
            FpCategory::Nan | FpCategory::Infinite => x,
            FpCategory::Zero => -1. / (x * x),
        }
    }

    #[inline(always)]
    pub(crate) fn ln(x: f64) -> f64 {
        log2(x) * ::core::f64::consts::LN_2
    }

    /// Inspiration: https://suraj.sh/fast-square-root-approximation
    #[inline(always)]
    pub(crate) fn sqrt(x: f64) -> f64 {
        /// bit repr of 0.04303566602796716
        const BIT_HACK: u64 = 0x1ff7a7dceaaec900;

        let mut bits = x.to_bits();
        bits = BIT_HACK.wrapping_add(bits >> 1);

        let mut y = f64::from_bits(bits);

        y = (y + (x / y)) * 0.5;
        y = (y + (x / y)) * 0.5;
        y = (y + (x / y)) * 0.5;

        y
    }
}

#[cfg(not(any(feature = "std", feature = "libm")))]
pub(crate) use fallback::*;
