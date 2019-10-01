use serde::Deserialize;
use std::fs::File;
use std::error::Error as StdError;

#[derive(Deserialize, Debug, Clone)]
struct Author(String, String);

impl Author {
    #[inline]
    pub fn reference(&self) -> String {
        let Self(name, ..) = self;

        format!("[@{}]", name)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Commit(String, String);

impl Commit {
    #[inline]
    pub fn reference(&self) -> String {
        let Self(.., url) = self;

        let mut index = url.rfind('/').unwrap();
        index += 1;

        format!("[c:{}]", &url[index..index + 7])
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Change(String, usize, usize);

#[derive(Deserialize, Debug, Clone)]
struct Release {
    header: Option<String>,
    authors: Vec<Author>,
    commits: Vec<Commit>,
    #[serde(default)]
    added: Vec<Change>,
    #[serde(default)]
    changed: Vec<Change>,
    #[serde(default)]
    fixed: Vec<Change>,
    #[serde(default)]
    removed: Vec<Change>,
}

fn fmt(rel: &Release) {
    if let Some(header) = &rel.header {
        println!("{}\n", header);
    }

    println!("Thanks to the following for their contributions:\n");

    for author in &rel.authors {
        println!("- {}", author.reference());
    }

    println!();

    let print_list = |s, l: &[Change]| {
        if !l.is_empty() {
            println!("{}\n", s);

            for Change(category, commit, author) in l {
                let author = &rel.authors[author - 1];
                let commit = &rel.commits[commit - 1];

                let category = if category.is_empty() {
                    "".to_string()
                } else {
                    format!("[{}]", category)
                };

                println!(
                    "- {} {} ({}) {}",
                    category,
                    commit.0,
                    author.reference(),
                    commit.reference()
                );
            }

            println!();
        }
    };

    print_list("### Added", &rel.added);
    print_list("### Changed", &rel.changed);
    print_list("### Fixed", &rel.fixed);
    print_list("### Removed", &rel.removed);

    for author in &rel.authors {
        println!("{}: {}", author.reference(), author.1);
    }

    println!();

    for commit in &rel.commits {
        println!("{}: {}", commit.reference(), commit.1);
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    let name = std::env::args().nth(1).unwrap();
    let mut file = File::open(name)?;

    let release = serde_json::from_reader(&mut file)?;

    fmt(&release);

    Ok(())
}
