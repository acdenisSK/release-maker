//! The *cache* module defines an abstraction to the program's user-level cache directory.

use anyhow::{Context, Result};
use directories::BaseDirs;

use std::fs::read_dir;
use std::path::{Path, PathBuf};

/// `Cache` is a structure to keep track of repositories that are stored in the cache directory of
/// the program. The cache directory is per user. The path is provided by the system, such as
/// the XDG Base Directory specification on Linux, or Known Folders on Windows.
#[derive(Debug, Clone)]
pub struct Cache {
    path: PathBuf,
    program_name: String,
}

impl Cache {
    /// Creates a new instance of a `Cache` that is associated to a program by its name.
    pub fn new<I>(program_name: I) -> Result<Self>
    where
        I: Into<String>,
    {
        let base_dirs = BaseDirs::new().context("System did not have a valid home directory")?;

        Ok(Cache {
            path: base_dirs.cache_dir().into(),
            program_name: program_name.into(),
        })
    }

    /// Returns the path to the user cache directory given by the system.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the path to the program's directory inside the user cache directory.
    pub fn program_path(&self) -> PathBuf {
        self.path.join(&self.program_name)
    }

    /// Returns the name of the program this `Cache` instance belongs to.
    pub fn program_name(&self) -> &str {
        &self.program_name
    }

    /// Returns the path to a repository called `repo_name` in the cache directory.
    pub fn repository_path<P>(&self, repo_name: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        self.program_path().join(repo_name)
    }

    /// Returns an iterator of [`PathBuf`]s to all repositories inside the user cache directory.
    ///
    /// [`PathBuf`]: std::path::PathBuf
    pub fn repositories(&self) -> Result<impl Iterator<Item = PathBuf> + '_> {
        Ok(read_dir(self.program_path())?.flat_map(|entry| entry.map(|e| e.path())))
    }

    /// Returns the path to a remote repository in the cache directory.
    /// The repository name is derived using the URL.
    ///
    /// # Errors
    ///
    /// This returns an error when:
    /// - `repo_url` is a misconforming URL and it's impossible to get the repository name.
    pub fn repository_path_url(&self, repo_url: &str) -> Result<PathBuf> {
        let repo_name = repo_url
            .rsplit('/')
            .next()
            .context("Could not find the repository name")?;

        Ok(self.repository_path(repo_name))
    }

    /// Returns a boolean indicating that a repository exists in the user cache directory.
    pub fn repository_exists<P>(&self, repo_name: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.repository_path(repo_name).exists()
    }

    /// Clears the program cache, removing all repositories inside of the user cache directory.
    pub fn clear(&self) -> Result<()> {
        std::fs::remove_dir_all(self.program_path()).map_err(From::from)
    }
}
