use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    core::{
        image_parser::{AppProcParser, ImageParser},
        state::PHashResult,
    },
    img_proc::Image,
};

/// Parses input images given by img_proc::Image data struct.
pub trait ImagesProcessor {
    fn run(&self, images: &Vec<Image>) -> Vec<PHashResult>;
}
pub struct RayonImagesProcessor {
    image_parser: Box<dyn ImageParser>,
}
impl ImagesProcessor for RayonImagesProcessor {
    fn run(&self, images: &Vec<Image>) -> Vec<PHashResult> {
        let res: Vec<PHashResult> = images
            .par_iter()
            .progress_with(ProgressBar::new(images.len() as u64))
            .enumerate()
            .map(move |(id, image)| {
                let res = self.image_parser.run(image, id as u32);
                res
            })
            .collect();
        res
    }
}
impl Default for RayonImagesProcessor {
    fn default() -> Self {
        let image_parser = Box::new(AppProcParser::default());
        Self { image_parser }
    }
}
