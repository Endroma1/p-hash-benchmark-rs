use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use super::Error;
use image::DynamicImage;
use image::io::Reader as ImageReader;

#[derive(Default)]
pub struct Modifications {
    methods: Vec<Box<dyn ImageModification>>,
}
impl Deref for Modifications {
    type Target = Vec<Box<dyn ImageModification>>;
    fn deref(&self) -> &Self::Target {
        &self.methods
    }
}
impl DerefMut for Modifications {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.methods
    }
}

impl Modifications {
    pub fn new() -> Self {
        Modifications::default()
    }
    pub fn push(&mut self, modification: impl ImageModification + 'static) {
        self.methods.push(Box::new(modification));
    }
}

pub trait ImageModification: Send + Sync {
    fn apply(&self, img: &DynamicImage) -> DynamicImage;
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct ModifiedImage {
    mod_id: u16,
    img: Option<image::DynamicImage>,
}
impl ModifiedImage {
    fn new(mod_img: image::DynamicImage, mod_id: u16) -> Self {
        Self {
            img: Some(mod_img),
            mod_id,
        }
    }
    pub fn get_img(&self) -> Option<&DynamicImage> {
        self.img.as_ref()
    }
    pub fn get_mod_id(&self) -> u16 {
        self.mod_id
    }
    pub fn close_img(&mut self) {
        self.img = None
    }
}

pub fn modify_image<'a>(
    path: &Path,
    modifications: &'a Modifications,
) -> Result<impl Iterator<Item = ModifiedImage> + 'a, Error> {
    // Modifies image with the modifications that matches the ids.
    let img = ImageReader::open(path)?.decode()?;

    let modified_images = modifications
        .iter()
        .enumerate()
        .map(move |(id, modification)| {
            let mod_img = modification.apply(&img);
            ModifiedImage::new(mod_img, id as u16)
        });
    Ok(modified_images)
}
