//! API client for [crates.io](https://crates.io).
//!
//! It aims to provide an easy to use and complete client for retrieving
//! information about Rust's crate ecosystem.
//!
//! **Note:** Right now, only a synchronous client is available.
//! Once the Async version of hyper stabilizes, an asynchronous client based
//! on Tokio will be added.
//!
//! # Examples
//!
//! Print the most downloaded crates and their non-optional dependencies:
//!
//! ```
//! use crates_io_api::{SyncClient, Error};
//!
//! fn list_top_dependencies() -> Result<(), Error> {
//!     // Instantiate the client.
//!     let client = SyncClient::new();
//!     // Retrieve summary data.
//!     let summary = client.summary()?;
//!     for c in summary.most_downloaded {
//!         println!("{}:", c.id);
//!         for dep in client.crate_dependencies(&c.id, &c.max_version)? {
//!             // Ignore optional dependencies.
//!             if !dep.optional {
//!                 println!("    * {} - {}", dep.id, dep.version_id);
//!             }
//!         }
//!     }
//!     Ok(())
//! }
//! ```

#![recursion_limit = "128"]

use failure::Fail;

mod async_client;
mod sync_client;
mod types;

pub use crate::async_client::Client as AsyncClient;
pub use crate::sync_client::SyncClient;
pub use crate::types::*;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Http(reqwest::Error),
    #[fail(display = "{}", _0)]
    Url(url::ParseError),
    #[fail(display = "{}", _0)]
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    #[fail(display = "Not found")]
    NotFound,
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
