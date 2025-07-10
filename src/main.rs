use imagehash::{AverageHash, HashMethod};
use modify_image::ModifyImage;
use std::env;
mod imagehash;
mod modify_image;

fn main() {
    let path = env::current_dir()
        .expect("Failed to get current directory")
        .join("images")
        .join("test-mod.jpg");
    let img = image::open(&path).expect(&format!("Could not open image {}", path.display()));

    let save_path = env::current_dir()
        .expect("Failed to get current directory")
        .join("test_mod_img.png");

    let mod_img = ModifyImage::new(&img, "rotate");
    let status = mod_img.and_then(|m| m.save(save_path));

    match status {
        Ok(()) => println!("Modified image"),
        Err(e) => println!("{e}"),
    }

    let hasher = AverageHash::new();
    let hash = hasher.hash(img);
    let hex = hash.to_hex();
    println!("{}", hex)
}
