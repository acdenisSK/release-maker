use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;

#[derive(Debug, Clone)]
struct OneOrMore(Vec<usize>);

impl<'de> Deserialize<'de> for OneOrMore {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = Vec<usize>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("one item or more")
            }

            fn visit_u64<E: DeError>(self, v: u64) -> Result<Self::Value, E> {
                Ok(vec![v as usize])
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut v = Vec::with_capacity(seq.size_hint().unwrap_or(0));

                while let Some(elem) = seq.next_element()? {
                    v.push(elem);
                }

                Ok(v)
            }
        }

        deserializer.deserialize_any(V).map(OneOrMore)
    }
}

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
#[serde(untagged)]
enum Change {
    Normal(String, usize, usize),
    Custom(String, String, OneOrMore, OneOrMore),
}

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
        let cat = |category: &str| {
            if category.is_empty() {
                "".to_string()
            } else {
                format!("[{}]", category)
            }
        };

        if !l.is_empty() {
            println!("{}\n", s);

            for change in l {
                match change {
                    Change::Normal(category, author, commit) => {
                        let category = cat(category);
                        let author = &rel.authors[author - 1];
                        let commit = &rel.commits[commit - 1];

                        println!(
                            "- {} {} ({}) {}",
                            category,
                            commit.0,
                            author.reference(),
                            commit.reference()
                        );
                    },
                    Change::Custom(category, name, OneOrMore(authors), OneOrMore(commits)) => {
                        let category = cat(category);

                        print!("- {} {} (", category, name);

                        let mut first = true;

                        for author in authors {
                            if !first {
                                print!(" ");
                            }

                            let author = &rel.authors[author - 1];
                            print!("{}", author.reference());

                            first = false;

                        }

                        print!(") ");

                        first = true;
                        for commit in commits {
                            if !first {
                                print!(" ");
                            }

                            let commit = &rel.commits[commit - 1];
                            print!("{}", commit.reference());
                            first = false;
                        }

                        println!();
                    }
                }
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
