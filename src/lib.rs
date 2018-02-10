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
//! use crates_io_api::{SyncClient, Result};
//! fn list_top_dependencies() -> Result<()> {
//!     // Instantiate the client.
//!     let client = SyncClient::new()?;
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

#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate reqwest;
extern crate chrono;
extern crate time;

pub mod types;

use std::iter::Extend;

use serde::de::DeserializeOwned;
use reqwest::{Url, StatusCode, UrlError};

use types::*;

error_chain! {
    foreign_links {
        Reqwest(reqwest::Error);
        Url(UrlError);
    }

    errors {
        ServerError {}
        NotFound {}
    }
}

/// Used to specify the sort behaviour of the Client::crates() method.
#[derive(Debug, Clone)]
pub enum Sort {
    /// Sort alphabetically.
    Alphabetical,
    /// Sort by relevance (meaningless if used without a query).
    Relevance,
    /// Sort by downloads.
    Downloads,
}

impl Sort {
    fn to_str(&self) -> &str {
        use self::Sort::*;
        match *self {
            Alphabetical => "alpha",
            Relevance => "",
            Downloads => "downloads",
        }
    }
}

/// Options for the [crates]() method of the client.
///
/// Used to specify pagination, sorting and a query.
pub struct ListOptions {
    pub sort: Sort,
    pub per_page: u64,
    pub page: u64,
    pub query: Option<String>,
}

/// A synchronous client for the crates.io API.
pub struct SyncClient {
    client: reqwest::Client,
    base_url: Url,
}

impl SyncClient {
    /// Instantiate a new synchronous API client.
    ///
    /// This will fail if the underlying http client could not be created.
    pub fn new() -> Result<Self> {
        let c = SyncClient {
            client: reqwest::Client::new(),
            base_url: Url::parse("https://crates.io/api/v1/").unwrap(),
        };
        Ok(c)
    }

    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T> {
        let mut res = self.client.get(url).send()?;
        if !res.status().is_success() {
            if res.status() == StatusCode::NotFound {
                Err(ErrorKind::NotFound.into())
            } else {
                Err(ErrorKind::ServerError.into())
            }
        } else {
            let data: T = res.json()?;
            Ok(data)
        }
    }

    /// Retrieve a summary containing crates.io wide information.
    pub fn summary(&self) -> Result<Summary> {
        let url = Url::parse("https://crates.io/summary")?;
        self.get(url)
    }

    /// Retrieve information of a crate.
    ///
    /// If you require detailed information, consider using [full_crate]().
    pub fn get_crate(&self, name: &str) -> Result<CrateResponse> {
        let url = self.base_url.join("crates/")?.join(name)?;
        self.get(url)
    }

    /// Retrieve download stats for a crate.
    pub fn crate_downloads(&self, name: &str) -> Result<Downloads> {
        let url = self.base_url.join(&format!("crates/{}/downloads", name))?;
        self.get(url)
    }

    /// Retrieve the owners of a crate.
    pub fn crate_owners(&self, name: &str) -> Result<Vec<User>> {
        let url = self.base_url.join(&format!("crates/{}/owners", name))?;
        let resp: Owners = self.get(url)?;
        Ok(resp.users)
    }

    /// Load all reverse dependencies of a crate.
    ///
    /// Note: Since the reverse dependency endpoint requires pagination, this
    /// will result in multiple requests if the crate has more than 100 reverse
    /// dependencies.
    pub fn crate_reverse_dependencies(&self, name: &str) -> Result<Vec<Dependency>> {
        let mut page = 1;
        let mut deps = Vec::new();
        loop {
            let url = self.base_url
                .join(&format!("crates/{}/reverse_dependencies?per_page=100&page={}",
                              name,
                              page))?;
            let res: Dependencies = self.get(url)?;
            if res.dependencies.is_empty() {
                deps.extend(res.dependencies);
                page += 1;
            } else {
                break;
            }
        }
        Ok(deps)
    }

    /// Retrieve the authors for a crate version.
    pub fn crate_authors(&self, name: &str, version: &str) -> Result<Authors> {
        let url = self.base_url
            .join(&format!("crates/{}/{}/authors", name, version))?;
        let res: AuthorsResponse = self.get(url)?;
        Ok(Authors{
            names: res.meta.names,
            users: res.users,
        })
    }

