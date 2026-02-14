use crate::{
    core::{app_proc::AppProcess, images_processor::PHashResult},
    img_proc::Image,
};
/// Parses one image and returns a PHashResult which reperensents the modified images and hashes
/// for the input image.
pub trait ImageParser: Sync + Send {
    fn run(&self, image: &Image, id: u32) -> PHashResult;
}
#[derive(Debug, Default)]
pub struct AppProcParser {}
impl ImageParser for AppProcParser {
    fn run(&self, image: &Image, id: u32) -> PHashResult {
        let mut app_proc = AppProcess::new();
        let res = app_proc.run(image.get_path(), id);
        if let Err(e) = res {
            tracing::error!("app proc failed with error {}", e);
        }
        let proc_res = app_proc.finish();
        proc_res
    }
}
