mod collection;
mod error;
mod hashing_methods;
mod interface;

pub use collection::{HashResult, HashingMethods, SelectedHashingMethods};
pub use error::Error;
pub use hashing_methods::*;
pub use interface::{Hash, HashingMethod, hash_images};
