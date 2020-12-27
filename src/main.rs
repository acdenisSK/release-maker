#![deny(rust_2018_idioms)]

mod git;
mod release;

use git::{Commit, Repository};
use release::{generate_msg, Change, Release};

use anyhow::Result;
use clap::Clap;
use serde_json::to_string_pretty;

use std::fs::File;
use std::path::PathBuf;

static EXPLANATION: &str = include_str!("../texts/explanation.txt");
static EXAMPLE: &str = include_str!("../texts/example.json");
static GOTCHAS: &str = include_str!("../texts/gotchas.txt");

/// A utility tool to quickly create changelogs for Github releases.
#[derive(Clap)]
#[clap(name = "release-maker", version = "0.2.0")]
enum App {
    Retrieve(Retrieve),
    Generate(Generate),
}

/// Retrieve a list of Git commits from a repository's branch into json that
/// can be plugged into the `generate` subcommand.
#[derive(Clap)]
#[clap(version = "0.2.0")]
struct Retrieve {
    /// Path to directory of a Git repository.
    #[clap(parse(from_os_str), default_value = ".")]
    path: PathBuf,
    /// The branch to retrieve the list of commits from.
    ///
    /// Defaults to `master` if left undefined.
    #[clap(short, long, default_value = "master")]
    branch: String,
    /// A commit hash to define the start boundary of the list.
    #[clap(short, long)]
    start: Option<String>,
    /// A commit hash to define the (inclusive) end boundary of the list.
    ///
    /// If left undefined, this will retrieve ALL commits from the start of the list.
    #[clap(short, long)]
    end: Option<String>,
}

/// Generate markdown-formatted output from json input.
#[derive(Clap)]
#[clap(version = "0.2.0")]
struct Generate {
    /// Path to input file.
    ///
    /// If the path is absent, standard input will be used instead.
    #[clap(parse(from_os_str))]
    path: Option<PathBuf>,
    /// Print example input.
    #[clap(long)]
    example: bool,
    /// Print an explanation of the input's layout and the generated output.
    #[clap(long)]
    explain: bool,
    /// Print gotchas of this command's output.
    #[clap(long)]
    gotchas: bool,
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

fn retrieve(retr: Retrieve) -> Result<()> {
    let repo = Repository::open(&retr.path)?;
    let mut commits = repo.commits(&retr.branch)?;

    if let Some(start) = retr.start {
        commits = commits.start(&start);
    }

    if let Some(end) = retr.end {
        commits = commits.end(&end);
    }

    let release = generate_release(repo.url()?, commits);

    println!("{}", to_string_pretty(&release)?);

    Ok(())
}

fn generate(gen: Generate) -> Result<()> {
    if gen.example {
        print!("{}", EXAMPLE);
    }

    if gen.explain {
        if gen.example {
            println!();
        }

        print!("{}", EXPLANATION);
    }

    if gen.gotchas {
        if gen.example || gen.explain {
            println!();
        }

        print!("{}", GOTCHAS);
    }

    if gen.example || gen.explain || gen.gotchas {
        return Ok(());
    }

    let reader: Box<dyn std::io::Read> = match gen.path {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(std::io::stdin()),
    };

    let mut reader = std::io::BufReader::new(reader);
    let release = serde_json::from_reader(&mut reader)?;

    let mut res = String::new();
    generate_msg(&mut res, &release)?;
    println!("{}", res);

    Ok(())
}

fn main() -> Result<()> {
    let app = App::parse();

    match app {
        App::Generate(gen) => generate(gen),
        App::Retrieve(retr) => retrieve(retr),
    }
}
