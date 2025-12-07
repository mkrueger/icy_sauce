use std::path::PathBuf;

use bstr::BString;

use crate::SauceDataType;

#[derive(Debug, thiserror::Error)]
pub enum SauceError {
    #[error("Unsupported SAUCE version: {0}")]
    UnsupportedSauceVersion(BString),

    #[error("Invalid comment block")]
    InvalidCommentBlock,

    #[error("Unsupported SAUCE date: {0}")]
    UnsupportedSauceDate(BString),

    #[error("Binary file width limit exceeded: {0}")]
    BinFileWidthLimitExceeded(i32),

    #[error("Unsupported data type for operation: {0:?}")]
    UnsupportedDataType(SauceDataType),

    #[error("IO error reading '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Comment limit exceeded (255)")]
    CommentLimitExceeded,

    #[error("Comment too long: {0} bytes only up to 64 bytes are allowed.")]
    CommentTooLong(usize),

    #[error("Title too long: {0} bytes only up to 35 bytes are allowed.")]
    TitleTooLong(usize),

    #[error("Author too long: {0} bytes only up to 20 bytes are allowed.")]
    AuthorTooLong(usize),

    #[error("Group too long: {0} bytes only up to 20 bytes are allowed.")]
    GroupTooLong(usize),

    #[error("Font name too long: {0} bytes only up to 22 bytes are allowed.")]
    FontNameTooLong(usize),

    #[error("Missing EOF marker (0x1A) before SAUCE record")]
    MissingEofMarker,
}

impl SauceError {
    /// Create an IoError with the given path and source error.
    pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        SauceError::IoError {
            path: path.into(),
            source,
        }
    }
}
