#![deny(rust_2018_idioms)]

use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use structopt::StructOpt;

use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashSet;

static EXPLANATION: &str = include_str!("explanation.txt");
static EXAMPLE: &str = include_str!("../example.json");

#[derive(Debug, Clone)]
struct OneOrMore(Vec<String>);

impl<'de> Deserialize<'de> for OneOrMore {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = Vec<String>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("one string or more")
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                Ok(vec![v.to_string()])
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let cap = seq.size_hint().unwrap_or(0);

                if cap == 0 {
                    return Err(A::Error::custom("expected at least one string"));
                }

                let mut v = Vec::with_capacity(cap);

                while let Some(elem) = seq.next_element()? {
                    v.push(elem);
                }

                Ok(v)
            }
        }

        deserializer.deserialize_any(V).map(OneOrMore)
    }
}

#[derive(Debug, Default, Clone)]
struct Author(String);

impl Author {
    fn new<I>(name: I) -> Self
        where I: Into<String>
    {
        Self(name.into())
    }

    fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[@{}]", self.name())
    }
}

#[derive(Debug, Default, Clone)]
struct Commit(String);

impl Commit {
    fn new<I>(hash: I) -> Self
        where I: Into<String>
    {
        let hash = hash.into();
        assert!(hash.len() >= 7, "commit hashes ought to at least or longer than 7 characters");
        Self(hash)
    }

    fn hash(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[c:{}]", &self.hash()[..7])
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Change(String, String, OneOrMore, OneOrMore);

#[derive(Deserialize, Debug, Clone)]
struct Release {
    #[serde(default)]
    header: String,
    repo_url: String,
    #[serde(default)]
    added: Vec<Change>,
    #[serde(default)]
    changed: Vec<Change>,
    #[serde(default)]
    fixed: Vec<Change>,
    #[serde(default)]
    removed: Vec<Change>,
}

impl Release {
    fn iter(&self) -> impl Iterator<Item = &Change> + '_ {
        self.added.iter()
            .chain(self.changed.iter())
            .chain(self.fixed.iter())
            .chain(self.removed.iter())
    }

    fn get_authors(&self) -> Vec<String> {
        self.iter()
            .flat_map(|Change(_, _, OneOrMore(authors), _)| {
                authors.iter().cloned()
            })
            // Avoid duplicate instances of authors.
            .collect::<HashSet<String>>()
            .into_iter()
            .collect()
    }

    fn get_commits(&self) -> Vec<String> {
        self.iter()
            .flat_map(|Change(_, _, _, OneOrMore(commits))| {
                commits.iter().cloned()
            })
            .collect()
    }
}

fn write_separated<T, It>(f: &mut impl fmt::Write, it: It, sep: &str) -> fmt::Result
where
    It: IntoIterator<Item = T>,
    T: fmt::Display,
{
    let it = it.into_iter();
    let mut first = true;

    for elem in it {
        if !first {
            f.write_str(sep)?;
        }

        write!(f, "{}", elem)?;

        first = false;
    }

    Ok(())
}

fn write_list(
    f: &mut impl fmt::Write,
    header: &str,
    changes: &[Change],
) -> fmt::Result {
    if changes.is_empty() {
        return Ok(());
    }

    writeln!(f, "{}\n", header)?;

    for change in changes {
        let Change(category, name, OneOrMore(authors), OneOrMore(commits)) = change;

        assert!(!category.is_empty(), "categores cannot be empty");

        write!(f, "- [{}] {} (", category, name)?;
        let authors = authors.iter().map(Author::new);
        write_separated(f, authors, " ")?;
        write!(f, ") ")?;

        let commits = commits.iter().map(Commit::new);
        write_separated(f, commits, " ")?;

        writeln!(f)?;
    }

    writeln!(f)?;

    Ok(())
}

fn generate_msg(f: &mut impl fmt::Write, rel: &Release) -> fmt::Result {
    if !rel.header.is_empty() {
        writeln!(f, "{}\n", rel.header)?;
    }

    writeln!(f, "Thanks to the following for their contributions:\n")?;

    let mut authors = rel.get_authors();
    // Sort alphabetically.
    authors.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let commits = rel.get_commits();

    for name in &authors {
        writeln!(f, "- {}", Author::new(name))?;
    }

    writeln!(f)?;

    write_list(f, "### Added", &rel.added)?;
    write_list(f, "### Changed", &rel.changed)?;
    write_list(f, "### Fixed", &rel.fixed)?;
    write_list(f, "### Removed", &rel.removed)?;

    for name in authors {
        let author = Author::new(name);
        writeln!(f, "{}: https://github.com/{}", author, author.name())?;
    }

    writeln!(f)?;

    for hash in commits {
        let commit = Commit::new(hash);
        writeln!(f, "{}: {}/commit/{}", commit, rel.repo_url, commit.hash())?;
    }

    Ok(())
}

#[derive(StructOpt)]
#[structopt(
    name = "release-maker",
    about = "A utility tool for easy changelog creation"
)]
struct App {
    /// Path to input file. Will use stdin if absent.
    #[structopt(parse(from_os_str))]
    file: Option<PathBuf>,
    /// Print example input.
    #[structopt(long)]
    example: bool,
    /// Print an explanation of the input's layout and the generated output.
    #[structopt(long)]
    explain: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::from_args();

    if app.example {
        print!("{}", EXAMPLE);
    }

    if app.explain {
        if app.example {
            println!();
        }

        print!("{}", EXPLANATION);
    }

    if app.example || app.explain {
        return Ok(());
    }

    let reader: Box<dyn std::io::Read> = match app.file {
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
