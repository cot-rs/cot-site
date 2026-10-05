use std::sync::OnceLock;

use cot::request::RequestHead;
use cot::request::extractors::FromRequestHead;

static CONFIG: OnceLock<Result<AnalyticsConfig, String>> = OnceLock::new();
const DEFAULT_ENABLED: bool = true;
const DEFAULT_PROJECT_TOKEN: &str = "phc_CpMFffYnhPmXZrMy8oh8TqeB2kK4EopYaMft3yhgtHiY";
const US_POSTHOG_HOST: &str = "https://us.i.posthog.com";
const EU_POSTHOG_HOST: &str = "https://eu.i.posthog.com";
const DEFAULT_HOST: &str = EU_POSTHOG_HOST;
const DEFAULT_ALLOW_LOCALHOST: bool = false;

#[derive(Debug, Clone, Default)]
pub(crate) struct AnalyticsConfig {
    pub enabled: bool,
    pub project_token: String,
    pub host: String,
    pub allow_localhost: bool,
}

impl AnalyticsConfig {
    fn parse(
        enabled: Option<&str>,
        token: Option<&str>,
        host: Option<&str>,
        allow_localhost: Option<&str>,
    ) -> Result<Self, String> {
        let enabled = match enabled {
            Some(value) => parse_bool(Some(value))?,
            None => DEFAULT_ENABLED,
        };
        if !enabled {
            return Ok(Self::default());
        }
        let configured_token = token.unwrap_or_default().trim();
        let token = if configured_token.is_empty() {
            DEFAULT_PROJECT_TOKEN
        } else {
            configured_token
        };
        if !token.starts_with("phc_")
            || token.len() <= 4
            || !token
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err(
                "POSTHOG_PROJECT_TOKEN must be a public phc_ project token, not a personal API key"
                    .to_owned(),
            );
        }
        let configured_host = host
            .map(str::trim)
            .filter(|host| !host.is_empty())
            .unwrap_or(DEFAULT_HOST);
        let host = match configured_host.trim_end_matches('/') {
            US_POSTHOG_HOST => US_POSTHOG_HOST,
            EU_POSTHOG_HOST => EU_POSTHOG_HOST,
            other => {
                return Err(format!(
                    "Unknown POSTHOG_HOST {other:?}; expected {US_POSTHOG_HOST} or {EU_POSTHOG_HOST}"
                ));
            }
        };
        let allow_localhost = match allow_localhost {
            Some(value) => parse_bool(Some(value))?,
            None => DEFAULT_ALLOW_LOCALHOST,
        };
        Ok(Self {
            enabled,
            project_token: token.to_owned(),
            host: host.to_owned(),
            allow_localhost,
        })
    }
}

fn parse_bool(value: Option<&str>) -> Result<bool, &'static str> {
    match value {
        None | Some("false") | Some("0") | Some("") => Ok(false),
        Some("true") | Some("1") => Ok(true),
        _ => Err("PostHog boolean settings must be true, false, 1, or 0"),
    }
}

pub(crate) fn configuration() -> cot::Result<&'static AnalyticsConfig> {
    CONFIG
        .get_or_init(|| {
            let enabled = std::env::var("POSTHOG_ENABLED").ok();
            let token = std::env::var("POSTHOG_PROJECT_TOKEN").ok();
            let host = std::env::var("POSTHOG_HOST").ok();
            let allow_localhost = std::env::var("POSTHOG_ALLOW_LOCALHOST").ok();
            AnalyticsConfig::parse(
                enabled.as_deref(),
                token.as_deref(),
                host.as_deref(),
                allow_localhost.as_deref(),
            )
        })
        .as_ref()
        .map_err(|message| {
            cot::Error::internal(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                message.clone(),
            ))
        })
}

impl FromRequestHead for AnalyticsConfig {
    async fn from_request_head(_head: &RequestHead) -> cot::Result<Self> {
        configuration().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_by_default_uses_the_builtin_project_and_eu_host() {
        let config = AnalyticsConfig::parse(None, None, None, None).unwrap();
        assert_eq!(config.enabled, DEFAULT_ENABLED);
        assert_eq!(config.project_token, DEFAULT_PROJECT_TOKEN);
        assert_eq!(config.host, DEFAULT_HOST);
        assert_eq!(config.allow_localhost, DEFAULT_ALLOW_LOCALHOST);

        let blank_host = AnalyticsConfig::parse(Some("true"), None, Some("  "), None).unwrap();
        assert_eq!(blank_host.host, DEFAULT_HOST);
    }

    #[test]
    fn can_be_disabled_and_ignores_credentials() {
        let config = AnalyticsConfig::parse(Some("false"), Some("phx_secret"), None, None).unwrap();
        assert!(!config.enabled);
        assert!(config.project_token.is_empty());
    }

    #[test]
    fn enabled_requires_public_token_and_matching_cloud_host() {
        for host in [US_POSTHOG_HOST, EU_POSTHOG_HOST] {
            let config =
                AnalyticsConfig::parse(Some("true"), Some("phc_test"), Some(host), None).unwrap();
            assert!(config.enabled);
            assert!(!config.allow_localhost);
            assert!(config.host.ends_with(".i.posthog.com"));
        }
        let eu_host_with_slash = format!("{EU_POSTHOG_HOST}/");
        let config = AnalyticsConfig::parse(
            Some("true"),
            Some("phc_test"),
            Some(&eu_host_with_slash),
            None,
        )
        .unwrap();
        assert_eq!(config.host, EU_POSTHOG_HOST);
        for token in [None, Some("")] {
            let config =
                AnalyticsConfig::parse(Some("true"), token, Some(US_POSTHOG_HOST), None).unwrap();
            assert_eq!(config.project_token, DEFAULT_PROJECT_TOKEN);
        }
        for token in [Some("phx_secret"), Some("phc_"), Some("phc_\"><script>")] {
            assert!(
                AnalyticsConfig::parse(Some("true"), token, Some(US_POSTHOG_HOST), None).is_err()
            );
        }
        let error = AnalyticsConfig::parse(
            Some("true"),
            Some("phc_test"),
            Some("https://example.com"),
            None,
        )
        .unwrap_err();
        assert!(error.contains("https://example.com"));
        for dashboard_host in ["https://us.posthog.com", "https://eu.posthog.com"] {
            assert!(
                AnalyticsConfig::parse(Some("true"), Some("phc_test"), Some(dashboard_host), None)
                    .is_err()
            );
        }
        assert!(AnalyticsConfig::parse(Some("maybe"), None, None, None).is_err());
    }
}
