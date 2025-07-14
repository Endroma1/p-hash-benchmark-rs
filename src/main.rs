use clap::Parser;
use imagehash::{AverageHash, HashMethod};
use imagemodify::modification::{Blur, ImageModification};
use p_hash_rust::Config;
use std::env;
mod imagehash;
mod imagemodify;
mod lib;

fn main() {
    let args = Args::parse();

    if args.create_config {
        println!("Creating default config");

        let path = Config::default_path();

        let config = Config::create_default(&path);

        match config {
            Ok(()) => println!("Created config to path {:?}", path),
            Err(e) => println!("Failed to create config, {}", e),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    create_config: bool,
}

fn test() {
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
