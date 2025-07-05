mod hashing_method {
    use image::DynamicImage;

    trait Hash {
        fn to_hex(&self) -> String;
    }

    impl Hash for imagehash::Hash {
        fn to_hex(&self) -> String {
            hex::encode(self.to_bytes())
        }
    }

    trait ImageHash {
        fn hash(&self, img: &DynamicImage) -> imagehash::Hash;
    }

    mod ahash {
        use super::{DynamicImage, ImageHash};

        struct AverageHash<'a> {
            hash_len: usize,
            name: &'a str,
        }

        impl<'a> ImageHash for AverageHash<'a> {
            fn hash(&self, img: &DynamicImage) -> imagehash::Hash {
                imagehash::AverageHash::new()
                    .with_image_size(self.hash_len, self.hash_len)
                    .with_hash_size(self.hash_len, self.hash_len)
                    .hash(img)
            }
        }
    }
}
