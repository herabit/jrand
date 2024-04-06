cpufeatures::new!(cpuid_rdseed, "rdseed");
cpufeatures::new!(cpuid_rdrand, "rdrand");

use cfg_if::cfg_if;

use super::EntropySource;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        use core::arch::x86_64 as arch;
    } else if #[cfg(target_arch = "x86")] {
        use core::arch::x86 as arch;
    }
}

const RDRAND_LIMIT: u32 = 10;

#[target_feature(enable = "rdrand")]
unsafe fn rdrand_32() -> Option<u32> {
    for _ in 0..RDRAND_LIMIT {
        let mut rand = 0u32;

        if arch::_rdrand32_step(&mut rand) == 1 {
            return Some(rand);
        }
    }

    None
}

#[target_feature(enable = "rdrand")]
#[cfg(target_arch = "x86_64")]
unsafe fn rdrand_64() -> Option<u64> {
    for _ in 0..RDRAND_LIMIT {
        let mut rand = 0u64;

        if arch::_rdrand64_step(&mut rand) == 1 {
            return Some(rand);
        }
    }

    None
}

#[target_feature(enable = "rdrand")]
unsafe fn rdrand() -> Option<u64> {
    if cfg!(not(target_arch = "x86_64")) {
        let upper = (rdrand_32()? << 32) as u64;
        let lower = rdrand_32()? as u64;

        Some(upper | lower)
    } else {
        rdrand_64()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RdRand(());

impl RdRand {
    #[inline]
    pub fn new() -> Option<RdRand> {
        cpuid_rdrand::get().then_some(RdRand(()))
    }

    #[inline]
    pub unsafe fn new_unchecked() -> RdRand {
        RdRand(())
    }

    #[inline]
    pub fn try_next_u64(self) -> Option<u64> {
        // SAFETY: If we have a `RdRand`, then the rdrand instruction must exist.
        unsafe { rdrand() }
    }

    #[inline]
    pub fn try_next_i64(self) -> Option<i64> {
        self.try_next_u64().map(|x| x as i64)
    }

    #[inline]
    pub fn next_u64(self) -> u64 {
        self.try_next_u64()
            .expect("failed to generate random number with rdrand")
    }

    #[inline]
    pub fn next_i64(self) -> i64 {
        self.next_u64() as i64
    }
}

impl EntropySource for RdRand {
    fn get_entropy(self) -> super::NextI64 {
        || {
            // SAFETY: Same invariants discussed inside of [`RdRand::try_next_u64`].
            unsafe { RdRand::new_unchecked().next_u64() as i64 }
        }
    }
}
