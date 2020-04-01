#![deny(rust_2018_idioms)]

use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use structopt::StructOpt;

use std::fmt;
use std::fs::File;
use std::path::PathBuf;

static EXPLANATION: &str = include_str!("explanation.txt");
static EXAMPLE: &str = include_str!("../example.json");

#[derive(Debug, Clone)]
struct OneOrMore(Vec<usize>);

impl<'de> Deserialize<'de> for OneOrMore {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = Vec<usize>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("one index or more")
            }

            fn visit_u64<E: DeError>(self, v: u64) -> Result<Self::Value, E> {
                Ok(vec![v as usize])
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let cap = seq.size_hint().unwrap_or(0);

                if cap == 0 {
                    return Err(A::Error::custom("expected at least one index"));
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
struct Author {
    name: String,
    url: String,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[@{}]", self.name)
    }
}

impl<'de> Deserialize<'de> for Author {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Vis;

        impl<'de> Visitor<'de> for Vis {
            type Value = Author;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an author's name or their name and their github page")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut author = Author::default();

                author.name = match seq.next_element()? {
                    Some(name) => name,
                    None => return Err(A::Error::custom("missing author name")),
                };

                author.url = match seq.next_element()? {
                    Some(url) => url,
                    None => format!("https://github.com/{}", author.name),
                };

                Ok(author)
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Author {
                    name: v.to_string(),
                    url: format!("https://github.com/{}", v),
                })
            }
        }

        deserializer.deserialize_any(Vis)
    }
}

#[derive(Debug, Default, Clone)]
struct Commit {
    name: String,
    hash: String,
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[c:{}]", &self.hash[..7])
    }
}

impl<'de> Deserialize<'de> for Commit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut strings = <Vec<String> as Deserialize<'de>>::deserialize(deserializer)?;

        if strings.len() != 2 {
            return Err(D::Error::custom(
                "expected two strings in an array for the name and hash of commit",
            ));
        }

        let name = strings.remove(0);
        let hash = strings.remove(0);

        if hash.len() < 7 {
            return Err(D::Error::custom(
                "hash identifier must be at least or longer than 7 characters",
            ));
        }

        Ok(Commit { name, hash })
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Change {
    Normal(String, usize, usize),
    Custom(String, String, OneOrMore, OneOrMore),
}

#[derive(Deserialize, Debug, Clone)]
struct Release {
    #[serde(default)]
    header: String,
    repo_url: String,
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
    rel: &Release,
) -> fmt::Result {
    if changes.is_empty() {
        return Ok(());
    }

    writeln!(f, "{}\n", header)?;

    let cat = |category: &str| format!("[{}]", category);

    for change in changes {
        match change {
            Change::Normal(category, author, commit) => {
                assert!(!category.is_empty());

                let category = cat(category);
                let author = &rel.authors[author - 1];
                let commit = &rel.commits[commit - 1];

                writeln!(f, "- {} {} ({}) {}", category, commit.name, author, commit)?;
            }
            Change::Custom(category, name, OneOrMore(authors), OneOrMore(commits)) => {
                assert!(!category.is_empty());

                let category = cat(category);

                print!("- {} {} (", category, name);

                let authors = authors.iter().map(|i| &rel.authors[i - 1]);
                write_separated(f, authors, " ")?;
                write!(f, ") ")?;

                let commits = commits.iter().map(|i| &rel.commits[i - 1]);
                write_separated(f, commits, " ")?;

                writeln!(f)?;
            }
        }

        writeln!(f)?;
    }

    Ok(())
}

fn generate_msg(f: &mut impl fmt::Write, rel: &Release) -> fmt::Result {
    if !rel.header.is_empty() {
        writeln!(f, "{}\n", rel.header)?;
    }

    writeln!(f, "Thanks to the following for their contributions:\n")?;

    for author in &rel.authors {
        writeln!(f, "- {}", author)?;
    }

    writeln!(f)?;

    write_list(f, "### Added", &rel.added, rel)?;
    write_list(f, "### Changed", &rel.changed, rel)?;
    write_list(f, "### Fixed", &rel.fixed, rel)?;
    write_list(f, "### Removed", &rel.removed, rel)?;

    for author in &rel.authors {
        writeln!(f, "{}: {}", author, author.url)?;
    }

    writeln!(f)?;

    for commit in &rel.commits {
        writeln!(f, "{}: {}/commit/{}", commit, rel.repo_url, commit.hash)?;
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
