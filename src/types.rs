//! Types for the data that is available via the API.

use chrono::{DateTime, NaiveDate, Utc};
use serde_derive::*;
use std::collections::HashMap;

/// Used to specify the registry being queried by either client.
pub struct Registry {
    /// Url of the registry
    pub url: String,
    /// Name of the registry
    pub name: Option<String>,
    /// Token used to authenticate registry requests.
    pub token: Option<String>,
}

/// Used to specify the sort behaviour of the `Client::crates()` method.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiErrors {
    /// Individual errors.
    pub errors: Vec<ApiError>,
}

/// Used to specify the sort behaviour of the `Client::crates()` method.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// Error message.
    pub detail: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.detail.as_deref().unwrap_or("Unknown API Error")
        )
    }
}

/// Used to specify the sort behaviour of the `Client::crates()` method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sort {
    /// Sort alphabetically.
    Alphabetical,
    /// Sort by relevance (meaningless if used without a query).
    Relevance,
    /// Sort by downloads.
    Downloads,
    /// Sort by recent downloads
    RecentDownloads,
    /// Sort by recent updates
    RecentUpdates,
    /// Sort by new
    NewlyAdded,
}

impl Sort {
    pub(crate) fn to_str(&self) -> &str {
        match self {
            Self::Alphabetical => "alpha",
            Self::Relevance => "",
            Self::Downloads => "downloads",
            Self::RecentDownloads => "recent-downloads",
            Self::RecentUpdates => "recent-updates",
            Self::NewlyAdded => "new",
        }
    }
}

/// Options for the [crates]() method of the client.
///
/// Used to specify pagination, sorting and a query.
#[derive(Clone, Debug)]
pub struct CratesQuery {
    /// Sort.
    pub(crate) sort: Sort,
    /// Number of items per page.
    pub(crate) per_page: u64,
    /// The page to fetch.
    pub(crate) page: u64,
    pub(crate) user_id: Option<u64>,
    /// Crates.io category name.
    /// See https://crates.io/categories
    /// NOTE: requires lower-case dash-separated categories, not the pretty
    /// titles visible in the listing linked above.
    pub(crate) category: Option<String>,
    /// Search query string.
    pub(crate) search: Option<String>,
}

impl CratesQuery {
    pub(crate) fn build(&self, mut q: url::form_urlencoded::Serializer<'_, url::UrlQuery<'_>>) {
        q.append_pair("page", &self.page.to_string());
        q.append_pair("per_page", &self.per_page.to_string());
        q.append_pair("sort", self.sort.to_str());
        if let Some(id) = self.user_id {
            q.append_pair("user_id", &id.to_string());
        }
        if let Some(search) = &self.search {
            q.append_pair("q", search);
        }
        if let Some(cat) = &self.category {
            q.append_pair("category", cat);
        }
    }
}

impl CratesQuery {
    /// Construct a new [`CratesQueryBuilder`].
    pub fn builder() -> CratesQueryBuilder {
        CratesQueryBuilder::new()
    }

    /// Get a reference to the crate query's sort.
    pub fn sort(&self) -> &Sort {
        &self.sort
    }

    /// Set the crate query's sort.
    pub fn set_sort(&mut self, sort: Sort) {
        self.sort = sort;
    }

    /// Get the crate query's per page.
    pub fn page_size(&self) -> u64 {
        self.per_page
    }

    /// Set the crate query's per page.
    pub fn set_page_size(&mut self, per_page: u64) {
        self.per_page = per_page;
    }

    /// Get the crate query's page.
    pub fn page(&self) -> u64 {
        self.page
    }

    /// Set the crate query's page.
    pub fn set_page(&mut self, page: u64) {
        self.page = page;
    }

    /// Get the crate query's user id.
    pub fn user_id(&self) -> Option<u64> {
        self.user_id
    }

    /// Set the crate query's user id.
    pub fn set_user_id(&mut self, user_id: Option<u64>) {
        self.user_id = user_id;
    }

    /// Get a reference to the crate query's category.
    pub fn category(&self) -> Option<&String> {
        self.category.as_ref()
    }

    /// Set the crate query's category.
    pub fn set_category(&mut self, category: Option<String>) {
        self.category = category;
    }

