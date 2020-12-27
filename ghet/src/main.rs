use anyhow::Result;
use ghet::{Commit, Repository};
use rmaker::{Change, Release};
use serde_json::to_string_pretty;
use clap::Clap;

/// Get a list of commits from a Git repository.
#[derive(Clap)]
struct App {
    /// The path, which may be a directory path or URL, to a Git repository
    #[clap(default_value = ".")]
    path: String,
    /// The branch to construct the list of commits from.
    /// Defaults to `master` if left undefined.
    #[clap(short, long, default_value = "master")]
    branch: String,
    /// A commit hash to define the start boundary of the list.
    #[clap(short, long)]
    start: Option<String>,
    /// A commit hash to define the (inclusive) end boundary of the list.
    /// If left undefined, this will retrieve ALL commits from the start of the list.
    #[clap(short, long)]
    end: Option<String>,
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
    let app = App::parse();

    let repo = Repository::open(&app.path)?;
    let mut commits = repo.commits(&app.branch)?;

    if let Some(start) = app.start {
        commits = commits.start(&start);
    }

    if let Some(end) = app.end {
        commits = commits.end(&end);
    }

    let release = generate_release(repo.url()?, commits);

    println!("{}", to_string_pretty(&release)?);

    Ok(())
}
