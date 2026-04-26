use std::fmt::Display;

use bitvec::prelude::*;
use image::DynamicImage;

use crate::image_hash::{HashingMethods, SelectedHashingMethods, collection::HashResult};

pub trait HashingMethod: Send + Sync {
    fn hash(&self, img: &DynamicImage) -> Hash;
    fn name(&self) -> String;
}
pub fn hash_images(img: DynamicImage, hashing_methods: &SelectedHashingMethods) -> Vec<HashResult> {
    // Hashes image with hashing methods that correlate to the given ids
    hashing_methods
        .iter()
        .enumerate()
        .map(move |(id, method)| HashResult::new(method.hash(&img), id as u16))
        .collect()
}

#[derive(Debug)]
pub struct Hash {
    bits: BitVec<u8, Msb0>,
}

impl From<&[u8]> for Hash {
    fn from(value: &[u8]) -> Self {
        let value = BitVec::from_slice(value);
        Self { bits: value }
    }
}

impl Hash {
    pub fn new(bits: BitVec<u8, Msb0>) -> Self {
        Self { bits }
    }
    pub fn to_bytes(&self) -> &[u8] {
        self.bits.as_raw_slice()
    }

    pub fn to_hex(self) -> String {
        let bytes = self.to_bytes();
        hex::encode(bytes)
    }
}
impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bits)
    }
}
