//! Cryptographic hash functions
//!
//! The [`sha256`] function requires feature either `sha2` or `ring` (not both).

pub trait Sha256 {
    fn sha256(data: &[u8]) -> [u8; 32];
}
