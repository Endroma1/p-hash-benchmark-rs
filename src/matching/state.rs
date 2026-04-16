use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use bitvec::view::BitViewSized;
use crossbeam::channel::{RecvError, bounded};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Clone only increases reference count of channels(hopefully)
#[derive(Clone)]
pub struct MatchState {
    producer: crossbeam::channel::Sender<Message>,
    consumer: crossbeam::channel::Receiver<Message>,
}
impl MatchState {
    pub fn new() -> Self {
        let (state_handle, sub) = bounded(10000);
        Self {
            consumer: sub,
            producer: state_handle,
        }
    }
    pub fn update(&self, component: Component, delta: u32) {
        self.producer
            .send(Message::Update { component, delta })
            .unwrap();
    }
    pub fn set(&self, component: Component, total: u32) {
        self.producer
            .send(Message::Set { component, total })
            .unwrap();
    }
    pub fn get(&self) -> Result<Message, RecvError> {
        self.consumer.recv()
    }
}

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
#[derive(Debug, FromRow)]
pub struct Match {
    #[sqlx(rename = "hash1_id")]
    hash_id1: u32,
    #[sqlx(rename = "hash2_id")]
    hash_id2: u32,
    #[sqlx(flatten)]
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
#[derive(Debug, Clone, Copy, FromRow)]
pub struct HammingDistance {
    #[sqlx(rename = "hamming_distance")]
    distance: u32,
    #[sqlx(rename = "hash_len")]
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
    pub fn relative(&self) -> f32 {
        self.distance as f32 / self.entry_length as f32
    }
}
