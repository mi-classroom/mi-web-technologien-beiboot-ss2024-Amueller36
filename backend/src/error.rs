use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image creation error: {0}")]
    CreateImageError(String),
    #[error("Metadata error: {0}")]
    MetadataError(#[from] MetadataError),
    #[error("Other error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Could not deserialize metadata: {0}")]
    SerdeError(#[from] serde_json::Error),
}