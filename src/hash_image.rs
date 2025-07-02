mod hashing_method {
    use image::DynamicImage;

    trait ImageHash {
        fn hash(img: &DynamicImage) -> &str;
    }

    mod ahash {
        use super::ImageHash;

        struct AverageHash {
            hash_len: i16,
            name: &str,
        }

        impl ImageHash for AverageHash {
            fn hash(img: &image::DynamicImage) -> &str {}
        }
    }
}
