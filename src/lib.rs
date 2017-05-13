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

// use std::time::{Duration, Instant};
use std::iter::Extend;

use serde::de::DeserializeOwned;
use reqwest::{Client, Url, StatusCode, UrlError};

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

pub struct Pagination {
    pub per_page: u64,
    pub page: u64,
}

pub struct CratesIO {
    client: Client,
    base_url: Url,
}

impl CratesIO {
    pub fn new() -> Result<Self> {
        let c = CratesIO {
            client: Client::new()?,
            base_url: Url::parse("https://crates.io/api/v1/").unwrap(),
        };
        Ok(c)
    }

    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T> {
        println!("GETTING {}", url);
        let mut res = self.client.get(url).send()?;
        if !res.status().is_success() {
            if res.status() == &StatusCode::NotFound {
                Err(ErrorKind::NotFound.into())
            } else {
                Err(ErrorKind::ServerError.into())
            }
        } else {
            let data: T = res.json()?;
            Ok(data)
        }
    }

    /*
    fn get_all<T: DeserializeOwned>(&self, url: Url) -> Result<Vec<T>> {
        let mut items = Vec::<T>::new();

        let per_page = 100;
        let mut page = 1;

        loop {
            let mut paged_url = url.clone();
            paged_url.query_pairs_mut()
                     .append_pair("page", &page.to_string())
                     .append_pair("per_page", &per_page.to_string());
            let data = self.get(paged_url)?;
            items.push(data);
            break;
        }

        Ok(items)
    }
    */

    pub fn summary(&self) -> Result<Summary> {
        let url = Url::parse("https://crates.io/summary")?;
        self.get(url)
    }

    pub fn get_crate(&self, name: &str) -> Result<CrateResponse> {
        let url = self.base_url.join("crates/")?.join(name)?;
        self.get(url)
    }

    pub fn crate_downloads(&self, name: &str) -> Result<Downloads> {
        let url = self.base_url.join(
            &format!("crates/{}/downloads", name))?;
        self.get(url)
    }

    pub fn crate_owners(&self, name: &str) -> Result<Vec<User>> {
        let url = self.base_url.join(
            &format!("crates/{}/owners", name))?;
        let resp: Owners = self.get(url)?;
        Ok(resp.users)
    }

    pub fn crate_reverse_dependencies(&self, name: &str) -> Result<Vec<Dependency>> {
        let mut page = 1;
        let mut deps = Vec::new();
        loop {
            let url = self.base_url.join(
                &format!("crates/{}/reverse_dependencies?per_page=100&page={}", name, page))?;
            let res: Dependencies = self.get(url)?;
            if res.dependencies.len() > 0 {
                deps.extend(res.dependencies);
                page += 1;
            } else {
                break;
            }
        }
        Ok(deps)
    }

    pub fn crate_authors(&self, name: &str, version: &str) -> Result<Authors> {
        let url = self.base_url.join(
            &format!("crates/{}/{}/authors", name, version))?;
        self.get(url)
    }

    pub fn crate_dependencies(&self, name: &str, version: &str) -> Result<Vec<Dependency>> {
        let url = self.base_url.join(
            &format!("crates/{}/{}/dependencies", name, version))?;
        let resp: Dependencies = self.get(url)?;
        Ok(resp.dependencies)
    }


    fn full_version(&self, version: Version) -> Result<FullVersion> {
        let authors = self.crate_authors(&version.crate_name, &version.num)?;
        let deps = self.crate_dependencies(&version.crate_name, &version.num)?;

        let v = FullVersion{
            created_at: version.created_at,
            updated_at: version.updated_at,
            dl_path: version.dl_path,
            downloads: version.downloads,
            features: version.features,
            id: version.id,
            num: version.num,
            yanked: version.yanked,
            links: version.links,

            author_names: authors.meta.names,
            authors: authors.users,
            dependencies: deps,
        };
        Ok(v)
    }

    pub fn full_crate(&self, name: &str) -> Result<FullCrate> {
        let resp = self.get_crate(&name)?;
        let data = resp.crate_data;

        let dls = self.crate_downloads(name)?;
        let owners = self.crate_owners(name)?;
        let reverse_dependencies = self.crate_reverse_dependencies(name)?;

        let versions_res: Result<Vec<FullVersion>> = resp.versions.into_iter().map(|v| {
            self.full_version(v)
        }).collect();
        let versions = versions_res?;

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
}

#[cfg(test)]
mod test {
    use super::*;
}

