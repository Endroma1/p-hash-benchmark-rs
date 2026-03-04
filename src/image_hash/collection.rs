use std::ops::{Deref, DerefMut};

use crate::image_hash::{Hash, HashingMethod};

#[derive(Default)]
pub struct HashingMethods {
    methods: Vec<Box<dyn HashingMethod>>,
}
impl HashingMethods {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, method: impl HashingMethod + 'static) {
        self.methods.push(Box::new(method));
    }
}
impl Deref for HashingMethods {
    type Target = Vec<Box<dyn HashingMethod>>;
    fn deref(&self) -> &Self::Target {
        &self.methods
    }
}

impl DerefMut for HashingMethods {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.methods
    }
}

#[derive(Debug)]
pub struct HashResult {
    hash: Hash,
    hashing_method_id: u16,
}
impl HashResult {
    pub fn new(hash: impl Into<Hash>, hashing_method_id: u16) -> Self {
        Self {
            hash: hash.into(),
            hashing_method_id,
        }
    }
    pub fn hash(&self) -> &Hash {
        &self.hash
    }
    pub fn hashing_method_id(&self) -> &u16 {
        &self.hashing_method_id
    }
}
