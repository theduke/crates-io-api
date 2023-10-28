# Changelog

### Features

* Query registries other than crates-io
  - Additional `AsyncClient::build()` and `SyncClient::build()` functions.
    For building a client for an alternative registry.

### (Breaking) Changes

* `AsyncClient::with_http_client()` now requires the crate registry url to be specified.
* Types, make field optional: User {url}

## 0.8.1

* Add `AsyncClient::with_http_client` constructor
* Add `Crate::max_stable_version` field
* Improve error reporting for JSON decoding errors
* Deprecate the `Crate::license field`
  (field is unused and always empty)

## 0.8.0 - 2022-01-29

This version has quite a few breaking changes,
mainly to clean up and future-proof the API.

### Features

* Get user data with `Client::user()`
* Filter crates by category
* Filter crates by user_id
* Add `reverse_dependency_count()` to easily get the number of reverse deps
* Allow retrieving single reverse dependency pages (`crate_reverse_dependencies_page`)
* (async): Add a paginated Stream for listing crates (`AsyncClient::crates_stream()`)

### (Breaking) Changes

* Error
  - make Error #[non_exhaustive]
  - add Error::Api variant
  - Rename NotFound => NotFoundError
  - Rename PermissionDenied => PermissionDeniedError
  - Remove InvalidHeaders variant (only relevant for client construction)

* Types
  - Rename `CratesResponse` => `CratesPage`
  - Rename `DownloadsMeta` => `CrateDownloadsMeta`
  - Rename `Downloads` => `CrateDownloads`
  - Don't expose internal types (`AuthorsResponse`)
  - Remove unused `Authors`/`FullVersion`::users field

* General
  - Properly handle API errors (Error::Api variant)

* Querying
  - rename `ListOptions` to `CratesQuery`
  - make `CratesQuery` fields private (future proofing)
  - add `CratesQueryBuilder` for query construction

### Sync Client

* Remove `all_crates` method, which should never have been there...

### Async Client

* Clean up the old pre-async futures code
* Don't auto-clone: futures are now tied to the client lifetime.
  Manually clone if you need the futures to be owned.


## 0.7.3 - 2021-10-26

* Fix sort by relevance (https://github.com/theduke/crates_io_api/pull/35)
* Provide rustls option via feature flag (https://github.com/theduke/crates_io_api/pull/34)

## 0.7.2 - 2021-07-05

* Disable default features of chrono to have fewer dependencies.

## 0.7.1 - 2021-05-18

* Deprecate the `VersionLinks.authors` field that was removed from the API
  Now will always be empty.
  Field will be removed in 0.8.

## 0.6.1 - 2020-07-19

* Make `SyncClient` `Send + Sync` [#22](https://github.com/theduke/crates_io_api/pull/22)

## 0.6.0 - 2020-05-25

* Upgrade the async client to Futures 0.3 + reqwest 0.10
  (reqwest 0.10 also respects standard http_proxy env variables)
* Removed `failure` dependency
* Adhere to the crawler policy by requiring a custom user agent
* Add a *simple* rate limiter that restricts a client to one request in a given
  duration, and only a single concurrent request.

## 0.5.1 - 2019-08-23

* Fix faulty condition check in SyncClient::all_crates

## 0.5.0 - 2019/06/22

* Add 7 missing type fields for:
  * Crate {recent_downloads, exact_match}
  * CrateResponse {versions, keywords, categories}
  * Version {crate_size, published_by}
* Make field optional: User {kind}
* Fix getting the reverse dependencies.
  * Rearrange the received data for simpler manipulation.
  * Add 3 new types:
    * ReverseDependenciesAsReceived {dependencies, versions, meta}
    * ReverseDependencies {dependencies, meta}
    * ReverseDependency {crate_version, dependency}

## 0.4.1 - 2019/03/09

* Fixed errors for version information due to the `id` field being removed from the API.  [PR #11](https://github.com/theduke/crates_io_api/pull/11)

## 0.4.0 - 2019/03/01

* Added `with_user_agent` method to client
* Switch to 2018 edition, requiring rustc 1.31+

## 0.3.0 - 2018/10/09

* Upgrade reqwest to 0.9
* Upgrade to tokio instead of tokio_core

## 0.2.0 - 2018/04/29

* Add AsyncClient
* Switch from error_chain to failure
* Remove unused time dependency and loosen dependency constraints

## 0.1.0 - 2018/02/10

* Add some newly introduced fields in the API
* Fix URL for the /summary endpoint
* Upgrade dependencies
* Add a simple test
