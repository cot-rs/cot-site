use std::fmt::Display;
use std::str::FromStr;

use semver::Version as SemverVersion;
use thiserror::Error;

use crate::{LATEST_VERSION, MASTER_VERSION};

/// Errors related to version parsing and handling.
#[derive(Debug, Error)]
pub enum VersionError {
    /// Error parsing version string.
    #[error("invalid version string: {0}")]
    InvalidVersion(#[from] semver::Error),
}

/// A semver version type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version(SemverVersion);

impl Version {
    /// Creates a new version from major, minor, and patch numbers.
    ///
    /// # Example
    /// ```
    /// let v = Version::new(0, 5, 0);
    /// assert_eq!(v.to_string(), "0.5.0");
    /// ```
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version(SemverVersion::new(major, minor, patch))
    }

    /// Returns the major version number.
    ///
    /// # Example
    /// ```
    /// let v = Version::new(0, 5, 0);
    /// assert_eq!(v.major(), 0);
    /// ```
    pub fn major(&self) -> u64 {
        self.0.major
    }

    /// Returns the minor version number.
    ///
    /// # Example
    /// ```
    /// let v = Version::new(0, 5, 0);
    /// assert_eq!(v.minor(), 5);
    /// ```
    pub fn minor(&self) -> u64 {
        self.0.minor
    }

    /// Returns the patch version number.
    ///
    /// # Example
    /// ```
    /// let v = Version::new(0, 5, 0);
    /// assert_eq!(v.patch(), 0);
    /// ```
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

        // canonicalize version string by adding ".0" for missing minor/patch
        let s = canonicalize_version_string(s);

        let semver_version = SemverVersion::parse(s.as_str())?;
        Ok(Version(semver_version))
    }
}

fn canonicalize_version_string(s: &str) -> String {
    let s = s.trim_start_matches('v');
    let parts: Vec<&str> = s.split('.').collect();
    match parts.len() {
        1 => format!("{}.0.0", parts[0]),
        2 => format!("{}.{}.0", parts[0], parts[1]),
        _ => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version_parsing() {
        let v = Version::from_str("v0.5").unwrap();
        assert_eq!(v.to_string(), "0.5.0");

        let v = Version::from_str("0.5").unwrap();
        assert_eq!(v.to_string(), "0.5.0");

        let v = Version::from_str("master").unwrap();
        assert_eq!(v.to_string(), canonicalize_version_string(LATEST_VERSION));

        let v = Version::from_str("").unwrap();
        assert_eq!(v.to_string(), canonicalize_version_string(LATEST_VERSION));
    }

    #[test]
    fn test_canonicalize_version_string() {
        assert_eq!(canonicalize_version_string("v0.5"), "0.5.0");
        assert_eq!(canonicalize_version_string("0.5"), "0.5.0");
        assert_eq!(canonicalize_version_string("v0.5.1"), "0.5.1");
        assert_eq!(canonicalize_version_string("0.5.1"), "0.5.1");
    }
}
