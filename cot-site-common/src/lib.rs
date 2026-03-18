pub mod md_pages;
mod utils;
pub use utils::{Version, VersionError};

pub const MASTER_VERSION: &str = "master";
pub const LATEST_VERSION: &str = "v0.6";
pub const ALL_VERSIONS: &[&str] = &[
    MASTER_VERSION,
    LATEST_VERSION,
    "v0.5",
    "v0.4",
    "v0.3",
    "v0.2",
    "v0.1",
];