    /// Retrieve the dependencies of a crate version.
    pub fn crate_dependencies(&self, name: &str, version: &str) -> Result<Vec<Dependency>> {
        let url = self.base_url
            .join(&format!("crates/{}/{}/dependencies", name, version))?;
        let resp: Dependencies = self.get(url)?;
        Ok(resp.dependencies)
    }

    fn full_version(&self, version: Version) -> Result<FullVersion> {
        let authors = self.crate_authors(&version.crate_name, &version.num)?;
        let deps = self.crate_dependencies(&version.crate_name, &version.num)?;

        let v = FullVersion {
            created_at: version.created_at,
            updated_at: version.updated_at,
            dl_path: version.dl_path,
            downloads: version.downloads,
            features: version.features,
            id: version.id,
            num: version.num,
            yanked: version.yanked,
            links: version.links,
            readme_path: version.readme_path,

            author_names: authors.names,
            authors: authors.users,
            dependencies: deps,
        };
        Ok(v)
    }

    /// Retrieve all available information for a crate, including download
    /// stats,  owners and reverse dependencies.
    ///
    /// The `all_versions` argument controls the retrieval of detailed version
    /// information.
    /// If false, only the data for the latest version will be fetched, if true,
    /// detailed information for all versions will be available.
    /// Note: Each version requires two extra requests.
    pub fn full_crate(&self, name: &str, all_versions: bool) -> Result<FullCrate> {
        let resp = self.get_crate(name)?;
        let data = resp.crate_data;

        let dls = self.crate_downloads(name)?;
        let owners = self.crate_owners(name)?;
        let reverse_dependencies = self.crate_reverse_dependencies(name)?;


        let versions = if resp.versions.len() < 1 {
            vec![]
        } else if !all_versions {
            let v = self.full_version(resp.versions[0].clone())?;
            vec![v]
        } else {
            //let versions_res: Result<Vec<FullVersion>> = resp.versions
            resp.versions
                .into_iter()
                .map(|v| self.full_version(v))
                .collect::<Result<Vec<FullVersion>>>()?
        };

        let full = FullCrate {
            id: data.id,
            name: data.name,
            description: data.description,
            license: data.license,
            documentation: data.documentation,
            homepage: data.homepage,
            repository: data.repository,
            total_downloads: data.downloads,
            max_version: data.max_version,
            created_at: data.created_at,
            updated_at: data.updated_at,

            categories: resp.categories,
            keywords: resp.keywords,
            downloads: dls,
            owners,
            reverse_dependencies,

            versions: versions,
        };
        Ok(full)
    }

    /// Retrieve a page of crates, optionally constrained by a query.
    ///
    /// If you want to get all results without worrying about paging,
    /// use [all_crates]().
    ///
    /// # Examples
    ///
    /// Retrieve the first page of results for the query "api", with 100 items
    /// per page and sorted alphabetically.
    ///
    /// ```
    /// # fn f() -> crates_io_api::Result<()> {
    /// use crates_io_api::{SyncClient, ListOptions, Sort};
    ///
    /// let client = SyncClient::new()?;
    /// client.crates(ListOptions{
    ///   sort: Sort::Alphabetical,
    ///   per_page: 100,
    ///   page: 1,
    ///   query: Some("api".to_string()),
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn crates(&self, spec: ListOptions) -> Result<CratesResponse> {
        let mut url = self.base_url.join("crates")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("page", &spec.page.to_string());
            q.append_pair("per_page", &spec.per_page.to_string());
            q.append_pair("sort", spec.sort.to_str());
            if let Some(query) = spec.query {
                q.append_pair("q", &query);
            }
        }
        self.get(url)
    }

    /// Retrieve all crates, optionally constrained by a query.
    ///
    /// Note: This method fetches all pages of the result.
    /// This can result in a lot queries (100 results per query).
    pub fn all_crates(&self, query: Option<String>) -> Result<Vec<Crate>> {
        let mut page = 1;
        let mut crates = Vec::new();
        loop {
            let res = self.crates(ListOptions {
                                      query: query.clone(),
                                      sort: Sort::Alphabetical,
                                      per_page: 100,
                                      page: page,
                                  })?;
            if res.crates.is_empty() {
                crates.extend(res.crates);
                page += 1;
            } else {
                break;
            }
        }
        Ok(crates)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_client() {}
}
