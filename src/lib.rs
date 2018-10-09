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

#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate futures;
extern crate tokio;
extern crate url;

mod async;
mod sync;
mod types;

pub use async::Client as AsyncClient;
pub use sync::SyncClient;
pub use types::*;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Http(reqwest::Error),
    #[fail(display = "{}", _0)]
    Url(url::ParseError),
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
