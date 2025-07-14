// Currently a lot of boilerplate. Needs to be macroed
use std::collections::HashMap;

use error::ModifyError;
use image::DynamicImage;
use modification::{Blur, Modification};

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

#[derive(Default)]
struct Modifications {
    methods: HashMap<&'static str, Modification>,
}

impl Modifications {
    fn new() -> Self {
        Modifications::default()
    }
    fn add(mut self, name: &'static str, modification: Modification) -> Self {
        self.methods.insert(name, modification);
        self
    }

    fn load() -> Self {
        Modifications::new().add("blur", Modification::Blur(Blur::default()))
    }

    fn get(&self, name: &str) -> Result<&Modification, ModifyError> {
        let modification = self.methods.get(name);

        match modification {
            Some(m) => Ok(m),
            None => Err(ModifyError::UnknownModification(format!(
                "Failed to get modification {}",
                name
            ))),
        }
    }
}

pub mod modification {

    use super::DynamicImage;

    pub enum Modification {
        Blur(Blur),
        Rotate(Rotate),
    }

    impl Modification {
        fn apply(&self, img: &DynamicImage) -> DynamicImage {
            match self {
                Modification::Blur(blur) => blur.apply(img),
                Modification::Rotate(rotate) => rotate.apply(img),
            }
        }
    }

    pub trait ImageModification: Default {
        fn apply(&self, img: &DynamicImage) -> DynamicImage;

        fn new() -> Self {
            Self::default()
        }
    }

    //------------------------------------------------------

    pub struct Blur {
        sigma: f32,
    }

    impl Default for Blur {
        fn default() -> Self {
            Self { sigma: 0.9 }
        }
    }

    impl ImageModification for Blur {
        fn apply(&self, img: &DynamicImage) -> DynamicImage {
            img.blur(self.sigma)
        }
    }

    //-------------------------------------------------------

    pub struct Rotate {
        angle: Angle,
    }

    impl Default for Rotate {
        fn default() -> Self {
            Rotate {
                angle: Angle::Rot90,
            }
        }
    }

    enum Angle {
        Rot90,
        Rot180,
        Rot270,
    }

    impl ImageModification for Rotate {
        fn apply(&self, img: &DynamicImage) -> DynamicImage {
            match self.angle {
                Angle::Rot90 => img.rotate90(),
                Angle::Rot180 => img.rotate180(),
                Angle::Rot270 => img.rotate270(),
            }
        }
    }

    //-------------------------------------------------------
}
