mod hashing_method {
    use image::DynamicImage;

    trait ImageHash {
        fn hash(&self, img: &DynamicImage) -> String;
    }

    mod ahash {
        use super::{DynamicImage, ImageHash};

        struct AverageHash<'a> {
            hash_len: usize,
            name: &'a str,
        }

        impl<'a> ImageHash for AverageHash<'a> {
            fn hash(&self, img: &DynamicImage) -> String {
                let hash_bytes = imagehash::AverageHash::new()
                    .with_image_size(self.hash_len, self.hash_len)
                    .with_hash_size(self.hash_len, self.hash_len)
                    .hash(img)
                    .to_bytes();

                hex::encode(hash_bytes)
            }
        }
    }
}
