use rmaker::{Change, Release};

use structopt::StructOpt;

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use octocrab::Octocrab;

use anyhow::{anyhow, bail, Context, Result};

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    email: String,
    date: String,
}

#[derive(Debug, Deserialize)]
struct CommitData {
    author: User,
    committer: User,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GithubCommit {
    sha: String,
    html_url: String,
    commit: CommitData,
}

#[derive(Debug, Serialize)]
struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<String>,
}

#[derive(StructOpt)]
#[structopt(name = "ghet", about = "Get a list of commits from Github")]
struct App {
    /// The URL to a Github repository
    #[structopt(short, long)]
    url: String,
    /// Either a commit hash or branch name to define the start boundary of the list.
    /// If left undefined, this will assume the default branch of the repository.
    #[structopt(short, long)]
    start: Option<String>,
    /// A commit hash to define the end boundary of the list.
    /// If left undefined, this will retrieve ALL commits from the start of the list.
    #[structopt(short, long)]
    end: Option<String>,
}

fn extract_repository(url: &str) -> Result<&str> {
    match url.strip_prefix("https://github.com/") {
        Some(repo) => Ok(repo),
        None => Err(anyhow!("missing 'https://github.com' prefix")),
    }
}

fn retrieve_token() -> Result<String> {
    std::env::var("GITHUB_TOKEN").context("Failed to retrieve Github token from `$GITHUB_TOKEN`")
}

async fn generate_parameters(client: &Octocrab, app: &App, base_url: &str) -> Result<Parameters> {
    match &app.start {
        Some(start) => match &app.end {
            Some(end) => {
                let commit: GithubCommit = client
                    .get(format!("{}/{}", base_url, end), None::<&()>)
                    .await
                    .context("Failed to retrieve commit to specify the end boundary")?;

                Ok(Parameters {
                    sha: Some(start.clone()),
                    since: Some(commit.commit.committer.date),
                })
            }
            None => Ok(Parameters {
                sha: Some(start.clone()),
                since: None,
            }),
        },
        None if app.end.is_some() => bail!("Defined an `end` boundary, but not a `start` boundary"),
        None => Ok(Parameters {
            sha: None,
            since: None,
        }),
    }
}

fn generate_release(repo_url: String, commits: Vec<GithubCommit>) -> Release {
    Release {
        repo_url,
        added: commits
            .into_iter()
            .map(|commit| {
                let GithubCommit {
                    sha,
                    commit:
                        CommitData {
                            author, message, ..
                        },
                    ..
                } = commit;

                Change::new("any", message, author.name, sha)
            })
            .collect(),
        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::from_args();
    let repo =
        extract_repository(&app.url).context("Failed to extract repository out of the URL")?;

    let client = Octocrab::builder()
        .personal_token(retrieve_token()?)
        .build()?;

    let base_url = format!("/repos/{}/commits", repo);
    let parameters = generate_parameters(&client, &app, &base_url).await?;
    let result: Vec<GithubCommit> = client
        .get(base_url, Some(&parameters))
        .await
        .context("Failed to retrieve list of commits")?;

    let release = generate_release(app.url, result);
    println!("{}", to_string_pretty(&release)?);

    Ok(())
}
