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
//! ```rust
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

mod async_client;
mod error;
mod sync_client;
mod types;

const DEFAULT_USER_AGENT: &str = concat!("crates-io-api ", env!("CARGO_PKG_VERSION"));

pub use crate::{
    async_client::Client as AsyncClient,
    error::{Error, NotFound},
    sync_client::SyncClient,
    types::*,
};
