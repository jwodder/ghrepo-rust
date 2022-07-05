[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![CI Status](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/ghrepo-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/ghrepo-rust)
[![MIT License](https://img.shields.io/github/license/jwodder/ghrepo.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/ghrepo-rust) <!-- | [crates.io](https://crates.io/crates/ghrepo) --> <!-- | [Documentation](docs.rs/ghrepo) --> | [Issues](https://github.com/jwodder/ghrepo-rust/issues) <!-- | [Changelog](https://github.com/jwodder/ghrepo-rust/blob/master/CHANGELOG.md) -->

`ghrepo` extracts a GitHub repository's owner & name from various GitHub URL
formats (or just from a string of the form `OWNER/REPONAME` or `REPONAME`), and
the resulting object provides properties for going in reverse to determine the
possible URLs.  Also included is a function for determining the GitHub owner &
name for a local Git repository, plus a couple of other useful Git repository
inspection functions.

Example
=======

```rust
use std::error::Error;
use std::str::FromStr;
use ghrepo::GHRepo;

fn main() -> Result<(), Box<dyn Error>> {
    let repo = GHRepo::new("octocat", "repository")?;
    assert_eq!(repo.owner(), "octocat");
    assert_eq!(repo.name(), "repository");
    assert_eq!(repo.to_string(), "octocat/repository");
    assert_eq!(repo.html_url(), "https://github.com/octocat/repository");

    let repo2 = GHRepo::from_str("octocat/repository")?;
    assert_eq!(repo, repo2);

    let repo3 = GHRepo::from_str("https://github.com/octocat/repository")?;
    assert_eq!(repo, repo3);
    Ok(())
}
```

Command
=======

`ghrepo` also provides a command of the same name for getting the GitHub
repository associated with a local Git repository:

```text
ghrepo [<options>] [<dirpath>]
```

By default, the `ghrepo` command just outputs the repository "fullname" (a
string of the form `{owner}/{name}`).  If the `-J` or `--json` option is
supplied, a JSON object is instead output, containing fields for the repository
owner, name, fullname, and individual URLs, like so:

```json
{
    "owner": "jwodder",
    "name": "ghrepo-rust",
    "fullname": "jwodder/ghrepo-rust",
    "api_url": "https://api.github.com/repos/jwodder/ghrepo-rust",
    "clone_url": "https://github.com/jwodder/ghrepo-rust.git",
    "git_url": "git://github.com/jwodder/ghrepo-rust.git",
    "html_url": "https://github.com/jwodder/ghrepo-rust",
    "ssh_url": "git@github.com:jwodder/ghrepo-rust.git"
}
```

Options
-------

- `-J`, `--json` — Output JSON
- `-r REMOTE`, `--remote REMOTE` — Parse the GitHub URL from the given remote
  [default: `origin`]
