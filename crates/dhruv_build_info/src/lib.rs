//! Build identity for dhruv public surfaces.
//!
//! Exposes the workspace library version and the git commit hash captured
//! at build time, so downstream consumers can record exactly which build
//! produced a computation (precalc provenance, cache invalidation).

/// Workspace library version (from `Cargo.toml` `[workspace.package]`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Git commit hash of the source tree at build time, or `"unknown"` when
/// the build did not run inside a git checkout.
pub const GIT_HASH: &str = env!("DHRUV_BUILD_GIT_HASH");

/// Library version string.
pub fn version() -> &'static str {
    VERSION
}

/// Git commit hash string (40 hex chars, or `"unknown"`).
pub fn git_hash() -> &'static str {
    GIT_HASH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty_semverish() {
        assert!(!version().is_empty());
        assert!(version().split('.').count() >= 2);
    }

    #[test]
    fn git_hash_is_hex_or_unknown() {
        let hash = git_hash();
        assert!(
            hash == "unknown" || (hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit())),
            "unexpected git hash: {hash}"
        );
    }
}
