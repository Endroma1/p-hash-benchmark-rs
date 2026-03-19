# Benchmarking for perceptual hashes in facial recognition
Modifies, hashes and matches images.

## Usage
See `/examples/sqlite-rayon`. It uses two identical images from `images` dir.

```bash
cargo run --example sqlite-rayon

# Build with --release for better performance
cargo build --release --example sqlite-rayon  
```

It outputs to a SQLite db `data.db`.


![DB Structure](./p-hash-db.svg)


## Description
### Background
In certain facial recognition systems images can be sent to a server to identify a person that tries to log in. It is possible to spoof such a system by sending in images that are already sent in. To detect this it is possible to use perceptual hashes that stores image info in a string and compare incoming images to an existing set of hashes. To circumvent this, an attacket might sligtly modify images to fool the system into thinking that it is a completly new image.

### Goal
The aim of this program is to simulate such attacks and identify what hashing methods that are the most effective and robust at detecting modified images as identical to their originals and seperate from others.


