# Ghet

Ghet is a tool to *get* a list of commits from **G**it**h**ub. It is a utility to swiftly fetch
an arbitrary amount of commits from a repository that you have read access to.

The output is `release-maker`-compatible json. As a result, you can directly pipe it as input
to the `release-maker` program to generate the markdown changelog. Note, however, that the output
given by `ghet` will not be immediataly suitable for creating a changelog. Its job is to just get
a list of commits in the json format that can be plugged into `release-maker`. It cannot tell the
purpose of the changes, it cannot group commits or authors into a single change, it cannot create
a header detailing the release for you.

To clarify what is meant in the previous paragraph, let's dive into an example to demonstrate.
We will be using this repository as the variable of the example.

## Example

```
GITHUB_TOKEN="..." ghet --url https://github.com/acdenisSK/release-maker
```

*`GITHUB_TOKEN` is an environment variable containing your Github personal access token. You can
generate a new token by going into the [Personal Access Tokens][pat] page.*

Here we specify the repository with the `--url` flag, or `-u` for short. Without specifying
anything else, this command will get *ALL* of the commits from the repository's default branch.
If you need to be concrete with the amount of commits, and the location where the listing
should occur, you can do this with the `--start` (`-s`) and `--end` (`-e`) flags.

```
GITHUB_TOKEN="..." ghet --url https://github.com/acdenisSK/release-maker --start 586f5e3220022e33e1bb3c0ccc0380eb312d1b0c --end 2f540649cd0f5d3d6a47d3119f15f6367c289330
```

This command will fetch every commit, beginning from the `--start`ing boundary and finishing
at the `--end`ing boundary, inclusively. `--start` may be a branch name. Due to how Github's API
operates, the boundaries work from **top** to **bottom**. That is, from the most recent commit
to the oldest commit. Specifying a branch in `--start` will begin from its latest commit.

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

As can be seen, this is a usable changelog already. But as mentioned previously, it might not be a
suitable changelog.

This is due to the three facts of the output from `ghet`:
- The names of authors are retrieved from the data of the commits, not from Github.
  - This has the side-effect of breaking the link to the author's Github profile page.
- All changes are clumped into one purpose, the `added` purpose.
  - Of the commits used above, only one fits this purpose.
- The category of all changes is assumed `any`.

Therefore, it is recommended to alter the output from `ghet` to your needs the moment it is spat out!

## Further help

If you're seeking more information, consult the `--help` flag of `ghet`.

[pat]: https://github.com/settings/tokens
