pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Unimplemented error enum for development.
    Todo,

    /// Wrapper for std's IO error.
    StdIo(String),
    
    /// The path to a vault might be invalid.
    InvalidPath,
    
    /// Metadata parsing of the note was not successful.
    MetaDataError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::StdIo(format!("{:?}", e))
    }
}
