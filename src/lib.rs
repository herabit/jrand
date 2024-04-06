#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod entropy;

mod math;
mod random;
pub use random::*;
