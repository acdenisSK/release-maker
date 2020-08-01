use anyhow::Result;

pub mod cache;
pub mod git;

use cache::Cache;
use git::{Git, Repository};

/// Attempts to open a local repository in the filesystem, then in the cache.
/// If both attempts fail, a remote repository will be cloned into the cache using HTTP(s).
/// If the remote repository is already in the cache, it will be read from there.
pub fn open_repository<G>(git: G, cache: &Cache, repo_path: &str) -> Result<impl Repository>
where
    G: Git,
{
    if repo_path.starts_with("http://") || repo_path.starts_with("https://") {
        match git.open(cache.repository_path_url(repo_path)?) {
            Ok(repo) => Ok(repo),
            Err(_) => git.clone(repo_path, cache.repository_path_url(repo_path)?),
        }
    } else {
        git.open(repo_path)
            .or_else(|_| git.open(cache.repository_path(repo_path)))
    }
}
