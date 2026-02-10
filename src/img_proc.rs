use log::debug;
use std::{
    ffi::OsString,
    fmt::Display,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc::Sender},
    thread::{JoinHandle, spawn},
};

use walkdir::{DirEntry, WalkDir};
#[derive(Debug, Clone)]
pub struct Image {
    path: PathBuf,
    user: String,
}
impl Image {
    pub fn try_from_dir_entry(entry: DirEntry, root: &Path) -> Result<Self, Error> {
        let root_parent = root.parent().ok_or(Error::ParentNotFound {
            path: root.to_path_buf(),
        })?;
        let path = entry.path();
        let parent = path.parent().ok_or(Error::ParentNotFound {
            path: path.to_path_buf(),
        })?;
        let user = if root_parent == parent {
            debug!("comparing {:?} and {:?}", root_parent, parent);
            "undefined".to_string()
        } else {
            let user = parent.file_name().ok_or(Error::FileNameNotFound {
                path: parent.to_path_buf(),
            })?;
            user.to_str()
                .ok_or(Error::InvalidUnicode {
                    string: user.to_os_string(),
                })?
                .to_string()
        };

        Ok(Self {
            path: path.to_path_buf(),
            user,
        })
    }
    pub fn get_path(&self) -> &Path {
        &self.path
    }
    pub fn get_user(&self) -> &str {
        &self.user
    }
}

pub struct Images {
    images: Box<dyn Iterator<Item = Result<Image, Error>> + Send>,
}
impl Images {
    pub fn from_path(path: PathBuf) -> Self {
        let walker = WalkDir::new(path.clone())
            .into_iter()
            .filter_map(|r| r.ok())
            .filter(|e| e.path().is_file())
            .map(move |e| Image::try_from_dir_entry(e, &path).map(|i| i));
        Self {
            images: Box::new(walker),
        }
    }
    pub fn get_images(&self) -> &dyn Iterator<Item = Result<Image, Error>> {
        &*self.images
    }
}
impl Iterator for Images {
    type Item = Result<Image, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.images.next()
    }
}
struct ImageReader {
    img_path: PathBuf,
}

impl ImageReader {
    pub fn new(path: &Path) -> Self {
        Self {
            img_path: path.to_path_buf(),
        }
    }
    pub fn run(&self) -> Images {
        Images::from_path(self.img_path.clone())
    }
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Running,
    Stopped,
}
pub enum ImageReadMessage {
    Image { image: Image },
    Error { err: Error },
    Quit,
}

#[derive(Debug, Clone)]
pub enum Error {
    ParentNotFound { path: PathBuf },
    FileNameNotFound { path: PathBuf },
    InvalidUnicode { string: OsString },
    WalkDir { err: String },
}
impl From<walkdir::Error> for Error {
    fn from(value: walkdir::Error) -> Self {
        Error::WalkDir {
            err: value.to_string(),
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParentNotFound { path } => write!(f, "Parent path to path {:?} not found", path),
            Error::FileNameNotFound { path } => write!(f, "Name for file {:?} not found", path),
            Error::InvalidUnicode { string } => {
                write!(f, "Invalid unicode found for string {:?}", string)
            }
            Error::WalkDir { err } => write!(f, "WalkDirError: {}", err),
        }
    }
}
