//! Helper functions for querying crate registries

use crate::types::*;
use reqwest::header;
use std::env;

/// Setup the headers for a sync or async request
pub fn setup_headers(
    user_agent: &str,
    registry: Option<&Registry>,
) -> Result<header::HeaderMap, header::InvalidHeaderValue> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_str(user_agent)?,
    );

    match &registry {
        Some(registry) => match &registry.name {
            Some(name) => {
                if let Ok(token) =
                    env::var(format!("CARGO_REGISTRIES_{}_TOKEN", name.to_uppercase()))
                {
                    headers.insert(
                        header::AUTHORIZATION,
                        header::HeaderValue::from_str(&token)?,
                    );
                }
            }
            None => match &registry.token {
                Some(token) => {
                    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(token)?);
                }
                None => (),
            },
        },
        None => (),
    }

    Ok(headers)
}

/// Determine the url of the crate registry being queried.
pub fn base_url(registry: Option<&Registry>) -> &str {
    match registry {
        Some(reg) => reg.url.as_str(),
        None => "https://crates.io/api/v1/",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Error;

    #[test]
    fn test_base_url_default() -> Result<(), Error> {
        assert_eq!(base_url(None), "https://crates.io/api/v1/");
        Ok(())
    }

    #[test]
    fn test_base_url_private() -> Result<(), Error> {
        let reg = &Registry {
            url: "https://crates.foobar.com/api/v1/".to_string(),
            name: None,
            token: None,
        };
        assert_eq!(base_url(Some(reg)), "https://crates.foobar.com/api/v1/");
        Ok(())
    }

    #[test]
    fn test_crates_io_headers() -> Result<(), Error> {
        let reg = None;
        let user_agent = "crates-io-api-continuous-integration (github.com/theduke/crates-io-api)";
        let headers = setup_headers(user_agent, reg).unwrap();

        let mut exp_headers = header::HeaderMap::new();
        exp_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );

        assert_eq!(headers, exp_headers);
        Ok(())
    }

    #[test]
    fn test_private_registry_name_headers() -> Result<(), Error> {
        let reg = &Registry {
            url: "https://crates.foobar.com/api/v1/".to_string(),
            name: Some("foobar".to_string()),
            token: None,
        };
        env::set_var("CARGO_REGISTRIES_FOOBAR_TOKEN", "baz");
        let user_agent = "crates-io-api-continuous-integration (github.com/theduke/crates-io-api)";
        let headers = setup_headers(user_agent, Some(reg)).unwrap();

        let mut exp_headers = header::HeaderMap::new();
        exp_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );
        exp_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str("baz").unwrap(),
        );

        assert_eq!(headers, exp_headers);
        Ok(())
    }

    #[test]
    fn test_private_registry_token_headers() -> Result<(), Error> {
        let reg = &Registry {
            url: "https://crates.foobar.com/api/v1/".to_string(),
            name: None,
            token: Some("foobar".to_string()),
        };
        env::set_var("CARGO_REGISTRIES_FOOBAR_TOKEN", "baz");
        let user_agent = "crates-io-api-continuous-integration (github.com/theduke/crates-io-api)";
        let headers = setup_headers(user_agent, Some(reg)).unwrap();

        let mut exp_headers = header::HeaderMap::new();
        exp_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );
        exp_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str("foobar").unwrap(),
        );

        assert_eq!(headers, exp_headers);
        Ok(())
    }
}
