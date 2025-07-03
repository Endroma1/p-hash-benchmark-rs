use std::path::PathBuf;

use error::ModifyError;
use image::DynamicImage;
use modification::{ImageModification, get_modification};

pub struct ModifyImage<'a> {
    pub img: DynamicImage,
    pub modification_name: &'a str,
}

impl<'a> ModifyImage<'a> {
    pub fn new(
        img: &'a DynamicImage,
        modification_name: &'a str,
    ) -> Result<ModifyImage<'a>, ModifyError> {
        let mod_img = modify_img(&img, modification_name)?;

        Ok(ModifyImage {
            img: mod_img,
            modification_name,
        })
    }

    pub fn save(&self, save_path: PathBuf) -> Result<(), ModifyError> {
        Ok(self.img.save(save_path)?)
    }
}

fn modify_img(img: &DynamicImage, modification_name: &str) -> Result<DynamicImage, ModifyError> {
    Ok(get_modification(modification_name).map(|m| m.apply(img))?)
}

mod error {

    pub enum ModifyError {
        UnknownModification(String),
        ImageError(image::ImageError),
    }

    impl From<image::ImageError> for ModifyError {
        fn from(e: image::ImageError) -> Self {
            ModifyError::ImageError(e)
        }
    }

    impl std::fmt::Display for ModifyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ModifyError::UnknownModification(name) => {
                    write!(f, "Unknown modification {}", name)
                }
                ModifyError::ImageError(e) => write!(f, "Image error {}", e),
            }
        }
    }
}

mod modification {
    use super::DynamicImage;
    use super::error::ModifyError;

    pub fn get_modification(name: &str) -> Result<Modification, ModifyError> {
        match name {
            "blur" => Ok(Modification::Blur(blur::Blur::default())),
            "rotate" => Ok(Modification::Rotate(rotate::Angle::Rot90)),
            _ => Err(ModifyError::UnknownModification(name.to_string())),
        }
    }

    pub trait ImageModification {
        fn apply(&self, img: &DynamicImage) -> DynamicImage;
        fn name(&self) -> &str;
    }

    pub enum Modification {
        Blur(blur::Blur),
        Rotate(rotate::Angle),
    }

    impl ImageModification for Modification {
        fn apply(&self, img: &DynamicImage) -> DynamicImage {
            match self {
                Modification::Blur(b) => b.apply(img),
                Modification::Rotate(r) => r.apply(img),
            }
        }
        fn name(&self) -> &str {
            match self {
                Modification::Blur(b) => b.name(),
                Modification::Rotate(r) => r.name(),
            }
        }
    }

    mod blur {
        use super::{DynamicImage, ImageModification};

        pub struct Blur {
            sigma: f32,
            name: &'static str,
        }

        impl Default for Blur {
            fn default() -> Self {
                Self {
                    sigma: 0.9,
                    name: "blur",
                }
            }
        }

        impl ImageModification for Blur {
            fn apply(&self, img: &DynamicImage) -> DynamicImage {
                img.blur(self.sigma)
            }
            fn name(&self) -> &str {
                self.name
            }
        }
    }

    mod rotate {
        use super::{DynamicImage, ImageModification};

        pub enum Angle {
            Rot90,
            Rot180,
            Rot270,
        }

        impl ImageModification for Angle {
            fn apply(&self, img: &DynamicImage) -> DynamicImage {
                match self {
                    Angle::Rot90 => img.rotate90(),
                    Angle::Rot180 => img.rotate180(),
                    Angle::Rot270 => img.rotate270(),
                }
            }
            fn name(&self) -> &str {
                match self {
                    Angle::Rot90 => "rotate90",
                    Angle::Rot180 => "rotate180",
                    Angle::Rot270 => "rotate270",
                }
            }
        }
    }
}
