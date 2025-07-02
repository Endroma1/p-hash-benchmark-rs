use modify_image::ModifyProcess;
use std::env;
mod hash_image;
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

    let mod_img = modify_image::ModifyImage {
        img,
        modification_name: "rotate",
        save_path,
    };

    let status = mod_img.modify_img();

    match status {
        Ok(()) => println!("Modified image"),
        Err(e) => println!("{}", e),
    }
}
