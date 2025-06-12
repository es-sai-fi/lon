use std::process::Command;

use anyhow::{Context, Result, bail};
use nix_compat::nixhash::{HashAlgo, NixHash};
use serde::Deserialize;

#[derive(Deserialize)]
struct NixPrefetchGitResponse {
    hash: NixHash,
}

/// Fetch a git source and calculate its hash.
///
/// Uses the same store path (via `--name source`) as `builtins.fetchGit` to download the
/// source only once.
pub fn prefetch_git(url: &str, revision: &str, submodules: bool) -> Result<NixHash> {
    let mut command = Command::new("nix-prefetch-git");
    if submodules {
        command.arg("--fetch-submodules");
    }
    let output = command
        .arg("--name")
        .arg("source")
        .arg(url)
        .arg(revision)
        .output()
        .context("Failed to execute nix-prefetch-git. Most likely it's not on PATH")?;

    if !output.status.success() {
        bail!(
            "Failed to prefetch git from {url}@{revision}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let response: NixPrefetchGitResponse = serde_json::from_slice(&output.stdout)
        .context("Failed to deserialize nix-prefetch-git JSON response")?;

    Ok(response.hash)
}

/// Fetch a tarball and calculate its hash.
///
/// Uses the same store path (via `--name source`) as `builtins.fetchTarball` to download the
/// source only once.
pub fn prefetch_tarball(url: &str) -> Result<NixHash> {
    let output = Command::new("nix-prefetch-url")
        .arg("--unpack")
        .arg("--name")
        .arg("source")
        .arg("--type")
        .arg("sha256")
        .arg(url)
        .output()
        .context("Failed to execute nix-prefetch-url. Most likely it's not on PATH")?;

    if !output.status.success() {
        bail!(
            "Failed to prefetch tarball from {url}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(NixHash::from_str(stdout.trim(), Some(HashAlgo::Sha256))?)
}
