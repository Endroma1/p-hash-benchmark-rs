use std::{
    fmt::Display,
    path::{Path, PathBuf},
    slice::Iter,
    str::FromStr,
    sync::{Arc, OnceLock, mpsc},
};

use crate::{
    db::DB,
    img_hash::{HashingMethodID, HashingMethods, hash_images},
    img_mod::{ModificationID, Modifications, modify_image, open_image},
    img_proc::{ImageReadMessage, Images},
};
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use rayon::prelude::*;
use sqlx::{
    ConnectOptions, SqlitePool,
    sqlite::{self, SqliteConnectOptions, SqlitePoolOptions},
};

mod db;
mod img_hash;
mod img_mod;
mod img_proc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::debug!("starting phash");
    let test_path = Path::new("/home/endro/.local/share/p-hash/images/");

    log::debug!("Creating db ...");
    DB::create_db().await?;

    let app = App::new(test_path);
    app.run().await?;
    Ok(())
}

struct PHashResult {
    img_id: u32,
    mod_imgs: ModifiedImages,
    hashes: Hashes,
}

impl PHashResult {
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
pub fn get_modifications() -> &'static Modifications {
    static MODIFICATIONS: OnceLock<Modifications> = OnceLock::new();
    MODIFICATIONS.get_or_init(|| Modifications::new())
}
pub fn get_hashing_methods() -> &'static HashingMethods {
    static HASHING_METHODS: OnceLock<HashingMethods> = OnceLock::new();
    HASHING_METHODS.get_or_init(|| HashingMethods::new())
}
struct App {
    images: Vec<img_proc::Image>,
    modifications: &'static Modifications,
    hashing_methods: &'static HashingMethods,
}
impl App {
    pub fn new(path: &Path) -> Self {
        let images = Images::from_path(path.to_path_buf());
        let images = images
            .filter_map(|r| {
                if let Err(e) = r.as_ref() {
                    log::warn!("A image failed to process: {}", e);
                }
                r.ok()
            })
            .collect();

        let modifications = get_modifications();
        let hashing_methods = get_hashing_methods();
        Self {
            images,
            modifications,
            hashing_methods,
        }
    }
    pub async fn run(&self) -> Result<(), Error> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite:data.db")?.create_if_missing(true),
        )
        .await?;

        log::info!("starting image hashing");
        let res: Vec<PHashResult> = self
            .images
            .par_iter()
            .progress_with(ProgressBar::new(self.images.len() as u64))
            .enumerate()
            .map(move |(id, image)| {
                let mut app_proc = AppProcess::new();
                let res = app_proc.run(image.get_path());
                if let Err(e) = res {
                    log::error!("app proc failed with error {}", e);
                }
                let proc_res = app_proc.finish();
                PHashResult {
                    img_id: id as u32,
                    mod_imgs: proc_res.mod_imgs,
                    hashes: proc_res.hashes,
                }
            })
            .collect();

        log::info!("sending results to db");
        self.send_to_db(res, &pool).await?;
        Ok(())
    }
    async fn send_to_db(&self, results: Vec<PHashResult>, pool: &SqlitePool) -> Result<(), Error> {
        log::debug!("sending modifications to db");
        self.modifications.send_to_db(pool).await?;
        log::debug!("sending hashing_methods to db");
        self.hashing_methods.send_to_db(pool).await?;

        log::debug!("results len : {}", results.len());
        log::debug!("sending results to db");
        for result in results {
            let image = self
                .images
                .get(result.img_id as usize)
                .ok_or(Error::ImageNotFound {
                    id: result.img_id as usize,
                })?;

            let path_str = image.get_path().to_string_lossy().to_string();
            log::debug!("inserting image with id {}", result.img_id);
            sqlx::query(
                "
            INSERT INTO images (id, path, user) VALUES (?, ?, ?) ON CONFLICT(id) DO NOTHING;
            ",
            )
            .bind(result.img_id)
            .bind(path_str)
            .bind(image.get_user())
            .execute(pool)
            .await?;

            result.send_to_db(pool).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct AppProcessResult {
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
}
#[derive(Debug, Default)]
struct ModifiedImages {
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
impl<'a> IntoIterator for &'a ModifiedImages {
    type Item = &'a img_mod::ModifiedImage;
    type IntoIter = std::slice::Iter<'a, img_mod::ModifiedImage>;
    fn into_iter(self) -> Self::IntoIter {
        self.images.iter()
    }
}
#[derive(Debug)]
struct Hashes {
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
struct Hash {
    mod_img_id: u32,
    hash: img_hash::HashResult,
}
#[derive(Debug, Default)]
struct AppProcess {
    result: AppProcessResult,
}
impl AppProcess {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn finish(self) -> AppProcessResult {
        self.result
    }
    pub fn run(&mut self, img_path: &Path) -> Result<(), Error> {
        let modified_images = Self::modify_image(img_path)?;
        self.result.set_mod_imgs(modified_images);

        let ids = 0..self.result.mod_imgs.images.len();

        for id in ids {
            if let Err(e) = self.hash_image(id as u32) {
                log::error!("failed to hash an image {e}")
            }
        }
        Ok(())
    }
    fn modify_image(img_path: &Path) -> Result<ModifiedImages, Error> {
        let modifications = Modifications::new();
        let mod_ids = modifications.get_keys();
        let modified_images = match modify_image(img_path, mod_ids) {
            Ok(v) => v,
            Err(e) => {
                return Err(e.into());
            }
        };

        let modified_images = modified_images.filter_map(|r| {
            if let Err(e) = r.as_ref() {
                log::warn!("could not modify a image: {}", e);
            }
            r.ok()
        });
        Ok(ModifiedImages {
            images: modified_images.collect(),
        })
    }
    fn hash_image(&mut self, mod_img_id: u32) -> Result<(), Error> {
        let hashing_methods = HashingMethods::new();

        let hashing_method_ids = hashing_methods.get_keys();

        let modified_image = self.result.mod_imgs.get_img_mut(mod_img_id)?;

        let img = modified_image.get_img().ok_or(Error::ImageHandleClosed)?;

        hash_images(img.clone(), hashing_method_ids)
            .filter_map(|r| {
                if let Err(e) = r.as_ref() {
                    log::warn!("could not hash an image: {}", e);
                }
                r.ok()
            })
            .for_each(|h| {
                self.result.hashes.insert_hash(Hash {
                    mod_img_id,
                    hash: h,
                });
            });

        modified_image.close_img();
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    ImageProc { err: crate::img_proc::Error },
    ImageMod { err: crate::img_mod::Error },
    ModificationNotFound { id: usize },
    HashingMethodNotFound { id: usize },
    ImageNotFound { id: usize },
    ImageHandleClosed,
    Sqlx { err: sqlx::Error },
}
impl From<img_mod::Error> for Error {
    fn from(value: crate::img_mod::Error) -> Self {
        Self::ImageMod { err: value }
    }
}
impl From<img_proc::Error> for Error {
    fn from(value: crate::img_proc::Error) -> Self {
        Self::ImageProc { err: value }
    }
}
impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx { err: value }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImageProc { err } => write!(f, "Image proc error: {}", err),
            Self::ImageMod { err } => write!(f, "Image modification errer: {}", err),
            Self::ModificationNotFound { id } => write!(f, "Modification with id {} not found", id),
            Self::HashingMethodNotFound { id } => {
                write!(f, "Hashing method with id {} not found", id)
            }
            Self::ImageNotFound { id } => write!(f, "Image with id {} not found", id),
            Self::ImageHandleClosed => write!(f, "Image handle closed before expected"),
            Self::Sqlx { err } => write!(f, "Sqlx Error: {}", err),
        }
    }
}
impl std::error::Error for Error {}
