use core::sync::atomic::{AtomicI64, Ordering};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86;

pub type NextI64 = fn() -> i64;

pub trait EntropySource: Sized {
    fn get_entropy(self) -> NextI64;
}

impl EntropySource for NextI64 {
    fn get_entropy(self) -> NextI64 {
        self
    }
}

impl EntropySource for () {
    fn get_entropy(self) -> NextI64 {
        || 0
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StaticSource;

static STATIC_ENTROPY: AtomicI64 = AtomicI64::new(0);

impl StaticSource {
    pub fn get() -> i64 {
        STATIC_ENTROPY.load(Ordering::Relaxed)
    }

    pub fn set(value: i64) -> i64 {
        STATIC_ENTROPY.swap(value, Ordering::Relaxed)
    }
}

impl EntropySource for StaticSource {
    fn get_entropy(self) -> NextI64 {
        StaticSource::get
    }
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, Default)]
pub struct NanosecondSource;

impl EntropySource for NanosecondSource {
    fn get_entropy(self) -> NextI64 {
        || {
            use ::std::time::SystemTime;
            let time = SystemTime::now();

            let duration = match time.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(duration) => duration,
                Err(error) => error.duration(),
            };

            // We do not really care if it's lossy at this point.
            duration.as_nanos() as i64
        }
    }
}
