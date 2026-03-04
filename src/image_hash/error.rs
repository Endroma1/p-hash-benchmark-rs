use std::fmt::Display;

pub enum Error {
    HashingMethodNotFound { id: u16 },
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HashingMethodNotFound { id } => {
                write!(f, "Hashing method not found with id {}", id)
            }
        }
    }
}
