#![deny(rust_2018_idioms)]

use rmaker::generate_msg;

use structopt::StructOpt;

use std::fs::File;
use std::path::PathBuf;

static EXPLANATION: &str = include_str!("../explanation.txt");
static EXAMPLE: &str = include_str!("../example.json");

#[derive(StructOpt)]
#[structopt(
    name = "release-maker",
    about = "A utility tool to quickly create changelogs for Github releases"
)]
struct App {
    /// Path to input file. Standard input will be used if the path is absent.
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
