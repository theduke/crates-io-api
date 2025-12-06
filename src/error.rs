//! Error types.

/// Errors returned by the api client.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Low level http error
    #[error("Low level http error: {0}")]
    Http(#[from] reqwest::Error),
    /// Invalid url
    #[error("Invalid url: {0}")]
    Url(#[from] url::ParseError),
    /// Crate couldn't be found
    #[error("Resource at {0} couldn't be found.")]
    NotFound(String),
    /// No permission to access the resource.
    #[error("No permission to access the resource: {0}")]
    PermissionDenied(String),
    /// JSON decoding of API response failed.
    #[error("JSON decoding of API response failed: {0}")]
    JsonDecode(String),
    /// Error returned by the crates.io API directly.
    #[error("Error returned by the crates.io API directly: {0:?}")]
    Api(#[from] crate::types::ApiErrors),
}
