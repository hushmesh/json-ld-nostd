//! Cryptographic hash functions

pub trait Sha {
    fn hash(data: &[u8]) -> &[u8];
}
