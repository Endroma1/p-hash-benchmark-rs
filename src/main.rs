use imagehash::{AverageHash, HashMethod};
use imagemodify::modification::{Blur, ImageModification};
use std::env;
mod imagehash;
mod imagemodify;

fn main() {
    let path = env::current_dir()
        .expect("Failed to get current directory")
        .join("images")
        .join("test-mod.jpg");
    let img = image::open(&path).expect(&format!("Could not open image {}", path.display()));

    let save_path = env::current_dir()
        .expect("Failed to get current directory")
        .join("test_mod_img.png");

    let modifier = Blur::new();
    let mod_img = modifier.apply(&img);
    let save_status = mod_img.save(save_path);
    match save_status {
        Ok(()) => println!("Saved image"),
        Err(e) => println!("Could not save image {}", e),
    }

    let hasher = AverageHash::new();
    let hash = hasher.hash(img);
    let hex = hash.to_hex();
    println!("{}", hex)
}
