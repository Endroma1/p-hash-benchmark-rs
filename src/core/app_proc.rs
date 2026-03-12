use std::path::Path;

use crate::{
    core::{
        error::Error,
        images_processor::PHashResult,
        state::{self, Hash, ModifiedImage},
    },
    image_hash::{HashingMethods, hash_images},
    image_modify::{Modifications, ModifiedImages, modify_image},
};

#[derive(Default)]
pub struct AppProcess {
    result: PHashResult,
}
impl AppProcess {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn finish(self) -> PHashResult {
        self.result
    }
    pub fn run(
        &mut self,
        img_path: &Path,
        img_id: u32,
        modifications: &Modifications,
        hashing_methods: &HashingMethods,
    ) -> Result<(), Error> {
        let modified_images = Self::modify_image(img_path, img_id, modifications)?;
        self.result.set_mod_imgs(modified_images);

        let ids = 0..self.result.mod_imgs().len();

        for id in ids {
            if let Err(e) = self.hash_image(id as u32, &hashing_methods) {
                tracing::error!("failed to hash an image {e}")
            }
        }
        Ok(())
    }
    fn modify_image(
        img_path: &Path,
        img_id: u32,
        modifications: &Modifications,
    ) -> Result<ModifiedImages, Error> {
        let modified_images = modify_image(img_path, modifications)?;

        let mod_imgs_state = modified_images.map(|i| ModifiedImage::new(img_id, i));

        Ok(ModifiedImages::from(
            mod_imgs_state.collect::<Vec<state::ModifiedImage>>(),
        ))
    }
    fn hash_image(
        &mut self,
        mod_img_id: u32,
        hashing_methods: &HashingMethods,
    ) -> Result<(), Error> {
        let img = {
            let modified_image = self.result.mod_imgs().get_img(mod_img_id)?;

            modified_image.get_img().ok_or(Error::ImageHandleClosed)?
        };

        hash_images(img.clone(), hashing_methods).for_each(|r| {
            self.result
                .hashes_mut()
                .insert_hash(Hash::new(mod_img_id, r));
        });

        self.result
            .mod_imgs_mut()
            .get_img_mut(mod_img_id)?
            .close_img();
        Ok(())
    }
}
