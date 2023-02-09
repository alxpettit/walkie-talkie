#![allow(clippy::needless_lifetimes)]
// Lints erroneously detect lifetimes as erroneous,
// even when compiler forces us to use explicit lifetimes
// because of our type aliasing.

pub mod denoise;
pub mod fft;
pub mod mic;
pub mod pcmtypes;
pub mod speaker;
pub mod volume;
