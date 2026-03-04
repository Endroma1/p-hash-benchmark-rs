use image::DynamicImage;
use img_hash::HasherConfig;

use super::HashingMethod;
use crate::image_hash::Hash;

pub struct AverageHash {}
impl AverageHash {
    pub fn new() -> Self {
        Self {}
    }
}
impl HashingMethod for AverageHash {
    fn hash(&self, img: &DynamicImage) -> Hash {
        let hasher = HasherConfig::new()
            .hash_alg(img_hash::HashAlg::Mean)
            .to_hasher();
        let res = hasher.hash_image(img);
        res.as_bytes().into()
    }
    fn name(&self) -> &str {
        "average_hash"
    }
}

pub struct VertGradient {}
impl VertGradient {
    pub fn new() -> Self {
        Self {}
    }
}
impl HashingMethod for VertGradient {
    fn hash(&self, img: &DynamicImage) -> Hash {
        let hasher = HasherConfig::new()
            .hash_alg(img_hash::HashAlg::VertGradient)
            .to_hasher();
        let res = hasher.hash_image(img);
        res.as_bytes().into()
    }
    fn name(&self) -> &str {
        "vert_gradient"
    }
}
