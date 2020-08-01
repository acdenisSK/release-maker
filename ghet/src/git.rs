//! The *git* module defines an abstraction to the necessary wheels and cogs for understanding and
//! manipulating Git repositories. The wheels and cogs may be the `git` binary, or the `libgit2` C library.

use anyhow::Result;
use git2::Repository as Git2Repository;

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

mod private {
    use super::*;

    pub trait Restricted {}

    impl Restricted for Git2Repository {}

    pub enum Void {}

    impl Restricted for Void {}
}

/// Specifies an abstraction to a repository by the `git` binary, or the `libgit2` C library.
///
/// Cannot be implemented outside of the crate.
pub trait Repository: private::Restricted {
    /// Returns the URL to the repository.
    fn url(&self) -> Result<String>;
    /// Returns a vector of [`Commit`]s from a branch.
    ///
    /// [`Commit`]: struct.Commit.html
    fn commits(&self, branch: &str) -> Result<Vec<Commit>>;
}

impl Repository for Git2Repository {
    fn url(&self) -> Result<String> {
        Ok(self.find_remote("origin")?.url().unwrap().to_string())
    }

    fn commits(&self, branch: &str) -> Result<Vec<Commit>> {
        let reference = self.find_reference(&format!("refs/remotes/origin/{}", branch))?;

        let mut revwalk = self.revwalk()?;
        revwalk.push(reference.target().unwrap())?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;

        let mut commits = Vec::new();

        for oid in revwalk {
            let commit = self.find_commit(oid?)?;
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

impl Repository for private::Void {
    fn url(&self) -> Result<String> {
        unimplemented!()
    }

    fn commits(&self, _branch: &str) -> Result<Vec<Commit>> {
        unimplemented!()
    }
}

/// This trait is an an abstraction to the necessary wheels and cogs for understanding and
/// manipulating Git repositories. The wheels and cogs may be the `git` binary, or the `libgit2` C library.
pub trait Git {
    /// Repository type of the respective wheels and cog.
    type Repository: Repository;

    /// Clones a remote repository at `repo_url` into `destination`.
    fn clone<P>(&self, repo_url: &str, destination: P) -> Result<Self::Repository>
    where
        P: AsRef<Path>;

    /// Open a local repository at `repo_path`.
    fn open<P>(&self, repo_path: P) -> Result<Self::Repository>
    where
        P: AsRef<Path>;
}

/// Provides Git capabilities using the `libgit2` C library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Git2;

impl Git for Git2 {
    type Repository = Git2Repository;

    fn clone<P>(&self, repo_url: &str, destination: P) -> Result<Self::Repository>
    where
        P: AsRef<Path>,
    {
        Git2Repository::clone(repo_url, destination).map_err(From::from)
    }

    fn open<P>(&self, repo_path: P) -> Result<Self::Repository>
    where
        P: AsRef<Path>,
    {
        Git2Repository::open(repo_path).map_err(From::from)
    }
}

/// Provides Git capabilities using the `git` binary.
///
/// UNIMPLEMENTED.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitBin;

impl Git for GitBin {
    type Repository = private::Void;

    fn clone<P>(&self, _repo_url: &str, _destination: P) -> Result<Self::Repository>
    where
        P: AsRef<Path>,
    {
        todo!()
    }

    fn open<P>(&self, _repo_path: P) -> Result<Self::Repository>
    where
        P: AsRef<Path>,
    {
        todo!()
    }
}
