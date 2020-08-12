pub mod cache;
pub mod git;

use cache::Cache;
use git::Repository;

use anyhow::Result;

/// Attempts to open a local repository in the filesystem, then in the cache.
/// If both attempts fail, a remote repository will be cloned into the cache using HTTP(s).
/// If the remote repository is already in the cache, it will be read from there.
pub fn open_repository(cache: &Cache, repo: &str) -> Result<Repository>
{
    if repo.starts_with("http://") || repo.starts_with("https://") {
        match Repository::open(cache.repository_path_url(repo)?) {
            Ok(repo) => Ok(repo),
            Err(_) => Repository::clone(repo, cache.repository_path_url(repo)?),
        }
    } else {
        Repository::open(repo)
            .or_else(|_| Repository::open(cache.repository_path(repo)))
    }
}
