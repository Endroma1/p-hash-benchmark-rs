use std::fmt::Display;

use crate::{image_modify, matching};

#[derive(Debug)]
pub enum Error {
    ImageProc { err: crate::image_parse::Error },
    ImageMod { err: crate::image_modify::Error },
    ModificationNotFound { id: usize },
    HashingMethodNotFound { id: usize },
    ImageNotFound { id: usize },
    ImageHandleClosed,
    Sqlx { err: sqlx::Error },
    HomeDirNotFound,
    MatchError { err: matching::error::Error },
}
impl From<matching::error::Error> for Error {
    fn from(value: matching::error::Error) -> Self {
        Self::MatchError { err: value }
    }
}
impl From<image_modify::Error> for Error {
    fn from(value: crate::image_modify::Error) -> Self {
        Self::ImageMod { err: value }
    }
}
impl From<crate::image_parse::Error> for Error {
    fn from(value: crate::image_parse::Error) -> Self {
        Self::ImageProc { err: value }
    }
}
impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx { err: value }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImageProc { err } => write!(f, "Image proc error: {}", err),
            Self::ImageMod { err } => write!(f, "Image modification errer: {}", err),
            Self::ModificationNotFound { id } => write!(f, "Modification with id {} not found", id),
            Self::HashingMethodNotFound { id } => {
                write!(f, "Hashing method with id {} not found", id)
            }
            Self::ImageNotFound { id } => write!(f, "Image with id {} not found", id),
            Self::ImageHandleClosed => write!(f, "Image handle closed before expected"),
            Self::Sqlx { err } => write!(f, "Sqlx Error: {}", err),
            Self::HomeDirNotFound => write!(f, "Home dir not found"),
            Self::MatchError { err } => write!(f, "Error when matching: {}", err),
        }
    }
}
impl std::error::Error for Error {}
