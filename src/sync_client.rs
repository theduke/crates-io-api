use super::*;
use std::iter::Extend;

use log::trace;
use reqwest::{blocking::Client as HttpClient, StatusCode, Url};
use serde::de::DeserializeOwned;

use crate::{error::JsonDecodeError, helper::*, types::*};

/// A synchronous client for the crates.io API.
pub struct SyncClient {
    client: HttpClient,
    base_url: Url,
    rate_limit: std::time::Duration,
    last_request_time: std::sync::Mutex<Option<std::time::Instant>>,
}

impl SyncClient {
    /// Instantiate a new client.
    ///
    /// Returns an [`Error`] if the given user agent is invalid.
    ///
    /// To respect the offical [Crawler Policy](https://crates.io/policies#crawlers),
    /// you must specify both a descriptive user agent and a rate limit interval.
    ///
    /// At most one request will be executed in the specified duration.
    /// The guidelines suggest 1 per second or less.
    ///
    /// Example user agent: `"my_bot (my_bot.com/info)"` or `"my_bot (help@my_bot.com)"`.
    ///
    /// ```rust
    /// # fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = crates_io_api::AsyncClient::new(
    ///   "my_bot (help@my_bot.com)",
    ///   std::time::Duration::from_millis(1000),
    /// ).unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        user_agent: &str,
        rate_limit: std::time::Duration,
    ) -> Result<Self, reqwest::header::InvalidHeaderValue> {
        Self::build(user_agent, rate_limit, None)
    }

    /// ```rust
    /// use crates_io_api::{SyncClient,Registry};
    /// # fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = crates_io_api::SyncClient::build(
    ///   "my_bot (help@my_bot.com)",
    ///   std::time::Duration::from_millis(1000),
    ///   Some(&Registry{
    ///     url: "https://crates.my-registry.com/api/v1/".to_string(),
    ///     name: Some("my_registry".to_string()),
    ///     token: None,
    ///     }),
    ///  ).unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(
        user_agent: &str,
        rate_limit: std::time::Duration,
        registry: Option<&Registry>,
    ) -> Result<Self, reqwest::header::InvalidHeaderValue> {
        let headers = setup_headers(user_agent, registry)?;
        let base_url = base_url(registry);

        Ok(Self {
            client: HttpClient::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            base_url: Url::parse(base_url).unwrap(),
            rate_limit,
            last_request_time: std::sync::Mutex::new(None),
        })
    }

    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, Error> {
        trace!("GET {}", url);

        let mut lock = self.last_request_time.lock().unwrap();
        if let Some(last_request_time) = lock.take() {
            let now = std::time::Instant::now();
            if last_request_time.elapsed() < self.rate_limit {
                std::thread::sleep((last_request_time + self.rate_limit) - now);
            }
        }

        let time = std::time::Instant::now();

        let res = self.client.get(url.clone()).send()?;

        if !res.status().is_success() {
            let err = match res.status() {
                StatusCode::NOT_FOUND => Error::NotFound(super::error::NotFoundError {
                    url: url.to_string(),
                }),
                StatusCode::FORBIDDEN => {
                    let reason = res.text().unwrap_or_default();
                    Error::PermissionDenied(super::error::PermissionDeniedError { reason })
                }
                _ => Error::from(res.error_for_status().unwrap_err()),
            };

            return Err(err);
        }

        *lock = Some(time);

        let content = res.text()?;

        // First, check for api errors.

        if let Ok(errors) = serde_json::from_str::<ApiErrors>(&content) {
            return Err(Error::Api(errors));
        }

        let jd = &mut serde_json::Deserializer::from_str(&content);
        serde_path_to_error::deserialize::<_, T>(jd).map_err(|err| {
            Error::JsonDecode(JsonDecodeError {
                message: format!("Could not decode JSON: {err} (path: {})", err.path()),
            })
        })
    }

    /// Retrieve a summary containing crates.io wide information.
    pub fn summary(&self) -> Result<Summary, Error> {
        let url = self.base_url.join("summary").unwrap();
        self.get(url)
    }

    /// Retrieve information of a crate.
    ///
    /// If you require detailed information, consider using [full_crate]().
    pub fn get_crate(&self, crate_name: &str) -> Result<CrateResponse, Error> {
        let url = super::async_client::build_crate_url(&self.base_url, crate_name)?;
        self.get(url)
    }

    /// Retrieve download stats for a crate.
    pub fn crate_downloads(&self, crate_name: &str) -> Result<CrateDownloads, Error> {
        let url = super::async_client::build_crate_downloads_url(&self.base_url, crate_name)?;
        self.get(url)
    }

    /// Retrieve the owners of a crate.
    pub fn crate_owners(&self, crate_name: &str) -> Result<Vec<User>, Error> {
        let url = super::async_client::build_crate_owners_url(&self.base_url, crate_name)?;
        let resp: Owners = self.get(url)?;
        Ok(resp.users)
    }

    /// Get a single page of reverse dependencies.
    ///
    /// Note: if the page is 0, it is coerced to 1.
    pub fn crate_reverse_dependencies_page(
        &self,
        crate_name: &str,
        page: u64,
    ) -> Result<ReverseDependencies, Error> {
        let url =
            super::async_client::build_crate_reverse_deps_url(&self.base_url, crate_name, page)?;
        let page = self.get::<ReverseDependenciesAsReceived>(url)?;

        let mut deps = ReverseDependencies {
            dependencies: Vec::new(),
            meta: Meta { total: 0 },
        };
        deps.meta.total = page.meta.total;
        deps.extend(page);
        Ok(deps)
    }

    /// Load all reverse dependencies of a crate.
    ///
    /// Note: Since the reverse dependency endpoint requires pagination, this
    /// will result in multiple requests if the crate has more than 100 reverse
    /// dependencies.
    pub fn crate_reverse_dependencies(
        &self,
        crate_name: &str,
    ) -> Result<ReverseDependencies, Error> {
        let mut deps = ReverseDependencies {
            dependencies: Vec::new(),
            meta: Meta { total: 0 },
        };

        for page_number in 1.. {
            let page = self.crate_reverse_dependencies_page(crate_name, page_number)?;
            if page.dependencies.is_empty() {
                break;
            }

            deps.dependencies.extend(page.dependencies);
            deps.meta.total = page.meta.total;
        }
        Ok(deps)
    }

    /// Get the total count of reverse dependencies for a given crate.
    pub fn crate_reverse_dependency_count(&self, crate_name: &str) -> Result<u64, Error> {
        let page = self.crate_reverse_dependencies_page(crate_name, 1)?;
        Ok(page.meta.total)
    }

    /// Retrieve the authors for a crate version.
    pub fn crate_authors(&self, crate_name: &str, version: &str) -> Result<Authors, Error> {
        let url =
            super::async_client::build_crate_authors_url(&self.base_url, crate_name, version)?;
        let res: AuthorsResponse = self.get(url)?;
        Ok(Authors {
            names: res.meta.names,
        })
    }

    /// Retrieve the dependencies of a crate version.
    pub fn crate_dependencies(
        &self,
        crate_name: &str,
        version: &str,
    ) -> Result<Vec<Dependency>, Error> {
        let url =
            super::async_client::build_crate_dependencies_url(&self.base_url, crate_name, version)?;
        let resp: Dependencies = self.get(url)?;
        Ok(resp.dependencies)
    }

    fn full_version(&self, version: Version) -> Result<FullVersion, Error> {
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
            license: version.license,
            links: version.links,
            readme_path: version.readme_path,

            author_names: authors.names,
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
    ///
    /// Note: Each version requires two extra requests.
    pub fn full_crate(&self, name: &str, all_versions: bool) -> Result<FullCrate, Error> {
        let resp = self.get_crate(name)?;
        let data = resp.crate_data;

        let dls = self.crate_downloads(name)?;
        let owners = self.crate_owners(name)?;
        let reverse_dependencies = self.crate_reverse_dependencies(name)?;

        let versions = if resp.versions.is_empty() {
            vec![]
        } else if all_versions {
            //let versions_res: Result<Vec<FullVersion>> = resp.versions
            resp.versions
                .into_iter()
                .map(|v| self.full_version(v))
                .collect::<Result<Vec<FullVersion>, Error>>()?
        } else {
            let v = self.full_version(resp.versions[0].clone())?;
            vec![v]
        };

        let full = FullCrate {
            id: data.id,
            name: data.name,
            description: data.description,
            license: versions[0].license.clone(),
            documentation: data.documentation,
            homepage: data.homepage,
            repository: data.repository,
            total_downloads: data.downloads,
            max_version: data.max_version,
            max_stable_version: data.max_stable_version,
            created_at: data.created_at,
            updated_at: data.updated_at,

            categories: resp.categories,
            keywords: resp.keywords,
            downloads: dls,
            owners,
            reverse_dependencies,
            versions,
        };
        Ok(full)
    }

    /// Retrieve a page of crates, optionally constrained by a query.
    ///
    /// If you want to get all results without worrying about paging,
    /// use [`all_crates`].
    ///
    /// # Examples
    ///
    /// Retrieve the first page of results for the query "api", with 100 items
    /// per page and sorted alphabetically.
    ///
    /// ```rust
    /// # use crates_io_api::{SyncClient, CratesQuery, Sort, Error};
    ///
    /// # fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = SyncClient::new(
    /// #     "my-bot-name (my-contact@domain.com)",
    /// #     std::time::Duration::from_millis(1000),
    /// # ).unwrap();
    /// let q = CratesQuery::builder()
    ///   .sort(Sort::Alphabetical)
    ///   .search("awesome")
    ///   .build();
    /// let crates = client.crates(q)?;
    /// # std::mem::drop(crates);
    /// # Ok(())
    /// # }
    /// ```
    pub fn crates(&self, query: CratesQuery) -> Result<CratesPage, Error> {
        let mut url = self.base_url.join("crates")?;
        query.build(url.query_pairs_mut());

        self.get(url)
    }

    /// Retrieves a user by username.
    pub fn user(&self, username: &str) -> Result<User, Error> {
        let url = self.base_url.join(&format!("users/{}", username))?;
        self.get::<UserResponse>(url).map(|response| response.user)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn build_test_client() -> SyncClient {
        SyncClient::new(
            "crates-io-api-ci (github.com/theduke/crates-io-api)",
            std::time::Duration::from_millis(1000),
        )
        .unwrap()
    }

    #[test]
    fn test_summary() -> Result<(), Error> {
        let client = build_test_client();
        let summary = client.summary()?;
        assert!(summary.most_downloaded.len() > 0);
        assert!(summary.just_updated.len() > 0);
        assert!(summary.new_crates.len() > 0);
        assert!(summary.most_recently_downloaded.len() > 0);
        assert!(summary.num_crates > 0);
        assert!(summary.num_downloads > 0);
        assert!(summary.popular_categories.len() > 0);
        assert!(summary.popular_keywords.len() > 0);
        Ok(())
    }

    #[test]
    fn test_full_crate() -> Result<(), Error> {
        let client = build_test_client();
        client.full_crate("crates_io_api", false)?;
        Ok(())
    }

    /// Ensure that the sync Client remains send.
    #[test]
    fn sync_client_ensure_send() {
        let client = build_test_client();
        let _: &dyn Send = &client;
    }

    #[test]
    fn test_user_get_async() -> Result<(), Error> {
        let client = build_test_client();
        let user = client.user("theduke")?;
        assert_eq!(user.login, "theduke");
        Ok(())
    }

    #[test]
    fn test_crates_filter_by_user_async() -> Result<(), Error> {
        let client = build_test_client();

        let user = client.user("theduke")?;

        let res = client.crates(CratesQuery {
            user_id: Some(user.id),
            per_page: 5,
            ..Default::default()
        })?;

        assert!(!res.crates.is_empty());
        // Ensure all found have the searched user as owner.
        for krate in res.crates {
            let owners = client.crate_owners(&krate.name)?;
            assert!(owners.iter().any(|o| o.id == user.id));
        }

        Ok(())
    }

    #[test]
    fn test_crate_reverse_dependency_count() -> Result<(), Error> {
        let client = build_test_client();
        let count = client.crate_reverse_dependency_count("crates_io_api")?;
        assert!(count > 0);

        Ok(())
    }
}
