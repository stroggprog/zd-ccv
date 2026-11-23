//! Application version.
//!
//! The version is the single source of truth for both the `--version`
//! flag and the About dialog. Bump policy (see AGENTS.md):
//!
//! - `major` and `minor` change only when the user instructs it.
//! - `build` is incremented by one every time the project is built.

/// The application version with its three fields.
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl Version {
    /// The version in standard `major.minor.build` format.
    pub fn string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.build)
    }

    /// The version as a leaked static string, for APIs that require a
    /// `&'static str` (e.g. the clap `#[command(version = ...)]` attribute).
    pub fn leaked(&self) -> &'static str {
        Box::leak(self.string().into_boxed_str())
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.build)
    }
}

/// The current application version: the initial build number equals the
/// commit count at introduction (28) plus one for that build.
pub const VERSION: Version = Version {
    major: 1,
    minor: 0,
    build: 35,
};

#[cfg(test)]
mod tests {
    use super::{Version, VERSION};

    #[test]
    fn version_formats_as_major_dot_minor_dot_build() {
        let version = Version {
            major: 1,
            minor: 2,
            build: 3,
        };
        assert_eq!(version.string(), "1.2.3");
        assert_eq!(version.to_string(), "1.2.3");
    }

    /// The Cargo manifest version must always mirror this struct; if
    /// this test fails, `Cargo.toml` was not updated along with
    /// `src/version.rs`.
    #[test]
    fn cargo_manifest_version_stays_in_sync() {
        assert_eq!(env!("CARGO_PKG_VERSION"), VERSION.string());
    }
}
