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

/// Defines an iterator of [`Commit`]s.
///
/// The range of commits may be configuring using [`start`] and/or [`end`].
///
/// [`Commit`]: struct.Commit.html
/// [`start`]: #method.start
/// [`end`]: #method.end
pub struct Commits<'a> {
    repo: &'a git2::Repository,
    inner: git2::Revwalk<'a>,
    end: git2::Oid,
}

impl Commits<'_> {
    /// Defines the starting boundary for the commit list with a hash.
    ///
    /// # Panics
    ///
    /// Panics if the string is empty, is longer than 40 hex
    /// characters, or contains any non-hex characters.
    pub fn start(mut self, hash: &str) -> Self {
        self.inner.reset().unwrap();
        self.inner.push(git2::Oid::from_str(hash).unwrap()).unwrap();
        self
    }

    /// Defines the ending boundary (inclusive) for the commit list with a hash.
    ///
    /// # Panics
    ///
    /// Panics if the string is empty, is longer than 40 hex
    /// characters, or contains any non-hex characters.
    pub fn end(mut self, hash: &str) -> Self {
        self.end = git2::Oid::from_str(hash).unwrap();
        self
    }
}

impl Iterator for Commits<'_> {
    type Item = Commit;

    fn next(&mut self) -> Option<Self::Item> {
        let oid = match self.inner.next() {
            Some(Ok(oid)) => oid,
            _ => return None,
        };

        let commit = match self.repo.find_commit(oid) {
            Ok(commit) => commit,
            Err(_) => return None,
        };

        let author = commit.author();
        let committer = commit.committer();

        let commit = Commit {
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
        };

        if oid == self.end {
            // We have reached the ending boundary. Reset the Revwalk's configuration,
            // so that it no longers provides further commits.
            self.inner.reset().unwrap();
        }

        Some(commit)
    }
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

    /// Returns an iterator of [`Commit`]s from a branch.
    ///
    /// [`Commit`]: struct.Commit.html
    pub fn commits(&self, branch: &str) -> Result<Commits<'_>> {
        let reference = self
            .inner
            .find_reference(&format!("refs/remotes/origin/{}", branch))?;

        let mut revwalk = self.inner.revwalk()?;
        revwalk.push(reference.target().unwrap())?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;

        Ok(Commits {
            repo: &self.inner,
            inner: revwalk,
            end: git2::Oid::from_str("0")?,
        })
    }
}
