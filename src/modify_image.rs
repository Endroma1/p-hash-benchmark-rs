use error::ModifyError;
use image::DynamicImage;
use modification::{ImageModification, get_modification};
use std::path::PathBuf;

pub trait ModifyProcess {
    fn modify_img(&self) -> Result<(), ModifyError>;
}

pub struct ModifyImage<'a> {
    pub img: DynamicImage,
    pub modification_name: &'a str,
    pub save_path: PathBuf,
}

impl<'a> ModifyProcess for ModifyImage<'a> {
    fn modify_img(&self) -> Result<(), ModifyError> {
        let mod_img = get_modification(self.modification_name).map(|m| m.apply(&self.img))?;
        mod_img.save(&self.save_path)?;
        Ok(())
    }
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
