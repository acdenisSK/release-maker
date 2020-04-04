# Release Maker

## What is this?

I am the lead developer of [Serenity], a Rust Discord API wrapper. When publishing new versions we cater our changelogs to a specific style.
I had been writing in this style by hand, but I quickly got sick of it. Hence lead to the creation of this tool that simplifies the process.

## Okay, seems interesting. May I use it?

Sure, do whatever you want with it. To install, invoke `cargo install --git https://github.com/acdenisSK/release-maker` in your terminal.

To generate the output, provide a path to a input file containing the changes that have occured for a release. You can also use standard input if you don't specifiy a path.

Use the `--example` and `--explain` flags for understanding the input format. For further help, use the `--help` flag.

[Serenity]: https://github.com/serenity-rs/serenity
