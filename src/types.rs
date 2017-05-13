use std::collections::HashMap;
use chrono::{DateTime, UTC, NaiveDate};

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    total: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateLinks {
    owners: String,
    reverse_dependencies: String,
    version_downloads: String,
    versions: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
}

pub struct CratesResponse {
    pub crates: Crate,
    pub meta: Meta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionLinks {
    pub authors: String,
    pub dependencies: String,
    pub version_downloads: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    #[serde(rename="crate")]
    pub crate_name: String,
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
    pub dl_path: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
    pub id: u64,
    pub num: String,
    pub yanked: bool,
    pub links: VersionLinks,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub category: String,
    pub crates_cnt: u64,
    pub created_at: DateTime<UTC>,
    pub description: String,
    pub id: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Keyword {
    pub id: String,
    pub keyword: String,
    pub crates_cnt: u64,
    pub created_at: DateTime<UTC>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateResponse {
    pub categories: Vec<Category>,
    #[serde(rename="crate")]
    pub crate_data: Crate,
    pub keywords: Vec<Keyword>,
    pub versions: Vec<Version>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    pub just_updated: Vec<Crate>,
    pub most_downloaded: Vec<Crate>,
    pub new_crates: Vec<Crate>,
    pub num_crates: u64,
    pub num_downloads: u64,
    pub popular_categories: Vec<Category>,
    pub popular_keywords: Vec<Keyword>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionDownloads {
    pub date: NaiveDate,
    pub downloads: u64,
    pub id: u64,
    pub version: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtraDownloads {
    pub date: NaiveDate,
    pub downloads: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadsMeta {
    pub extra_downloads: Vec<ExtraDownloads>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Downloads {
    pub version_downloads: Vec<VersionDownloads>,
    pub meta: DownloadsMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub id: u64,
    pub kind: String,
    pub login: String,
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorsMeta {
    pub names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Authors {
    pub meta: AuthorsMeta,
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Owners {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Dependencies {
    pub dependencies: Vec<Dependency>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullVersion {
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
    pub dl_path: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
    pub id: u64,
    pub num: String,
    pub yanked: bool,
    pub links: VersionLinks,

    pub author_names: Vec<String>,
    pub authors: Vec<User>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,

    pub categories: Vec<Category>,
    pub keywords: Vec<Keyword>,
    pub downloads: Downloads,
    pub owners: Vec<User>,
    pub reverse_dependencies: Vec<Dependency>,

    pub versions: Vec<FullVersion>,
}
