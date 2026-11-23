//! Update checks against the GitHub repository.
//!
//! The About dialog offers a "Check 4 Update" action: fetch the
//! `Cargo.toml` from the repository's default branch (`main`), read the
//! `[package] version`, and compare it with the local `Version`.

use crate::version::{Version, VERSION};
use std::time::Duration;

/// The GitHub repository page, opened by the About dialog's hyperlink.
pub const REPO_URL: &str = "https://github.com/stroggprog/zd-ccv";

/// The raw `Cargo.toml` of the repository's default branch.
pub const CARGO_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/stroggprog/zd-ccv/main/Cargo.toml";

const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

pub type SemVer = (u32, u32, u32);

fn to_triple(version: &Version) -> SemVer {
    (version.major, version.minor, version.build)
}

fn format_triple((major, minor, build): SemVer) -> String {
    format!("{major}.{minor}.{build}")
}

/// Parse the `version = "x.y.z"` entry from the `[package]` section of a
/// Cargo.toml. Dependency sections may define their own `version` keys;
/// only the package section counts.
pub fn parse_package_version(text: &str) -> Option<SemVer> {
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(rest) = line.strip_prefix("version") else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let value = rest.split('#').next().unwrap_or("").trim();
        let quoted = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .unwrap_or(value);
        let mut parts = quoted.split('.');
        let (Some(major), Some(minor), Some(build)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if parts.next().is_some() {
            continue;
        }
        let triple = (
            major.parse::<u32>().ok()?,
            minor.parse::<u32>().ok()?,
            build.parse::<u32>().ok()?,
        );
        return Some(triple);
    }
    None
}

/// Fetch the repository manifest and return its package version.
pub fn fetch_remote_version() -> Result<SemVer, String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(FETCH_TIMEOUT))
        .build();
    let agent: ureq::Agent = config.into();
    let mut response = agent
        .get(CARGO_MANIFEST_URL)
        .call()
        .map_err(|e| e.to_string())?;
    if response.status().as_u16() != 200 {
        return Err(format!("HTTP status {}", response.status()));
    }
    let text = response
        .body_mut()
        .with_config()
        .limit(MAX_MANIFEST_BYTES)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    parse_package_version(&text).ok_or_else(|| "no package version in manifest".to_string())
}

/// Run an update check: returns a one-line user message and whether it
/// reports a failure (fetch or parse).
pub fn check_outcome() -> (String, bool) {
    let local = to_triple(&VERSION);
    match fetch_remote_version() {
        Err(e) => (format!("Update check failed: {e}"), true),
        Ok(remote) if remote > local => (
            format!(
                "A new version is available: {} (you have {}).",
                format_triple(remote),
                VERSION
            ),
            false,
        ),
        Ok(_) => (format!("Version {} is up to date.", VERSION), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_package_version_only() {
        let manifest = r#"
[package]
name = "zd-ccv"
version = "1.2.33"
edition = "2021"

[dependencies]
ureq = { version = "3", features = [] }

[profile.release]
opt-level = 3
"#;
        assert_eq!(parse_package_version(manifest), Some((1, 2, 33)));
    }

    #[test]
    fn handles_spacing_and_comments() {
        let manifest = "[package]\nname=\"x\"\n  version = \"0.1.4\"  # trailing\n[lib]";
        assert_eq!(parse_package_version(manifest), Some((0, 1, 4)));
    }

    #[test]
    fn rejects_missing_or_malformed_versions() {
        assert_eq!(parse_package_version("[package]\nname=\"x\"\n"), None);
        assert_eq!(
            parse_package_version("[dependencies]\nversion=\"9.9.9\""),
            None
        );
        assert_eq!(
            parse_package_version("[package]\nversion=\"1.2.3.4\"\n"),
            None
        );
        assert_eq!(parse_package_version("[package]\nversion=\"abc\"\n"), None);
    }

    #[test]
    fn comparison_uses_the_full_triple() {
        let local = to_triple(&VERSION);
        let mut newer = local;
        newer.2 += 1;
        assert!(newer > local);
        let mut same_as_local = local;
        assert!(!(same_as_local > local));
        same_as_local.0 += 1;
        assert!(same_as_local > local);
    }

    /// Real network check against GitHub; run manually with
    /// `cargo test update_network_check -- --ignored`.
    #[test]
    #[ignore]
    fn update_network_check() {
        let (message, _is_error) = check_outcome();
        eprintln!("{message}");
    }
}
