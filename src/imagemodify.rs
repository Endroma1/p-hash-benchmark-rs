use image::DynamicImage;

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

pub mod modification {
    use super::DynamicImage;

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
