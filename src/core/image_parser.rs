use crate::{
    core::{Error, app_proc::AppProcess, images_processor::PHashResult},
    image_hash::HashingMethods,
    image_modify::Modifications,
    image_parse::Image,
};
/// Parses one image and returns a PHashResult which reperensents the modified images and hashes
/// for the input image.
pub trait ImageParser: Sync + Send {
    fn run(
        &self,
        image: &Image,
        id: u32,
        modifications: &Modifications,
        hashing_methods: &HashingMethods,
    ) -> Result<PHashResult, Error>;
}
#[derive(Debug, Default)]
pub struct AppProcParser {}
impl ImageParser for AppProcParser {
    fn run(
        &self,
        image: &Image,
        id: u32,
        modifications: &Modifications,
        hashing_methods: &HashingMethods,
    ) -> Result<PHashResult, Error> {
        let mut app_proc = AppProcess::new();
        app_proc.run(image.get_path(), id, modifications, hashing_methods)?;

        let proc_res = app_proc.finish();
        Ok(proc_res)
    }
}
