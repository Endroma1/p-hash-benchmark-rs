use image::DynamicImage;

use super::ImageModification;
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
    fn name(&self) -> &str {
        "blur"
    }
}

//-------------------------------------------------------

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

//-------------------------------------------------------
