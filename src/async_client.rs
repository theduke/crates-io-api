use futures::{future, stream, Future, Stream};
use log::trace;
use reqwest::{header, r#async, StatusCode, Url};
use serde::de::DeserializeOwned;

use super::Error;
use crate::types::*;

/// Asynchronous client for the crates.io API.
#[derive(Clone)]
pub struct Client {
    client: r#async::Client,
    base_url: Url,
}

impl Client {
    /// Instantiate a new client.
    ///
    /// This will fail if the underlying http client could not be created.
    pub fn new() -> Self {
        Self {
            client: r#async::Client::new(),
            base_url: Url::parse("https://crates.io/api/v1/").unwrap(),
        }
    }

    pub fn with_user_agent(user_agent: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );
        Self {
            client: r#async::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            base_url: Url::parse("https://crates.io/api/v1/").unwrap(),
        }
    }

    fn get<T: DeserializeOwned>(&self, url: &Url) -> impl Future<Item = T, Error = Error> {
        trace!("GET {}", url);

        self.client
            .get(url.clone())
            .send()
            .map_err(Error::from)
            .and_then(|res| {
                if res.status() == StatusCode::NOT_FOUND {
                    return Err(Error::NotFound);
                }
                let res = res.error_for_status()?;
                Ok(res)
            })
            .and_then(|mut res| res.json().map_err(Error::from))
    }

    /// Retrieve a summary containing crates.io wide information.
    pub fn summary(&self) -> impl Future<Item = Summary, Error = Error> {
        let url = self.base_url.join("summary").unwrap();
        self.get(&url)
    }

    /// Retrieve information of a crate.
    ///
    /// If you require detailed information, consider using [full_crate]().
    pub fn get_crate(&self, name: &str) -> impl Future<Item = CrateResponse, Error = Error> {
        let url = self.base_url.join("crates/").unwrap().join(name).unwrap();
        self.get(&url)
    }

    /// Retrieve download stats for a crate.
    pub fn crate_downloads(&self, name: &str) -> impl Future<Item = Downloads, Error = Error> {
        let url = self
            .base_url
            .join(&format!("crates/{}/downloads", name))
            .unwrap();
        self.get(&url)
    }

    /// Retrieve the owners of a crate.
    pub fn crate_owners(&self, name: &str) -> impl Future<Item = Vec<User>, Error = Error> {
        let url = self
            .base_url
            .join(&format!("crates/{}/owners", name))
            .unwrap();
        self.get::<Owners>(&url).map(|data| data.users)
    }

    /// Load all reverse dependencies of a crate.
    ///
    /// Note: Since the reverse dependency endpoint requires pagination, this
    /// will result in multiple requests if the crate has more than 100 reverse
    /// dependencies.
    pub fn crate_reverse_dependencies(&self, name: &str)
        -> impl Future<Item = ReverseDependencies, Error = Error> {

        fn fetch_page(c: Client, name: String, mut tidy_rdeps: ReverseDependencies, page: u64)
            -> impl Future<Item = ReverseDependencies, Error = Error> + Send {

            let url = c.base_url.join(&format!(
                "crates/{0}/reverse_dependencies?per_page=100&page={1}", name, page
            )).unwrap();

            c.get::<ReverseDependenciesAsReceived>(&url).and_then(move |rdeps|
                -> Box<Future<Item = ReverseDependencies, Error = Error> + Send> {

                for d in rdeps.dependencies.iter() {
                    for v in rdeps.versions.iter() {
                        if v.id == d.version_id {
                            // Right now it iterates over the full vector for each vector element.
                            // For large vectors, it may be faster to remove each matched element
                            // using the drain_filter() method once it's stabilized:
                            // https://doc.rust-lang.org/nightly/std/vec/struct.Vec.html#method.drain_filter
                            tidy_rdeps.dependencies.push(
                                ReverseDependency {crate_version: v.clone(), dependency: d.clone()}
                            );
                        }
                    }
                }
                if !rdeps.dependencies.is_empty() {
                    tidy_rdeps.meta = rdeps.meta;
                    Box::new(fetch_page(c, name, tidy_rdeps, page + 1))
                } else {
                    Box::new(::futures::future::ok(tidy_rdeps))
                }
            })
        }

        fetch_page(self.clone(), name.to_string(), ReverseDependencies {
            dependencies: Vec::new(), meta:Meta{total:0} }, 1)
    }

    /// Retrieve the authors for a crate version.
    pub fn crate_authors(
        &self,
        name: &str,
        version: &str,
    ) -> impl Future<Item = Authors, Error = Error> {
        let url = self
            .base_url
            .join(&format!("crates/{}/{}/authors", name, version))
            .unwrap();
        self.get::<AuthorsResponse>(&url).map(|res| Authors {
            names: res.meta.names,
            users: res.users,
        })
    }

    /// Retrieve the dependencies of a crate version.
    pub fn crate_dependencies(
        &self,
        name: &str,
        version: &str,
    ) -> impl Future<Item = Vec<Dependency>, Error = Error> {
        let url = self
            .base_url
            .join(&format!("crates/{}/{}/dependencies", name, version))
            .unwrap();
        self.get::<Dependencies>(&url).map(|res| res.dependencies)
    }

    fn full_version(&self, version: Version) -> impl Future<Item = FullVersion, Error = Error> {
        let authors = self.crate_authors(&version.crate_name, &version.num);
        let deps = self.crate_dependencies(&version.crate_name, &version.num);

        authors.join(deps).map(|(authors, deps)| FullVersion {
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
        })
    }

    /// Retrieve all available information for a crate, including download
    /// stats,  owners and reverse dependencies.
    ///
    /// The `all_versions` argument controls the retrieval of detailed version
    /// information.
    /// If false, only the data for the latest version will be fetched, if true,
    /// detailed information for all versions will be available.
    /// Note: Each version requires two extra requests.
    pub fn full_crate(
        &self,
        name: &str,
        all_versions: bool,
    ) -> impl Future<Item = FullCrate, Error = Error> {
        let c = self.clone();
        let crate_and_versions = self.get_crate(name).and_then(
            move |info| -> Box<
                Future<Item = (CrateResponse, Vec<FullVersion>), Error = Error> + Send,
            > {
                if !all_versions {
                    Box::new(
                        c.full_version(info.versions[0].clone())
                            .map(|v| (info, vec![v])),
                    )
                } else {
                    Box::new(
                        ::futures::future::join_all(
                            info.versions
                                .clone()
                                .into_iter()
                                .map(|v| c.full_version(v))
                                .collect::<Vec<_>>(),
                        )
                        .map(|versions| (info, versions)),
                    )
                }
            },
        );

        let dls = self.crate_downloads(name);
        let owners = self.crate_owners(name);
        let reverse_dependencies = self.crate_reverse_dependencies(name);

        crate_and_versions
            .join4(dls, owners, reverse_dependencies)
            .map(|((resp, versions), dls, owners, reverse_dependencies)| {
                let data = resp.crate_data;
                FullCrate {
                    id: data.id,
                    name: data.name,
                    description: data.description,
                    license: resp.versions[0].license.clone(),
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
                }
            })
    }

    /// Retrieve a page of crates, optionally constrained by a query.
    ///
    /// If you want to get all results without worrying about paging,
    /// use [all_crates]().
    ///
    /// ```
    pub fn crates(&self, spec: ListOptions) -> impl Future<Item = CratesResponse, Error = Error> {
        let mut url = self.base_url.join("crates").unwrap();
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("page", &spec.page.to_string());
            q.append_pair("per_page", &spec.per_page.to_string());
            q.append_pair("sort", spec.sort.to_str());
            if let Some(query) = spec.query {
                q.append_pair("q", &query);
            }
        }
        self.get(&url)
    }

    /// Retrieve all crates, optionally constrained by a query.
    ///
    /// Note: This method fetches all pages of the result.
    /// This can result in a lot queries (100 results per query).
    pub fn all_crates(&self, query: Option<String>) -> impl Stream<Item = Crate, Error = Error> {
        let opts = ListOptions {
            query,
            sort: Sort::Alphabetical,
            per_page: 100,
            page: 1,
        };

        let c = self.clone();
        self.crates(opts.clone())
            .and_then(move |res| {
                let pages = (res.meta.total as f64 / 100.0).ceil() as u64;
                let streams_futures = (1..pages)
                    .map(|page| {
                        let opts = ListOptions {
                            page,
                            ..opts.clone()
                        };
                        c.crates(opts)
                            .and_then(|res| future::ok(stream::iter_ok(res.crates)))
                    })
                    .collect::<Vec<_>>();
                let stream = stream::futures_ordered(streams_futures).flatten();
                future::ok(stream)
            })
            .flatten_stream()
    }

    /// Retrieve all crates with all available extra information.
    ///
    /// Note: This method fetches not only all crates, but does multiple requests for each crate
    /// to retrieve extra information.
    ///
    /// This can result in A LOT of queries.
    pub fn all_crates_full(
        &self,
        query: Option<String>,
        all_versions: bool,
    ) -> impl Stream<Item = FullCrate, Error = Error> {
        let c = self.clone();
        self.all_crates(query)
            .and_then(move |cr| c.full_crate(&cr.name, all_versions))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_client() {
        let mut rt = ::tokio::runtime::Runtime::new().unwrap();

        let client = Client::new();

        let summary = rt.block_on(client.summary()).unwrap();
        assert!(summary.most_downloaded.len() > 0);

        for item in &summary.most_downloaded[0..3] {
            let _ = rt.block_on(client.full_crate(&item.name, false)).unwrap();
        }

        let crates = rt
            .block_on(client.all_crates(None).take(3).collect())
            .unwrap();
        println!("{:?}", crates);
    }
}
