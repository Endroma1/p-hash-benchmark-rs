use image::DynamicImage;
use img_hash::HasherConfig;

use super::HashingMethod;
use crate::image_hash::Hash;

pub struct AverageHash {
    size: u32,
}
impl AverageHash {
    pub fn new(size: u32) -> Self {
        Self { size }
    }
}
impl HashingMethod for AverageHash {
    fn hash(&self, img: &DynamicImage) -> Hash {
        let hasher = HasherConfig::new()
            .hash_alg(img_hash::HashAlg::Mean)
            .hash_size(self.size, self.size)
            .to_hasher();
        let res = hasher.hash_image(img);
        res.as_bytes().into()
    }
    fn name(&self) -> String {
        format!("average_hash{}", self.size)
    }
}

pub struct VertGradient {
    size: u32,
}
impl VertGradient {
    pub fn new(size: u32) -> Self {
        Self { size }
    }
}
impl HashingMethod for VertGradient {
    fn hash(&self, img: &DynamicImage) -> Hash {
        let hasher = HasherConfig::new()
            .hash_alg(img_hash::HashAlg::VertGradient)
            .hash_size(self.size, self.size)
            .to_hasher();
        let res = hasher.hash_image(img);
        res.as_bytes().into()
    }
    fn name(&self) -> String {
        format!("vert_gradient{}", self.size)
    }
}
pub struct Gradient {}
impl Gradient {
    pub fn new() -> Self {
        Self {}
    }
}
impl HashingMethod for Gradient {
    fn hash(&self, img: &DynamicImage) -> Hash {
        let hasher = HasherConfig::new()
            .hash_alg(img_hash::HashAlg::Gradient)
            .to_hasher();
        let res = hasher.hash_image(img);
        res.as_bytes().into()
    }
    fn name(&self) -> String {
        "vert_gradient".to_string()
    }
}
