//! Error types.

/// Errors returned by the api client.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Low-level http error.
    Http(reqwest::Error),
    /// Invalid URL.
    Url(url::ParseError),
    /// Crate could not be found.
    NotFound(NotFoundError),
    /// No permission to access the resource.
    PermissionDenied(PermissionDeniedError),
    /// JSON decoding of API response failed.
    JsonDecode(JsonDecodeError),
    /// Error returned by the crates.io API directly.
    Api(crate::types::ApiErrors),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(e) => e.fmt(f),
            Error::Url(e) => e.fmt(f),
            Error::NotFound(e) => e.fmt(f),
            Error::PermissionDenied(e) => e.fmt(f),
            Error::Api(err) => {
                let inner = if err.errors.is_empty() {
                    "Unknown API error".to_string()
                } else {
                    err.errors
                        .iter()
                        .map(|err| err.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                write!(f, "API Error ({})", inner)
            }
            Error::JsonDecode(err) => write!(f, "Could not decode API JSON response: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::Url(e) => Some(e),
            Error::NotFound(_) => None,
            Error::PermissionDenied(_) => None,
            Error::Api(_) => None,
            Error::JsonDecode(err) => Some(err),
        }
    }

    // TODO: uncomment once backtrace feature is stabilized (https://github.com/rust-lang/rust/issues/53487).
    /*
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        match self {
            Self::Http(e) => e.backtrace(),
            Self::Url(e) => e.backtrace(),
            Self::InvalidHeader(e) => e.backtrace(),
            Self::NotFound(_) => None,
        }
    }
    */
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::Url(e)
    }
}

/// Error returned when the JSON returned by the API could not be decoded.
#[derive(Debug)]
pub struct JsonDecodeError {
    pub(crate) message: String,
}

impl std::fmt::Display for JsonDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not decode JSON: {}", self.message)
    }
}

impl std::error::Error for JsonDecodeError {}

/// Error returned when a resource could not be found.
#[derive(Debug)]
pub struct NotFoundError {
    pub(crate) url: String,
}

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource at url '{}' could not be found", self.url)
    }
}

/// Error returned when a resource is not accessible.
#[derive(Debug)]
pub struct PermissionDeniedError {
    pub(crate) reason: String,
}

impl std::fmt::Display for PermissionDeniedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Permission denied: {}", self.reason)
    }
}
