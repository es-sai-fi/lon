use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{
    init::Convertible,
    sources::{GitHubSource, GitSource, Source, Sources},
};

#[derive(Debug, Deserialize)]
pub struct LockFile(BTreeMap<String, Package>);

#[derive(Debug, Deserialize)]
pub struct Package {
    owner: Option<String>,
    repo: String,
    branch: String,
    rev: String,
}

impl LockFile {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let lock_json = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read {:?}", path.as_ref()))?;

        serde_json::from_str(&lock_json).context("Failed to deserialize Niv lock file")
    }
}

impl Convertible for LockFile {
    fn convert(&self) -> Result<Sources> {
        let mut sources = Sources::default();

        for (name, package) in &self.0 {
            log::info!("Converting {name}...");

            if let Some(owner) = &package.owner {
                let source = GitHubSource::new(
                    owner,
                    &package.repo,
                    &package.branch,
                    Some(&package.rev),
                    false,
                )?;

                sources.add(name, Source::GitHub(source));
            } else {
                let source = GitSource::new(
                    &package.repo,
                    &package.branch,
                    Some(&package.rev),
                    false,
                    false,
                )?;

                sources.add(name, Source::Git(source));
            }
        }

        Ok(sources)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl LockFile {
        fn from_str(s: &str) -> Result<Self> {
            serde_json::from_str(s).context("Failed to deserialize Niv lock file")
        }
    }

    #[test]
    fn parse_niv_lock_file() -> Result<()> {
        LockFile::from_str(include_str!("../../tests/niv.json"))?;
        Ok(())
    }
}
