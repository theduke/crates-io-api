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
#![cfg(feature = "sync")]
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

#![recursion_limit = "128"]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
mod async_client;
mod error;

#[cfg(all(feature = "sync", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
mod sync_client;

mod types;
mod util;

#[cfg(feature = "async")]
pub use crate::async_client::Client as AsyncClient;
#[cfg(all(feature = "sync", not(target_arch = "wasm32")))]
pub use crate::sync_client::SyncClient;

pub use crate::{
    error::{Error, NotFoundError, PermissionDeniedError},
    types::*,
};
