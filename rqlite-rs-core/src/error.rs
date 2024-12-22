use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug)]
pub enum IntoTypedError {
    #[error("Failed to convert value to type")]
    ConversionError(#[from] serde_json::Error),
    #[error("Column not found")]
    ColumnNotFound,
    #[error("Value not found")]
    ValueNotFound,
    #[error("Expected {0} columns, found {1}")]
    ColumnCountMismatch(usize, usize),
    #[cfg(feature = "fast-blob")]
    #[error("Could not decode blob {0}")]
    BlobDecodeError(#[from] base64::DecodeError),
}
