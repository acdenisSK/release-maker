//! The *ghet* crate defines a small abstraction of the libgit2 C library
//! to simplify its usage for the `ghet` binary.

use anyhow::Result;

use std::path::Path;

/// Defines a Git user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub name: String,
    pub email: String,
}

/// Defines a Git commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub hash: String,
    pub author: User,
    pub committer: User,
    pub message: String,
}

/// A wrapper around the [`git2`] crate's [`Repository`] type.
///
/// [`git2`]: https://github.com/rust-lang/git2-rs
/// [`Repository`]: https://docs.rs/git2/*/git2/struct.Repository.html
pub struct Repository {
    inner: git2::Repository,
}

impl Repository {
    /// Open a local repository at `path`.
    pub fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            inner: git2::Repository::open(path)?,
        })
    }

    /// Returns the URL to the repository.
    pub fn url(&self) -> Result<String> {
        Ok(self.inner.find_remote("origin")?.url().unwrap().to_string())
    }

    /// Returns a vector of [`Commit`]s from a branch.
    ///
    /// [`Commit`]: struct.Commit.html
    pub fn commits(&self, branch: &str) -> Result<Vec<Commit>> {
        let reference = self.inner.find_reference(&format!("refs/remotes/origin/{}", branch))?;

        let mut revwalk = self.inner.revwalk()?;
        revwalk.push(reference.target().unwrap())?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;

        let mut commits = Vec::new();

        for oid in revwalk {
            let commit = self.inner.find_commit(oid?)?;
            let author = commit.author();
            let committer = commit.committer();

            commits.push(Commit {
                hash: commit.id().to_string(),
                author: User {
                    name: author.name().unwrap().to_string(),
                    email: author.email().unwrap().to_string(),
                },
                committer: User {
                    name: committer.name().unwrap().to_string(),
                    email: committer.email().unwrap().to_string(),
                },
                message: commit.summary().unwrap().to_string(),
            });
        }

        Ok(commits)
    }
}
