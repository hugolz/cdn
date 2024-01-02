#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Test error")]
    Test,

    #[error("Could not create file: {0}")]
    FileCreate(String),

    #[error("Could not compress the given data")]
    Compression,
}
