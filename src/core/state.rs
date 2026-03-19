use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, mpsc},
};

use sqlx::SqlitePool;

use crate::{
    core::{
        error::Error,
        images_processor::{PHashResult, PHashResults},
    },
    image_hash::{self, HashingMethods},
    image_modify::{self, Modifications},
    image_parse,
};
pub struct AppState {
    handler: mpsc::Sender<Message>,
}
impl AppState {
    pub fn new(hashes: HashingMethods, modifications: Modifications) -> Self {
        let handler = AppStateBuilder::new(hashes, modifications).spawn_app_state();
        Self { handler }
    }
    pub fn hashing_methods(&self) -> Arc<HashingMethods> {
        let (tx, rx) = oneshot::channel();
        self.handler.send(Message::HashingMethods(tx));
        let res = rx.recv().unwrap();
        res
    }
    pub fn modifications(&self) -> Arc<Modifications> {
        let (tx, rx) = oneshot::channel();
        self.handler.send(Message::Modifications(tx));
        let res = rx.recv().unwrap();
        res
    }
}

pub struct AppStateBuilder {
    hashes: Arc<HashingMethods>,
    modifications: Arc<Modifications>,
}
impl AppStateBuilder {
    pub fn new(hashes: HashingMethods, modifications: Modifications) -> Self {
        Self {
            hashes: Arc::new(hashes),
            modifications: Arc::new(modifications),
        }
    }

    /// Spawns app state and returns handler
    pub fn spawn_app_state(self) -> mpsc::Sender<Message> {
        let (tx, rx) = mpsc::channel();
        rayon::spawn(move || {
            while let Ok(m) = rx.recv() {
                match m {
                    Message::HashingMethods(r) => {
                        if r.send(Arc::clone(&self.hashes)).is_err() {
                            break;
                        };
                    }
                    Message::Modifications(r) => {
                        if r.send(Arc::clone(&self.modifications)).is_err() {
                            break;
                        };
                    }
                }
            }
        });
        tx
    }
}
pub enum Message {
    HashingMethods(oneshot::Sender<Arc<HashingMethods>>),
    Modifications(oneshot::Sender<Arc<Modifications>>),
}

#[derive(Debug)]
pub struct Hashes {
    hashes: Vec<Hash>,
}
impl Hashes {
    pub fn insert_hash(&mut self, hash: Hash) -> usize {
        let index = self.hashes.len();
        self.hashes.insert(index, hash);
        index
    }
}
impl Default for Hashes {
    fn default() -> Self {
        Self { hashes: Vec::new() }
    }
}
impl<'a> IntoIterator for &'a Hashes {
    type Item = &'a Hash;
    type IntoIter = std::slice::Iter<'a, Hash>;
    fn into_iter(self) -> Self::IntoIter {
        self.hashes.iter()
    }
}
impl FromIterator<Hash> for Hashes {
    fn from_iter<T: IntoIterator<Item = Hash>>(iter: T) -> Self {
        Self {
            hashes: iter.into_iter().collect(),
        }
    }
}
#[derive(Debug)]
pub struct Hash {
    mod_img_id: u32,
    hash: image_hash::HashResult,
}
impl Hash {
    pub fn new(mod_img_id: u32, hash: image_hash::HashResult) -> Self {
        Self { mod_img_id, hash }
    }
    pub fn mod_img_id(&self) -> u32 {
        self.mod_img_id
    }
    pub fn hash(&self) -> &image_hash::HashResult {
        &self.hash
    }
}

#[derive(Default)]
pub struct AppProcessResult {
    imgs: Images,
    phash_results: PHashResults,
}
impl AppProcessResult {
    pub fn new(imgs: Images, res: PHashResults) -> Self {
        Self {
            imgs,
            phash_results: res,
        }
    }
    pub fn phash_results(&self) -> &PHashResults {
        &self.phash_results
    }
    pub fn images(&self) -> &Images {
        &self.imgs
    }
    pub fn get_results(&self) -> Vec<(u32, &image_parse::Image, &PHashResult)> {
        let mut results = Vec::new();
        for (id, img) in self.imgs.iter().enumerate() {
            let res = self
                .phash_results
                .get(&(id as u32))
                .expect("invalid id found");
            results.push((id as u32, img, res));
        }
        results
    }
    pub async fn send_to_db(&self, pool: &SqlitePool) -> Result<(), Error> {
        let mut tx = pool.begin().await?;
        for (id, img) in self.imgs.iter().enumerate() {
            let path_str = img.get_path().to_string_lossy().to_string();
            sqlx::query(
                "
            INSERT INTO images (id, path, user) VALUES (?, ?, ?) ON CONFLICT(id) DO NOTHING;
            ",
            )
            .bind(id as u32)
            .bind(path_str)
            .bind(img.get_user())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.phash_results.send_to_db(pool).await?;
        Ok(())
    }
}
#[derive(Debug, Default)]
pub struct Images {
    imgs: Vec<image_parse::Image>,
}
impl Deref for Images {
    type Target = Vec<image_parse::Image>;
    fn deref(&self) -> &Self::Target {
        &self.imgs
    }
}
impl From<Vec<image_parse::Image>> for Images {
    fn from(value: Vec<image_parse::Image>) -> Self {
        Self { imgs: value }
    }
}
#[derive(Debug)]
pub struct ModifiedImage {
    img_id: u32,
    mod_img: image_modify::ModifiedImage,
}
impl ModifiedImage {
    pub fn new(img_id: u32, mod_img: image_modify::ModifiedImage) -> Self {
        Self { img_id, mod_img }
    }
}
impl Deref for ModifiedImage {
    type Target = image_modify::ModifiedImage;
    fn deref(&self) -> &Self::Target {
        &self.mod_img
    }
}
impl DerefMut for ModifiedImage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mod_img
    }
}
impl ModifiedImage {
    pub fn img_id(&self) -> u32 {
        self.img_id
    }
}
