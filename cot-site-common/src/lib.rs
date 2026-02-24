pub mod md_pages;

pub const MASTER_VERSION: &str = "master";
pub const LATEST_VERSION: &str = "v0.5";
pub const ALL_VERSIONS: &[&str] = &[MASTER_VERSION, "v0.5", "v0.4", "v0.3", "v0.2", "v0.1"];

use std::fmt::Display;
use std::str::FromStr;

use semver::Version as SemverVersion;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VersionError {
    #[error("invalid version string: {0}")]
    InvalidVersion(#[from] semver::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version(SemverVersion);

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version(SemverVersion::new(major, minor, patch))
    }

    pub fn major(&self) -> u64 {
        self.0.major
    }

    pub fn minor(&self) -> u64 {
        self.0.minor
    }

    pub fn patch(&self) -> u64 {
        self.0.patch
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // replace "master" with latest version
        let s = if s == MASTER_VERSION || s == "" {
            LATEST_VERSION
        } else {
            s
        };

        let s = s.trim_start_matches('v');

        // canonicalize version string by adding ".0" for missing minor/patch
        let parts: Vec<&str> = s.split('.').collect();
        let s = match parts.len() {
            1 => format!("{}.0.0", parts[0]),
            2 => format!("{}.{}.0", parts[0], parts[1]),
            _ => s.to_string(),
        };

        let semver_version = SemverVersion::parse(s.as_str())?;
        Ok(Version(semver_version))
    }
}
