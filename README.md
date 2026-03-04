# Benchmarking for perceptual hashes in facial recognition
Modifies, hashes and matches images.

## Usage
See `/examples/sqlite-rayon`. It uses two identical images from `images` dir.

```bash
cargo run --example sqlite-rayon

# Build with --release for better performance
cargo build --release --example sqlite-rayon  
```

