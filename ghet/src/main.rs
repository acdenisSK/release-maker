use anyhow::Result;
use ghet::{Commit, Repository};
use rmaker::{Change, Release};
use serde_json::to_string_pretty;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Get a list of commits from a Git repository.
struct App {
    /// The path, which may be a directory path or URL, to a Git repository
    #[structopt(default_value = ".")]
    path: String,
    /// The branch to construct the list of commits from.
    /// Defaults to `master` if left undefined.
    #[structopt(short, long, default_value = "master")]
    branch: String,
    /// A commit hash to define the start boundary of the list.
    #[structopt(short, long)]
    start: Option<String>,
    /// A commit hash to define the (inclusive) end boundary of the list.
    /// If left undefined, this will retrieve ALL commits from the start of the list.
    #[structopt(short, long)]
    end: Option<String>,
}

fn find_position(commits: &[Commit], hash: Option<String>) -> Option<usize> {
    let hash = hash?;
    commits.iter().position(|c| c.hash == hash)
}

fn generate_release(repo_url: String, commits: impl Iterator<Item = Commit>) -> Release {
    Release {
        repo_url,
        added: commits
            .map(|commit| Change::new("any", commit.message, commit.author.name, commit.hash))
            .collect(),
        ..Default::default()
    }
}

fn main() -> Result<()> {
    let app = App::from_args();

    let repo = Repository::open(&app.path)?;
    let mut commits = repo.commits(&app.branch)?;

    let start = find_position(&commits, app.start).unwrap_or(0);
    // As we are using an inclusive range, draining the list of commits by its length will result
    // in a panic. To avoid this, we subtract the length only if it is not zero.
    let end = find_position(&commits, app.end).unwrap_or(commits.len().checked_sub(1).unwrap_or(0));

    let release = generate_release(repo.url()?, commits.drain(start..=end));

    println!("{}", to_string_pretty(&release)?);

    Ok(())
}
