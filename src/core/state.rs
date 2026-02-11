use std::ops::Deref;

use sqlx::SqlitePool;

use crate::{core::error::Error, img_hash, img_mod};

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
    hash: img_hash::HashResult,
}
impl Hash {
    pub fn new(mod_img_id: u32, hash: img_hash::HashResult) -> Self {
        Self { mod_img_id, hash }
    }
}

pub struct PHashResult {
    img_id: u32,
    mod_imgs: ModifiedImages,
    hashes: Hashes,
}

impl PHashResult {
    pub fn img_id(&self) -> u32 {
        self.img_id
    }
    pub fn mod_imgs(&self) -> &ModifiedImages {
        &self.mod_imgs
    }
    pub fn hashes(&self) -> &Hashes {
        &self.hashes
    }
    pub fn new(img_id: u32, mod_imgs: ModifiedImages, hashes: Hashes) -> Self {
        Self {
            img_id,
            mod_imgs,
            hashes,
        }
    }
    pub async fn send_to_db(&self, pool: &SqlitePool) -> Result<(), Error> {
        let mut tx = pool.begin().await?;

        for hash in self.hashes.into_iter() {
            let img = self.mod_imgs.get_img(hash.mod_img_id)?;

            let res = sqlx::query(
                "
        INSERT INTO modified_images ( image_id, modification_id) VALUES (?,?);
            ",
            )
            .bind(self.img_id)
            .bind(img.get_mod_id())
            .execute(&mut *tx)
            .await?;

            let mod_img_id = res.last_insert_rowid();

            sqlx::query(
                "
                INSERT INTO hashes (mod_image_id, hashing_method_id) VALUES (?,?);
                ",
            )
            .bind(mod_img_id)
            .bind(hash.hash.hashing_method_id())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AppProcessResult {
    mod_imgs: ModifiedImages,
    hashes: Hashes,
}
impl AppProcessResult {
    pub fn set_mod_imgs(&mut self, val: ModifiedImages) {
        self.mod_imgs = val
    }
    pub fn set_hashes(&mut self, val: Hashes) {
        self.hashes = val
    }
    pub fn mod_imgs(&self) -> &ModifiedImages {
        &self.mod_imgs
    }
    pub fn mod_imgs_mut(&mut self) -> &mut ModifiedImages {
        &mut self.mod_imgs
    }

    pub fn hashes(&self) -> &Hashes {
        &self.hashes
    }
    pub fn hashes_mut(&mut self) -> &mut Hashes {
        &mut self.hashes
    }
    pub fn into_mod_imgs(self) -> ModifiedImages {
        self.mod_imgs
    }
    pub fn into_hashes(self) -> Hashes {
        self.hashes
    }
    pub fn into_parts(self) -> (ModifiedImages, Hashes) {
        (self.mod_imgs, self.hashes)
    }
}
#[derive(Debug, Default)]
pub struct ModifiedImages {
    images: Vec<img_mod::ModifiedImage>,
}
impl ModifiedImages {
    pub fn get_img(&self, id: u32) -> Result<&img_mod::ModifiedImage, Error> {
        self.images
            .get(id as usize)
            .ok_or(Error::ModificationNotFound { id: id as usize })
    }
    pub fn get_img_mut(&mut self, id: u32) -> Result<&mut img_mod::ModifiedImage, Error> {
        self.images
            .get_mut(id as usize)
            .ok_or(Error::ModificationNotFound { id: id as usize })
    }
}
impl Deref for ModifiedImages {
    type Target = Vec<img_mod::ModifiedImage>;
    fn deref(&self) -> &Self::Target {
        &self.images
    }
}
impl From<Vec<img_mod::ModifiedImage>> for ModifiedImages {
    fn from(value: Vec<img_mod::ModifiedImage>) -> Self {
        Self { images: value }
    }
}
impl<'a> IntoIterator for &'a ModifiedImages {
    type Item = &'a img_mod::ModifiedImage;
    type IntoIter = std::slice::Iter<'a, img_mod::ModifiedImage>;
    fn into_iter(self) -> Self::IntoIter {
        self.images.iter()
    }
}
