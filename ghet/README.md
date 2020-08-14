# `ghet`

`ghet` is a tool to get a list of commits from a local Git repository. It is a utility to swiftly
fetch an arbitrary amount of commits that you can plug into the `release-maker` program.

The output is `release-maker`-compatible json. You can store it in a file, make amendments, and create
a changelog out of it. You can also directly pipe it to `release-maker`'s standard input, but this is
discouraged (see below).

## Usage

```
ghet path/to/repo
```

Here we specify the path to the repository as the argument to `ghet`. Without specifying anything else,
this command will get *ALL* of the commits from the repository's default branch, assumed to be `master`.
If you leave out this argument, `ghet` will try to read a repository from the current directory.

If you need to be specific with the amount of commits, and the location where the listing
should occur, you can do this with the `--start` (`-s`) and `--end` (`-e`) flags. For exemplary purposes,
we will apply these flags on the release-maker repository, assuming it's the current directory:

```
ghet --start 586f5e3220022e33e1bb3c0ccc0380eb312d1b0c --end 2f540649cd0f5d3d6a47d3119f15f6367c289330
```

This command will fetch every commit, beginning from the `--start`ing boundary and finishing
at the `--end`ing boundary, inclusively. The boundaries operate from **top** to **bottom**.
Or in other words, from the most recent to oldest commit.

The output of the command is this:

```json
{
  "header": "",
  "repo_url": "https://github.com/acdenisSK/release-maker",
  "added": [
    [
      "any",
      "Move the explanation out of the src/ directory",
      "Alex M. M",
      "586f5e3220022e33e1bb3c0ccc0380eb312d1b0c"
    ],
    [
      "any",
      "Utilize Cargo workspaces",
      "Alex M. M",
      "ad67a354a06b20efb4ec6446bcbebc3194563c8c"
    ],
    [
      "any",
      "Add EditorConfig file",
      "Alex M. M",
      "2f540649cd0f5d3d6a47d3119f15f6367c289330"
    ]
  ],
  "changed": [],
  "fixed": [],
  "removed": []
}
```

The output of the `release-maker` program after piping the json above is this:

```markdown
Thanks to the following for their contributions:

- [@Alex M. M]

### Added

- [any] Move the explanation out of the src/ directory ([@Alex M. M]) [c:586f5e3]
- [any] Utilize Cargo workspaces ([@Alex M. M]) [c:ad67a35]
- [any] Add EditorConfig file ([@Alex M. M]) [c:2f54064]

[@Alex M. M]: https://github.com/Alex M. M

[c:586f5e3]: https://github.com/acdenisSK/release-maker/commit/586f5e3220022e33e1bb3c0ccc0380eb312d1b0c
[c:ad67a35]: https://github.com/acdenisSK/release-maker/commit/ad67a354a06b20efb4ec6446bcbebc3194563c8c
[c:2f54064]: https://github.com/acdenisSK/release-maker/commit/2f540649cd0f5d3d6a47d3119f15f6367c289330
```

As can be seen, this is already a usable changelog. However, it might not be a suitable changelog.

This is due to three facts of the output from `ghet`:
- The names of authors are retrieved from the data of the commits, not from Github.
  - Currently, `release-maker` assumes that the authors belong to Github accounts.
  - This has the side-effect of breaking the link to the author's Github profile page.
- All changes are clumped into one purpose, the `added` purpose.
  - Of the commits used above, only one fits this purpose.
- The category of all changes is assumed `any`.

It is, therefore, recommended to amend the output from `ghet` before piping it into `release-maker`.

## Further help

If you're seeking more information, consult the `--help` flag of `ghet`.
