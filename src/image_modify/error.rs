use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Image { err: image::ImageError },
    IO { err: std::io::Error },
    ModificationNotFound { id: usize },
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO { err: value }
    }
}
impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::Image { err: value }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image { err } => write!(f, "Image error: {}", err),
            Self::IO { err } => write!(f, "IO Error: {}", err),
            Self::ModificationNotFound { id } => {
                write!(f, "Modification not found with id: {}", id)
            }
        }
    }
}
