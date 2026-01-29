use std::{
    path::Path,
    sync::{Arc, mpsc},
};

use crate::img_proc::{ImageReadApp, ImageReadMessage};

enum ProcMessage {
    Quit,
}
mod img_mod;
mod img_proc;

fn main() {
    env_logger::init();
    log::debug!("starting phash");
    let test_path = Path::new("/home/heste/.local/share/p-hash/images/");

    let (tx, rx) = mpsc::channel();
    let mut img_read_app = ImageReadApp::new(tx);
    let join_handle = img_read_app.run(test_path.to_path_buf());

    let mut iterations = 0;
    loop {
        let res = rx.recv();
        match res {
            Ok(m) => {
                if let Some(m) = handle_message(m) {
                    match m {
                        ProcMessage::Quit => break,
                    }
                }
            }
            Err(_) => {
                log::warn!("image reader closed before expected");
                break;
            }
        }
        iterations += 1;
    }
    join_handle
        .join()
        .expect("could not join read image thread");

    log::debug!(
        "processed: {}",
        Arc::clone(&img_read_app.get_processed()).lock().unwrap()
    );
    log::debug!("iterations: {}", iterations);
}
fn handle_message(msg: ImageReadMessage) -> Option<ProcMessage> {
    match msg {
        ImageReadMessage::Image { image } => log::debug!("got image {:?}", image),
        ImageReadMessage::Error { err } => log::error!("error {}", err),
        ImageReadMessage::Quit => return Some(ProcMessage::Quit),
    }
    None
}
