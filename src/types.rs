//! Types for the data that is available via the API.

use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};

/// Pagination information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    /// The total amount of results.
    total: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrateLinks {
    owner_team: String,
    owner_user: String,
    owners: String,
    reverse_dependencies: String,
    version_downloads: String,
    versions: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Crate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    // TODO: determine badge format.
    // pub badges: Vec<??>,
    pub downloads: u64,
    pub categories: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub versions: Option<Vec<u64>>,
    pub max_version: String,
    pub links: CrateLinks,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CratesResponse {
    pub crates: Vec<Crate>,
    pub meta: Meta,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionLinks {
    pub authors: String,
    pub dependencies: String,
    pub version_downloads: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
    #[serde(rename="crate")]
    pub crate_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dl_path: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
    pub id: u64,
    pub num: String,
    pub yanked: bool,
    pub readme_path: Option<String>,
    pub links: VersionLinks,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub category: String,
    pub crates_cnt: u64,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub id: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keyword {
    pub id: String,
    pub keyword: String,
    pub crates_cnt: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrateResponse {
    pub categories: Vec<Category>,
    #[serde(rename="crate")]
    pub crate_data: Crate,
    pub keywords: Vec<Keyword>,
    pub versions: Vec<Version>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionDownloads {
    pub date: NaiveDate,
    pub downloads: u64,
    pub id: u64,
    pub version: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtraDownloads {
    pub date: NaiveDate,
    pub downloads: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadsMeta {
    pub extra_downloads: Vec<ExtraDownloads>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Downloads {
    pub version_downloads: Vec<VersionDownloads>,
    pub meta: DownloadsMeta,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub id: u64,
    pub kind: String,
    pub login: String,
    pub name: Option<String>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorsMeta {
    pub names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorsResponse {
    pub meta: AuthorsMeta,
    pub users: Vec<User>,
}

pub struct Authors {
    pub names: Vec<String>,
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Owners {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependencies {
    pub dependencies: Vec<Dependency>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullVersion {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dl_path: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
    pub id: u64,
    pub num: String,
    pub yanked: bool,
    pub readme_path: Option<String>,
    pub links: VersionLinks,

    pub author_names: Vec<String>,
    pub authors: Vec<User>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub categories: Vec<Category>,
    pub keywords: Vec<Keyword>,
    pub downloads: Downloads,
    pub owners: Vec<User>,
    pub reverse_dependencies: Vec<Dependency>,

    pub versions: Vec<FullVersion>,
}
