#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Test error")]
    Test,

    #[error("Could not create file: {0}")]
    FileCreate(String),

    #[error("Could not write to file: {0}")]
    FileWrite(String),

    #[error("Could not open file: {0}")]
    FileOpen(String),

    #[error("Could not compress the given data")]
    Compression,

    #[error("The given id doesn't correspond to any saved cache")]
    NotFound,
}
