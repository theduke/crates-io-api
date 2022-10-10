//! API client for [crates.io](https://crates.io).
//!
//! It aims to provide an easy to use and complete client for retrieving
//! information about Rust's crate ecosystem.
//!
//! Both a [AsyncClient](struct.AsyncClient.html) and a [SyncClient](struct.SyncClient.html) are available, providing either a
//! Futures based or a blocking interface.
//!
//! Please read the official crates.io [Crawler Policy](https://crates.io/policies#crawlers)
//! before using this crate.
//!
//! Due to this policy, you must specify both a user agent and a desired
//! rate limit delay when constructing a client.
//! See [SyncClient::new](struct.SyncClient.html#method.new) and [AsyncClient::new](struct.AsyncClient.html#method.new) for more information.
//!
//! # Examples
//!
//! Print the most downloaded crates and their non-optional dependencies:
//!
//! ```rust
//! use crates_io_api::{SyncClient, Error};
//!
//! fn list_top_dependencies() -> Result<(), Error> {
//!     // Instantiate the client.
//!     let client = SyncClient::new(
//!          "my-user-agent (my-contact@domain.com)",
//!          std::time::Duration::from_millis(1000),
//!     ).unwrap();
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
//! Instantiate a client for a private registry with environment variable authentication
//!
//! ```rust
//! use crates_io_api::{SyncClient,Registry};
//! let client = SyncClient::new(
//!          "my-user-agent (my-contact@domain.com)",
//!          std::time::Duration::from_millis(1000),
//!     ).unwrap();
//! ```

#![recursion_limit = "128"]
#![deny(missing_docs)]

mod async_client;
mod error;
mod helper;
mod sync_client;
mod types;

pub use crate::{
    async_client::Client as AsyncClient,
    error::{Error, NotFoundError, PermissionDeniedError},
    helper::*,
    sync_client::SyncClient,
    types::*,
};
