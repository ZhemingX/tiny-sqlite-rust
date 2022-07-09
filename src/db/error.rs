#[derive(Debug)]
pub enum DbError {
    IoError(std::io::Error),
    Other(String),
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

pub type DbResult<T> = Result<T, DbError>;
