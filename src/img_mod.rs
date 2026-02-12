use std::{collections::HashMap, fmt::Display, path::Path};

use image::{
    DynamicImage,
    imageops::{blur, unsharpen},
};
use image::{imageops::rotate180, io::Reader as ImageReader};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct ModifiedImage {
    mod_id: ModificationID,
    img: Option<image::DynamicImage>,
}
impl ModifiedImage {
    fn try_new(img: image::DynamicImage, mod_id: ModificationID) -> Result<Self, Error> {
        let modifications = Modifications::new();
        let modification = modifications
            .get_modification(mod_id)
            .ok_or(Error::ModificationNotFound { id: mod_id })?;
        let mod_img = modification.modify(&img);
        Ok(Self {
            img: Some(mod_img),
            mod_id,
        })
    }
    pub fn get_img(&self) -> Option<&DynamicImage> {
        self.img.as_ref()
    }
    pub fn get_mod_id(&self) -> ModificationID {
        self.mod_id
    }
    pub fn close_img(&mut self) {
        self.img = None
    }
}

pub fn modify_image(
    path: &Path,
    ids: Vec<u16>,
) -> Result<impl Iterator<Item = Result<ModifiedImage, Error>>, Error> {
    // Modifies image with the modifications that matches the ids.
    let img = ImageReader::open(path)?.decode()?;
    let modified_imgs = ids
        .into_iter()
        .map(move |id| ModifiedImage::try_new(img.clone(), id.clone()));

    Ok(modified_imgs)
}
pub fn open_image(path: &Path) -> Result<DynamicImage, Error> {
    let img = ImageReader::open(path)?.decode()?;
    Ok(img)
}
pub type ModificationID = u16;

#[derive(Default)]
pub struct Modifications {
    modifications: HashMap<ModificationID, Box<dyn Modification>>,
}

impl Modifications {
    pub fn new() -> Self {
        let mut mods = Vec::new();
        mods.push(Box::new(GaussianBlur::new(1.0)) as Box<dyn Modification>);
        mods.push(Box::new(Rotate::new()) as Box<dyn Modification>);
        mods.push(Box::new(UnSharpen::new()) as Box<dyn Modification>);

        let mut modifications = HashMap::new();
        for (i, m) in mods.into_iter().enumerate() {
            modifications.insert(i as u16, m);
        }
        Self { modifications }
    }
    pub fn get_methods(&self) -> impl Iterator<Item = &dyn Modification> {
        self.modifications.values().map(|m| &**m)
    }
    pub fn get_keys(&self) -> Vec<u16> {
        self.modifications.keys().copied().collect()
    }
    pub fn get_modification(&self, id: u16) -> Option<&dyn Modification> {
        self.modifications.get(&id).map(|m| &**m)
    }
    pub async fn send_to_db(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        for (id, obj) in &self.modifications {
            sqlx::query(
                "
                INSERT INTO modifications (id, name) VALUES (?, ?) ON CONFLICT(id) DO NOTHING;
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
pub trait Modification: Send + Sync {
    fn modify(&self, image: &DynamicImage) -> DynamicImage;
    fn name(&self) -> &str;
}

struct GaussianBlur {
    sigma: f32,
    name: String,
}
impl GaussianBlur {
    pub fn new(sigma: f32) -> Self {
        Self {
            sigma,
            name: "gaussian_blur".to_string(),
        }
    }
}
impl Modification for GaussianBlur {
    fn modify(&self, image: &DynamicImage) -> DynamicImage {
        let buff = blur(image, self.sigma);
        DynamicImage::ImageRgba8(buff)
    }
    fn name(&self) -> &str {
        &self.name
    }
}
struct Rotate {
    name: String,
}
impl Rotate {
    fn new() -> Self {
        Self {
            name: "rotate".to_string(),
        }
    }
}
impl Modification for Rotate {
    fn modify(&self, image: &DynamicImage) -> DynamicImage {
        let buff = rotate180(image);
        DynamicImage::ImageRgba8(buff)
    }
    fn name(&self) -> &str {
        &self.name
    }
}
struct UnSharpen {
    name: String,
}
impl UnSharpen {
    fn new() -> Self {
        Self {
            name: "unsharpen".to_string(),
        }
    }
}
impl Modification for UnSharpen {
    fn modify(&self, image: &DynamicImage) -> DynamicImage {
        let buff = unsharpen(image, 1.0, 2);
        DynamicImage::ImageRgba8(buff)
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub enum Error {
    Image { err: image::ImageError },
    IO { err: std::io::Error },
    ModificationNotFound { id: u16 },
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO { err: value }
    }
}
impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::Image { err: value }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image { err } => write!(f, "Image error: {}", err),
            Self::IO { err } => write!(f, "IO Error: {}", err),
            Self::ModificationNotFound { id } => {
                write!(f, "Modification not found with id: {}", id)
            }
        }
    }
}
