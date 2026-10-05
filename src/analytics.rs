use std::sync::OnceLock;

use cot::request::RequestHead;
use cot::request::extractors::FromRequestHead;

static CONFIG: OnceLock<Result<AnalyticsConfig, &'static str>> = OnceLock::new();
const DEFAULT_PROJECT_TOKEN: &str = "phc_CpMFffYnhPmXZrMy8oh8TqeB2kK4EopYaMft3yhgtHiY";

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
    ) -> Result<Self, &'static str> {
        let enabled = parse_bool(enabled)?;
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
                "POSTHOG_PROJECT_TOKEN must be a public phc_ project token, not a personal API key",
            );
        }
        let host = match host.unwrap_or_default().trim().trim_end_matches('/') {
            "https://us.i.posthog.com" => "https://us.i.posthog.com",
            "https://eu.i.posthog.com" => "https://eu.i.posthog.com",
            _ => return Err("POSTHOG_HOST must be a US or EU PostHog Cloud host"),
        };
        Ok(Self {
            enabled,
            project_token: token.to_owned(),
            host: host.to_owned(),
            allow_localhost: parse_bool(allow_localhost)?,
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
                *message,
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
    fn disabled_by_default_and_ignores_credentials() {
        let config = AnalyticsConfig::parse(None, Some("phx_secret"), None, None).unwrap();
        assert!(!config.enabled);
        assert!(config.project_token.is_empty());
    }

    #[test]
    fn enabled_requires_public_token_and_matching_cloud_host() {
        for host in ["https://us.i.posthog.com", "https://eu.i.posthog.com/"] {
            let config =
                AnalyticsConfig::parse(Some("true"), Some("phc_test"), Some(host), None).unwrap();
            assert!(config.enabled);
            assert!(!config.allow_localhost);
            assert!(config.host.ends_with(".i.posthog.com"));
        }
        for token in [None, Some("")] {
            let config =
                AnalyticsConfig::parse(Some("true"), token, Some("https://us.i.posthog.com"), None)
                    .unwrap();
            assert_eq!(config.project_token, DEFAULT_PROJECT_TOKEN);
        }
        for token in [Some("phx_secret"), Some("phc_"), Some("phc_\"><script>")] {
            assert!(
                AnalyticsConfig::parse(Some("true"), token, Some("https://us.i.posthog.com"), None)
                    .is_err()
            );
        }
        assert!(
            AnalyticsConfig::parse(
                Some("true"),
                Some("phc_test"),
                Some("https://example.com"),
                None
            )
            .is_err()
        );
        for dashboard_host in ["https://us.posthog.com", "https://eu.posthog.com"] {
            assert!(
                AnalyticsConfig::parse(Some("true"), Some("phc_test"), Some(dashboard_host), None)
                    .is_err()
            );
        }
        assert!(AnalyticsConfig::parse(Some("maybe"), None, None, None).is_err());
    }
}
