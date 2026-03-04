use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Sqlx { err: sqlx::Error },
    HashesNotEqualLength { l1: u32, l2: u32 },
    NotEnougHashes(usize),
}
impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx { err: value }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlx { err } => write!(f, "Sqlx Error: {}", err),
            Self::HashesNotEqualLength { l1, l2 } => write!(
                f,
                "Input hashes does not have equal length: {} != {} ",
                l1, l2
            ),
            Self::NotEnougHashes(len) => write!(
                f,
                "Not enough hashes found to begin matching. Expected len >= 2, found {} ",
                len
            ),
        }
    }
}
