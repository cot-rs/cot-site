use std::sync::OnceLock;

use cot::config::ProjectConfig;
use cot::request::RequestHead;
use cot::request::extractors::FromRequestHead;
use serde::Deserialize;

static CONFIG: OnceLock<AnalyticsConfig> = OnceLock::new();

// https://posthog.com/docs/privacy#is-it-ok-for-my-api-key-to-be-exposed-and-public
const DEFAULT_PROJECT_TOKEN: &str = "phc_CpMFffYnhPmXZrMy8oh8TqeB2kK4EopYaMft3yhgtHiY";
const US_POSTHOG_HOST: &str = "https://us.i.posthog.com";
const EU_POSTHOG_HOST: &str = "https://eu.i.posthog.com";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AnalyticsConfig {
    pub enabled: bool,
    pub project_token: String,
    pub host: String,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            project_token: DEFAULT_PROJECT_TOKEN.to_owned(),
            host: EU_POSTHOG_HOST.to_owned(),
        }
    }
}

impl TryFrom<&ProjectConfig> for AnalyticsConfig {
    type Error = String;

    fn try_from(config: &ProjectConfig) -> Result<Self, String> {
        let Some(table) = config.extra.get("posthog") else {
            return Ok(Self::default());
        };
        let analytics: Self = table
            .clone()
            .try_into()
            .map_err(|error| format!("invalid posthog config: {error}"))?;
        analytics.validate()?;
        Ok(analytics)
    }
}

impl AnalyticsConfig {
    fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if !self.project_token.starts_with("phc_") {
            return Err("posthog.project_token must be a public phc_ project token".to_owned());
        }
        let host = self.host.trim_end_matches('/');
        if host != US_POSTHOG_HOST && host != EU_POSTHOG_HOST {
            return Err(format!(
                "posthog.host must be {US_POSTHOG_HOST} or {EU_POSTHOG_HOST}, got {:?}",
                self.host
            ));
        }
        Ok(())
    }
}

pub(crate) fn init(config: &ProjectConfig) -> cot::Result<()> {
    let analytics = AnalyticsConfig::try_from(config).map_err(cot::Error::internal)?;
    CONFIG
        .set(analytics)
        .expect("analytics config should be set once");
    Ok(())
}

impl FromRequestHead for AnalyticsConfig {
    async fn from_request_head(_head: &RequestHead) -> cot::Result<Self> {
        CONFIG
            .get()
            .cloned()
            .ok_or_else(|| cot::Error::internal("analytics config is not initialized"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(toml: &str) -> Result<AnalyticsConfig, String> {
        AnalyticsConfig::try_from(&ProjectConfig::from_toml(toml).unwrap())
    }

    #[test]
    fn missing_table_uses_defaults() {
        assert_eq!(parse("").unwrap(), AnalyticsConfig::default());
    }

    #[test]
    fn fields_can_be_overridden() {
        let config = parse(
            r#"
            [posthog]
            project_token = "phc_test"
            host = "https://us.i.posthog.com"
            "#,
        )
        .unwrap();
        assert!(config.enabled);
        assert_eq!(config.project_token, "phc_test");
        assert_eq!(config.host, US_POSTHOG_HOST);
    }

    #[test]
    fn can_be_disabled() {
        let config = parse("[posthog]\nenabled = false\nproject_token = \"phx_secret\"").unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(parse("[posthog]\nproject_token = \"phx_secret\"").is_err());
        assert!(parse("[posthog]\nhost = \"https://example.com\"").is_err());
        assert!(parse("[posthog]\nhost = \"https://eu.posthog.com\"").is_err());
        assert!(parse("[posthog]\nenabeld = false").is_err());
    }
}
