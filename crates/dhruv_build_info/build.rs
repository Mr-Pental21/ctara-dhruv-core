//! Capture the git commit hash at build time.
//!
//! Falls back to "unknown" outside a git checkout (e.g. released source
//! archives) so builds never fail on missing git state.

use std::process::Command;

fn git_hash() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let hash = String::from_utf8(output.stdout).ok()?;
    let hash = hash.trim();
    if hash.is_empty() {
        return None;
    }
    Some(hash.to_string())
}

fn main() {
    // Re-run when the checked-out commit changes.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs");
    let hash = git_hash().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=DHRUV_BUILD_GIT_HASH={hash}");
}