    /// Get a reference to the crate query's search.
    pub fn search(&self) -> Option<&String> {
        self.search.as_ref()
    }

    /// Set the crate query's search.
    pub fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }
}

impl Default for CratesQuery {
    fn default() -> Self {
        Self {
            sort: Sort::RecentUpdates,
            per_page: 30,
            page: 1,
            user_id: None,
            category: None,
            search: None,
        }
    }
}

/// Builder that enables easy construction of a [`CratesQuery`].
pub struct CratesQueryBuilder {
    query: CratesQuery,
}

impl CratesQueryBuilder {
    /// Construct a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            query: CratesQuery::default(),
        }
    }

    /// Set the sorting method.
    #[must_use]
    pub fn sort(mut self, sort: Sort) -> Self {
        self.query.sort = sort;
        self
    }

    /// Set the page size.
    #[must_use]
    pub fn page_size(mut self, size: u64) -> Self {
        self.query.per_page = size;
        self
    }

    /// Filter by a user id.
    #[must_use]
    pub fn user_id(mut self, user_id: u64) -> Self {
        self.query.user_id = Some(user_id);
        self
    }

    /// Crates.io category name.
    /// See https://crates.io/categories
    /// NOTE: requires lower-case dash-separated categories, not the pretty
    /// titles visible in the listing linked above.
    #[must_use]
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.query.category = Some(category.into());
        self
    }

    /// Search term.
    #[must_use]
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.query.search = Some(search.into());
        self
    }

    /// Finalize the builder into a usable [`CratesQuery`].
    #[must_use]
    pub fn build(self) -> CratesQuery {
        self.query
    }
}

impl Default for CratesQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    /// The total amount of results.
    pub total: u64,
}

/// Links to individual API endpoints that provide crate details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateLinks {
    pub owner_team: String,
    pub owner_user: String,
    pub owners: String,
    pub reverse_dependencies: String,
    pub version_downloads: String,
    pub versions: Option<String>,
}

/// A Rust crate published to crates.io.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Crate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    // FIXME: Remove on next breaking version bump.
    #[deprecated(
        since = "0.8.1",
        note = "This field is always empty. The license is only available on a specific `Version` of a crate or on `FullCrate`. This field will be removed in the next minor version bump."
    )]
    pub license: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    // TODO: determine badge format.
    // pub badges: Vec<??>,
    pub downloads: u64,
    pub recent_downloads: Option<u64>,
    /// NOTE: not set if the crate was loaded via a list query.
    pub categories: Option<Vec<String>>,
    /// NOTE: not set if the crate was loaded via a list query.
    pub keywords: Option<Vec<String>>,
    pub versions: Option<Vec<u64>>,
    pub max_version: String,
    pub max_stable_version: Option<String>,
    pub links: CrateLinks,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub exact_match: Option<bool>,
}

/// Full data for a crate listing.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CratesPage {
    pub crates: Vec<Crate>,
    #[serde(default)]
    pub versions: Vec<Version>,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
    #[serde(default)]
    pub categories: Vec<Category>,
    pub meta: Meta,
}

/// Links to API endpoints providing extra data for a crate version.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct VersionLinks {
    #[deprecated(
        since = "0.7.1",
        note = "This field was removed from the API and will always be empty. Will be removed in 0.8.0."
    )]
    #[serde(default)]
    pub authors: String,
    pub dependencies: String,
    pub version_downloads: String,
}

/// A [`Crate`] version.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Version {
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dl_path: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
    pub id: u64,
    pub num: String,
    pub yanked: bool,
    pub license: Option<String>,
    pub readme_path: Option<String>,
    pub links: VersionLinks,
    pub crate_size: Option<u64>,
    pub published_by: Option<User>,
}

/// A crate category.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Category {
    pub category: String,
    pub crates_cnt: u64,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub id: String,
    pub slug: String,
}

/// A keyword available on crates.io.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Keyword {
    pub id: String,
    pub keyword: String,
    pub crates_cnt: u64,
    pub created_at: DateTime<Utc>,
}

/// Full data for a crate.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateResponse {
    pub categories: Vec<Category>,
    #[serde(rename = "crate")]
    pub crate_data: Crate,
    pub keywords: Vec<Keyword>,
    pub versions: Vec<Version>,
}

