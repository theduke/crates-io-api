use super::*;
use std::iter::Extend;

use log::trace;
use reqwest::{blocking::Client as HttpClient, header, StatusCode, Url};
use serde::de::DeserializeOwned;

use crate::types::*;

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
    /// To respect the offical [Crawler Policy](https://crates.io/policies#crawlers),
    /// you must specify both a descriptive user agent and a rate limit interval.
    ///
    /// At most one request will be executed in the specified duration.
    /// The guidelines suggest 1 per second or less.
    ///
    /// Example user agent: `"my_bot (my_bot.com/info)"` or `"my_bot (help@my_bot.com)"`.
    ///
    /// ```rust
    /// # fn f() -> Result<(), crates_io_api::Error> {
    /// let client = crates_io_api::AsyncClient::new(
    ///   "my_bot (help@my_bot.com)",
    ///   std::time::Duration::from_millis(1000),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        user_agent: &str,
        rate_limit: std::time::Duration,
    ) -> Result<Self, reqwest::header::InvalidHeaderValue> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent)?,
        );

        Ok(Self {
            client: HttpClient::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            base_url: Url::parse("https://crates.io/api/v1/").unwrap(),
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

        let res = {
            let res = self.client.get(url.clone()).send()?;

            if res.status() == StatusCode::NOT_FOUND {
                return Err(Error::NotFound(super::NotFound {
                    url: url.to_string(),
                }));
            }
            res.error_for_status()?
        };

        *lock = Some(time);

        let data: T = res.json()?;
        Ok(data)
    }

    /// Retrieve a summary containing crates.io wide information.
    pub fn summary(&self) -> Result<Summary, Error> {
        let url = self.base_url.join("summary").unwrap();
        self.get(url)
    }

    /// Retrieve information of a crate.
    ///
    /// If you require detailed information, consider using [full_crate]().
    pub fn get_crate(&self, name: &str) -> Result<CrateResponse, Error> {
        let url = self.base_url.join("crates/")?.join(name)?;
        self.get(url)
    }

    /// Retrieve download stats for a crate.
    pub fn crate_downloads(&self, name: &str) -> Result<Downloads, Error> {
        let url = self.base_url.join(&format!("crates/{}/downloads", name))?;
        self.get(url)
    }

    /// Retrieve the owners of a crate.
    pub fn crate_owners(&self, name: &str) -> Result<Vec<User>, Error> {
        let url = self.base_url.join(&format!("crates/{}/owners", name))?;
        let resp: Owners = self.get(url)?;
        Ok(resp.users)
    }

    fn crate_reverse_dependencies_page(
        &self,
        crate_name: &str,
        page: u64,
    ) -> Result<ReverseDependenciesAsReceived, Error> {
        let url = self.base_url.join(&format!(
            "crates/{}/reverse_dependencies?per_page=100&page={}",
            crate_name, page
        ))?;
        self.get(url)
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

            deps.extend(page);
        }
        Ok(deps)
    }

    /// Get the total count of reverse dependencies for a given crate.
    pub fn crate_reverse_dependency_count(&self, crate_name: &str) -> Result<u64, Error> {
        let page = self.crate_reverse_dependencies_page(crate_name, 1)?;
        Ok(page.meta.total)
    }

    /// Retrieve the authors for a crate version.
    pub fn crate_authors(&self, name: &str, version: &str) -> Result<Authors, Error> {
        let url = self
            .base_url
            .join(&format!("crates/{}/{}/authors", name, version))?;
        let res: AuthorsResponse = self.get(url)?;
        Ok(Authors {
            names: res.meta.names,
            users: res.users,
        })
    }

    /// Retrieve the dependencies of a crate version.
    pub fn crate_dependencies(&self, name: &str, version: &str) -> Result<Vec<Dependency>, Error> {
        let url = self
            .base_url
            .join(&format!("crates/{}/{}/dependencies", name, version))?;
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
    /// # use crates_io_api::{SyncClient, ListOptions, Sort, Error};
    ///
    /// # fn f() -> Result<(), Error> {
    /// # let client = SyncClient::new( "my-bot-name (my-contact@domain.com)", std::time::Duration::from_millis(1000))?;
    /// client.crates(ListOptions{
    ///   sort: Sort::Alphabetical,
    ///   per_page: 100,
    ///   page: 1,
    ///   query: Some("api".to_string()),
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn crates(&self, spec: ListOptions) -> Result<CratesResponse, Error> {
        let mut url = self.base_url.join("crates")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("page", &spec.page.to_string());
            q.append_pair("per_page", &spec.per_page.to_string());

            if spec.sort != Sort::Relevance {
                q.append_pair("sort", spec.sort.to_str());
            }

            if let Some(id) = spec.user_id {
                q.append_pair("user_id", &id.to_string());
            }
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
    pub fn all_crates(&self, query: Option<String>) -> Result<Vec<Crate>, Error> {
        let mut page = 1;
        let mut crates = Vec::new();
        loop {
            let res = self.crates(ListOptions {
                query: query.clone(),
                sort: Sort::Alphabetical,
                per_page: 100,
                page,
                user_id: None,
            })?;
            if !res.crates.is_empty() {
                crates.extend(res.crates);
                page += 1;
            } else {
                break;
            }
        }
        Ok(crates)
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

        let res = client.crates(ListOptions {
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
