use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Hash {
    id: u32,
    hash: Vec<u8>,
}
impl Hash {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }
}
pub struct Hashes {
    hashes: Vec<Hash>,
}
impl Deref for Hashes {
    type Target = Vec<Hash>;
    fn deref(&self) -> &Self::Target {
        &self.hashes
    }
}
impl From<Vec<Hash>> for Hashes {
    fn from(value: Vec<Hash>) -> Self {
        Self { hashes: value }
    }
}

#[derive(Default)]
pub struct Matches {
    matches: Vec<Match>,
}
impl Deref for Matches {
    type Target = Vec<Match>;
    fn deref(&self) -> &Self::Target {
        &self.matches
    }
}
impl DerefMut for Matches {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matches
    }
}
pub struct Match {
    hash_id1: u32,
    hash_id2: u32,
    hamming_distance: HammingDistance,
}
impl Match {
    pub fn new(hash_id1: u32, hash_id2: u32, hamming_distance: HammingDistance) -> Self {
        Self {
            hash_id1,
            hash_id2,
            hamming_distance,
        }
    }
    pub fn hash_id1(&self) -> u32 {
        self.hash_id1
    }
    pub fn hash_id2(&self) -> u32 {
        self.hash_id2
    }
    pub fn hamming_distance(&self) -> &HammingDistance {
        &self.hamming_distance
    }
}
#[derive(Clone, Copy)]
pub struct HammingDistance {
    distance: u32,
    entry_length: u32,
}
impl HammingDistance {
    pub fn new(distance: u32, entry_length: u32) -> Self {
        Self {
            distance,
            entry_length,
        }
    }
    pub fn distance(&self) -> u32 {
        self.distance
    }
    pub fn entry_length(&self) -> u32 {
        self.entry_length
    }
}
