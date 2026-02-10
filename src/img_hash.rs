use std::{collections::HashMap, fmt::Display};

use image::DynamicImage;
use img_hash::HasherConfig;
use sqlx::SqlitePool;

pub trait HashingMethod: Send + Sync {
    fn hash(&self, img: &DynamicImage) -> Hash;
    fn name(&self) -> &str;
}
pub type HashingMethodID = u16;
pub fn hash_images(
    img: DynamicImage,
    ids: Vec<u16>,
) -> impl Iterator<Item = Result<HashResult, Error>> {
    // Hashes image with hashing methods that correlate to the given ids
    ids.into_iter()
        .map(move |id| HashResult::try_new(img.clone(), id.clone()))
}

#[derive(Default)]
pub struct HashingMethods {
    methods: HashMap<u16, Box<dyn HashingMethod>>,
}
impl HashingMethods {
    pub fn new() -> Self {
        let mut hashing_methods = Vec::new();
        hashing_methods.push(Box::new(AverageHash::new()) as Box<dyn HashingMethod>);
        hashing_methods.push(Box::new(VertGradient::new()) as Box<dyn HashingMethod>);

        let mut methods = HashMap::new();
        for (i, m) in hashing_methods.into_iter().enumerate() {
            methods.insert(i as u16, m);
        }

        Self { methods }
    }
    pub fn get_methods(&self) -> impl Iterator<Item = &dyn HashingMethod> {
        self.methods.values().map(|m| &**m)
    }
    pub fn get_keys(&self) -> Vec<u16> {
        self.methods.keys().cloned().collect()
    }
    pub fn get_method(&self, id: u16) -> Option<&dyn HashingMethod> {
        self.methods.get(&id).map(|m| &**m)
    }
    pub async fn send_to_db(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        for (id, obj) in &self.methods {
            sqlx::query(
                "
                INSERT INTO hashing_methods (id, name) VALUES (?,?) ON CONFLICT(id) DO NOTHING;
                ",
            )
            .bind(id)
            .bind(obj.name())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

struct AverageHash {}
impl AverageHash {
    fn new() -> Self {
        Self {}
    }
}
impl HashingMethod for AverageHash {
    fn hash(&self, img: &DynamicImage) -> Hash {
        let hasher = HasherConfig::new()
            .hash_alg(img_hash::HashAlg::Mean)
            .to_hasher();
        let res = hasher.hash_image(img);
        res.as_bytes().into()
    }
    fn name(&self) -> &str {
        "average_hash"
    }
}

struct VertGradient {}
impl VertGradient {
    fn new() -> Self {
        Self {}
    }
}
impl HashingMethod for VertGradient {
    fn hash(&self, img: &DynamicImage) -> Hash {
        let hasher = HasherConfig::new()
            .hash_alg(img_hash::HashAlg::VertGradient)
            .to_hasher();
        let res = hasher.hash_image(img);
        res.as_bytes().into()
    }
    fn name(&self) -> &str {
        "vert_gradient"
    }
}
#[derive(Debug)]
pub struct Hash {
    hash: Box<[u8]>,
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
    pub fn try_new(img: DynamicImage, id: HashingMethodID) -> Result<Self, Error> {
        let hashing_methods = HashingMethods::new();
        let hash = hashing_methods
            .get_method(id)
            .ok_or(Error::HashingMethodNotFound { id })?
            .hash(&img);

        Ok(Self {
            hash,
            hashing_method_id: id,
        })
    }
    pub fn hash(&self) -> &Hash {
        &self.hash
    }
    pub fn hashing_method_id(&self) -> &u16 {
        &self.hashing_method_id
    }
}
impl From<&[u8]> for Hash {
    fn from(value: &[u8]) -> Self {
        Self { hash: value.into() }
    }
}
pub enum Error {
    HashingMethodNotFound { id: HashingMethodID },
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HashingMethodNotFound { id } => {
                write!(f, "Hashing method not found with id {}", id)
            }
        }
    }
}
