use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

pub enum Message {
    // Update the progress of a component
    Update { component: Component, delta: u32 },
    // Set the total expected progress.
    Set { component: Component, total: u32 },
}

#[derive(Debug, Sequence, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Component {
    Fetcher,
    Processor,
    Parser,
}
impl Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetcher => write!(f, "Fetcher"),
            Self::Processor => write!(f, "Processor"),
            Self::Parser => write!(f, "Parser"),
        }
    }
}

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
#[derive(Debug)]
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
#[derive(Debug, Clone, Copy)]
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
