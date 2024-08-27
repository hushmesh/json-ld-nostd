//! Cryptographic hash functions

use alloc::vec::Vec;

pub trait Sha {
    fn hash(data: &[u8]) -> Vec<u8>;
}
