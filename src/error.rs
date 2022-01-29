//! Error types.

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Http(reqwest::Error),
    Url(url::ParseError),
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    NotFound(NotFound),
    PermissionDenied(PermissionDenied),
    Api(crate::types::ApiErrors),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(e) => e.fmt(f),
            Error::Url(e) => e.fmt(f),
            Error::InvalidHeader(e) => e.fmt(f),
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
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::Url(e) => Some(e),
            Error::InvalidHeader(e) => Some(e),
            Error::NotFound(_) => None,
            Error::PermissionDenied(_) => None,
            Error::Api(_) => None,
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

impl From<reqwest::header::InvalidHeaderValue> for Error {
    fn from(e: reqwest::header::InvalidHeaderValue) -> Self {
        Error::InvalidHeader(e)
    }
}

#[derive(Debug)]
pub struct NotFound {
    pub(crate) url: String,
}

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resouce at url '{}' could not be found", self.url)
    }
}

#[derive(Debug)]
pub struct PermissionDenied {
    pub(crate) reason: String,
}

impl std::fmt::Display for PermissionDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Permission denied: {}", self.reason)
    }
}
