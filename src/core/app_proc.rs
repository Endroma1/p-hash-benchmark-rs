use std::path::Path;

use crate::{
    core::{
        error::Error,
        state::{AppProcessResult, Hash, ModifiedImages},
    },
    img_hash::{HashingMethods, hash_images},
    img_mod::{self, Modifications, modify_image},
};

#[derive(Debug, Default)]
pub struct AppProcess {
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

        let ids = 0..self.result.mod_imgs().len();

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
        Ok(ModifiedImages::from(
            modified_images.collect::<Vec<img_mod::ModifiedImage>>(),
        ))
    }
    fn hash_image(&mut self, mod_img_id: u32) -> Result<(), Error> {
        let hashing_methods = HashingMethods::new();

        let hashing_method_ids = hashing_methods.get_keys();

        let img = {
            let modified_image = self.result.mod_imgs().get_img(mod_img_id)?;

            modified_image.get_img().ok_or(Error::ImageHandleClosed)?
        };

        hash_images(img.clone(), hashing_method_ids)
            .filter_map(|r| {
                if let Err(e) = r.as_ref() {
                    log::warn!("could not hash an image: {}", e);
                }
                r.ok()
            })
            .for_each(|h| {
                self.result
                    .hashes_mut()
                    .insert_hash(Hash::new(mod_img_id, h));
            });

        self.result
            .mod_imgs_mut()
            .get_img_mut(mod_img_id)?
            .close_img();
        Ok(())
    }
}
