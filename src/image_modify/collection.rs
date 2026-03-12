use std::ops::Deref;

use crate::{core::state::ModifiedImage, image_modify::Error};

#[derive(Debug, Default)]
pub struct ModifiedImages {
    images: Vec<ModifiedImage>,
}
impl ModifiedImages {
    pub fn get_img(&self, id: u32) -> Result<&ModifiedImage, Error> {
        self.images
            .get(id as usize)
            .ok_or(Error::ModificationNotFound { id: id as usize })
    }
    pub fn get_img_mut(&mut self, id: u32) -> Result<&mut ModifiedImage, Error> {
        self.images
            .get_mut(id as usize)
            .ok_or(Error::ModificationNotFound { id: id as usize })
    }
}
impl Deref for ModifiedImages {
    type Target = Vec<ModifiedImage>;
    fn deref(&self) -> &Self::Target {
        &self.images
    }
}
impl From<Vec<ModifiedImage>> for ModifiedImages {
    fn from(value: Vec<ModifiedImage>) -> Self {
        Self { images: value }
    }
}
impl<'a> IntoIterator for &'a ModifiedImages {
    type Item = &'a ModifiedImage;
    type IntoIter = std::slice::Iter<'a, ModifiedImage>;
    fn into_iter(self) -> Self::IntoIter {
        self.images.iter()
    }
}
