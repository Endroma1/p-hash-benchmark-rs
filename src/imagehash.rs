use bitvec::prelude::*;
use image::{DynamicImage, imageops::FilterType::Lanczos3};

pub struct Hash {
    bits: BitVec,
}

impl Hash {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut bits = self.bits.clone();
        let rem = bits.len() % 8;
        if rem != 0 {
            bits.resize(bits.len() + (8 - rem), false)
        }

        bits.chunks(8)
            .map(|c| c.iter().fold(0u8, |acc, bit| (acc << 1) | (*bit as u8)))
            .collect()
    }

    pub fn to_hex(self) -> String {
        let bytes = self.to_bytes();
        hex::encode(bytes)
    }
}

pub trait HashMethod: Default {
    fn new() -> Self {
        Self::default()
    }

    fn hash(self, img: DynamicImage) -> Hash;
}

pub struct AverageHash {
    img_size: (usize, usize),
}

impl HashMethod for AverageHash {
    fn hash(self, img: DynamicImage) -> Hash {
        let resized = img.resize(self.img_size.0 as u32, self.img_size.1 as u32, Lanczos3);
        let grayscaled = resized.to_luma8();

        let (width, height) = grayscaled.dimensions();
        let total_pixels = (height * width) as f32;

        let total_intensity: f32 = grayscaled.pixels().map(|p| p[0] as f32).sum();

        let average = total_intensity / total_pixels;

        let hash = grayscaled
            .pixels()
            .map(|p| (p[0] as f32) < average)
            .collect();

        Hash { bits: hash }
    }
}

impl Default for AverageHash {
    fn default() -> Self {
        AverageHash { img_size: (8, 8) }
    }
}