/// Summary for crates.io.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Summary {
    pub just_updated: Vec<Crate>,
    pub most_downloaded: Vec<Crate>,
    pub new_crates: Vec<Crate>,
    pub most_recently_downloaded: Vec<Crate>,
    pub num_crates: u64,
    pub num_downloads: u64,
    pub popular_categories: Vec<Category>,
    pub popular_keywords: Vec<Keyword>,
}

/// Download data for a single crate version.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct VersionDownloads {
    pub date: NaiveDate,
    pub downloads: u64,
    pub version: u64,
}

/// Crate downloads that don't fit a particular date.
/// Only required for old download data.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct ExtraDownloads {
    pub date: NaiveDate,
    pub downloads: u64,
}

/// Additional data for crate downloads.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateDownloadsMeta {
    pub extra_downloads: Vec<ExtraDownloads>,
}

/// Download data for all versions of a [`Crate`].
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateDownloads {
    pub version_downloads: Vec<VersionDownloads>,
    pub meta: CrateDownloadsMeta,
}

/// A crates.io user.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct User {
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub id: u64,
    pub kind: Option<String>,
    pub login: String,
    pub name: Option<String>,
    pub url: Option<String>,
}

/// Additional crate author metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct AuthorsMeta {
    pub names: Vec<String>,
}

/// API Response for authors data.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub(crate) struct AuthorsResponse {
    pub meta: AuthorsMeta,
}

/// Crate author names.
#[allow(missing_docs)]
pub struct Authors {
    pub names: Vec<String>,
}

/// Crate owners.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Owners {
    pub users: Vec<User>,
}

/// A crate dependency.
/// Specifies the crate and features.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Dependency {
    pub crate_id: String,
    pub default_features: bool,
    pub downloads: u64,
    pub features: Vec<String>,
    pub id: u64,
    pub kind: String,
    pub optional: bool,
    pub req: String,
    pub target: Option<String>,
    pub version_id: u64,
}

/// List of dependencies of a crate.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Dependencies {
    pub dependencies: Vec<Dependency>,
}

/// Single reverse dependency (aka a dependent) of a crate.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct ReverseDependency {
    pub crate_version: Version,
    pub dependency: Dependency,
}

// This is how reverse dependencies are received
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct ReverseDependenciesAsReceived {
    pub dependencies: Vec<Dependency>,
    pub versions: Vec<Version>,
    pub meta: Meta,
}

/// Full list of reverse dependencies for a crate (version).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct ReverseDependencies {
    pub dependencies: Vec<ReverseDependency>,
    pub meta: Meta,
}

impl ReverseDependencies {
    /// Fills the dependencies field from a ReverseDependenciesAsReceived struct.
    pub(crate) fn extend(&mut self, rdeps: ReverseDependenciesAsReceived) {
        for d in rdeps.dependencies {
            for v in &rdeps.versions {
                if v.id == d.version_id {
                    // Right now it iterates over the full vector for each vector element.
                    // For large vectors, it may be faster to remove each matched element
                    // using the drain_filter() method once it's stabilized:
                    // https://doc.rust-lang.org/nightly/std/vec/struct.Vec.html#method.drain_filter
                    self.dependencies.push(ReverseDependency {
                        crate_version: v.clone(),
                        dependency: d.clone(),
                    });
                }
            }
        }
    }
}

/// Complete information for a crate version.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct FullVersion {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dl_path: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
    pub id: u64,
    pub num: String,
    pub yanked: bool,
    pub license: Option<String>,
    pub readme_path: Option<String>,
    pub links: VersionLinks,

    pub author_names: Vec<String>,
    pub dependencies: Vec<Dependency>,
}

/// Complete information for a crate.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct FullCrate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub total_downloads: u64,
    pub max_version: String,
    pub max_stable_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub categories: Vec<Category>,
    pub keywords: Vec<Keyword>,
    pub downloads: CrateDownloads,
    pub owners: Vec<User>,
    pub reverse_dependencies: ReverseDependencies,

    pub versions: Vec<FullVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct UserResponse {
    pub user: User,
}
