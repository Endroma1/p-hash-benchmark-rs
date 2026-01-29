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
}

pub struct Images {
    images: Box<dyn Iterator<Item = Result<Image, Error>>>,
}
impl Images {
    fn try_from_path(path: PathBuf) -> Self {
        let walker = WalkDir::new(path.clone())
            .into_iter()
            .filter_map(|r| r.ok())
            .filter(|e| e.path().is_file())
            .map(move |e| Image::try_from_dir_entry(e, &path));
        Self {
            images: Box::new(walker),
        }
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
        Images::try_from_path(self.img_path.clone())
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
pub struct ImageReadApp {
    tx: Sender<ImageReadMessage>,
    processed: Arc<Mutex<u32>>,
}
impl ImageReadApp {
    pub fn new(tx: Sender<ImageReadMessage>) -> Self {
        Self {
            tx,
            processed: Arc::new(Mutex::new(0)),
        }
    }
    pub fn run(&mut self, path: PathBuf) -> JoinHandle<()> {
        let processed = Arc::clone(&self.processed);
        let tx = self.tx.clone();
        let join_handle = spawn(move || {
            let images = Images::try_from_path(path);

            for image in images {
                let message = match image {
                    Ok(img) => ImageReadMessage::Image { image: img },
                    Err(err) => ImageReadMessage::Error { err },
                };

                let res = tx.send(message);

                if let Err(_) = res {
                    log::error!("send for image reader failed, tunnel probably closed");
                    break;
                }
                *processed.lock().unwrap() += 1;
            }

            tx.send(ImageReadMessage::Quit);
        });
        log::debug!("imageread app executed successfully");
        join_handle
    }
    pub fn get_processed(&self) -> Arc<Mutex<u32>> {
        Arc::clone(&self.processed)
    }
}

pub enum Error {
    ParentNotFound { path: PathBuf },
    FileNameNotFound { path: PathBuf },
    InvalidUnicode { string: OsString },
    WalkDir { err: walkdir::Error },
}
impl From<walkdir::Error> for Error {
    fn from(value: walkdir::Error) -> Self {
        Error::WalkDir { err: value }
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
