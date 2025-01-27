use reqwest::Url;

use super::Error;

pub(crate) fn build_crate_url(base: &Url, crate_name: &str) -> Result<Url, Error> {
    let mut url = base.join("crates")?;
    url.path_segments_mut().unwrap().push(crate_name);

    // Guard against slashes in the crate name.
    // The API returns a nonsensical error in this case.
    if crate_name.contains('/') {
        Err(Error::NotFound(crate::error::NotFoundError {
            url: url.to_string(),
        }))
    } else {
        Ok(url)
    }
}

fn build_crate_url_nested(base: &Url, crate_name: &str) -> Result<Url, Error> {
    let mut url = base.join("crates")?;
    url.path_segments_mut().unwrap().push(crate_name).push("/");

    // Guard against slashes in the crate name.
    // The API returns a nonsensical error in this case.
    if crate_name.contains('/') {
        Err(Error::NotFound(crate::error::NotFoundError {
            url: url.to_string(),
        }))
    } else {
        Ok(url)
    }
}

pub(crate) fn build_crate_downloads_url(base: &Url, crate_name: &str) -> Result<Url, Error> {
    build_crate_url_nested(base, crate_name)?
        .join("downloads")
        .map_err(Error::from)
}

pub(crate) fn build_crate_owners_url(base: &Url, crate_name: &str) -> Result<Url, Error> {
    build_crate_url_nested(base, crate_name)?
        .join("owners")
        .map_err(Error::from)
}

pub(crate) fn build_crate_reverse_deps_url(
    base: &Url,
    crate_name: &str,
    page: u64,
) -> Result<Url, Error> {
    build_crate_url_nested(base, crate_name)?
        .join(&format!("reverse_dependencies?per_page=100&page={page}"))
        .map_err(Error::from)
}

pub(crate) fn build_crate_authors_url(
    base: &Url,
    crate_name: &str,
    version: &str,
) -> Result<Url, Error> {
    build_crate_url_nested(base, crate_name)?
        .join(&format!("{version}/authors"))
        .map_err(Error::from)
}

pub(crate) fn build_crate_dependencies_url(
    base: &Url,
    crate_name: &str,
    version: &str,
) -> Result<Url, Error> {
    build_crate_url_nested(base, crate_name)?
        .join(&format!("{version}/dependencies"))
        .map_err(Error::from)
}
