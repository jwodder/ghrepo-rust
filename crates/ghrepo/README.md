[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![CI Status](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/ghrepo-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/ghrepo-rust)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.74-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ghrepo-rust.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/ghrepo-rust) | [crates.io](https://crates.io/crates/ghrepo) | [Documentation](https://docs.rs/ghrepo) | [Issues](https://github.com/jwodder/ghrepo-rust/issues) | [Changelog](https://github.com/jwodder/ghrepo-rust/blob/master/crates/ghrepo/CHANGELOG.md)

`ghrepo` extracts a GitHub repository's owner & name from various GitHub URL
formats (or just from a string of the form `OWNER/REPONAME` or `REPONAME`), and
the resulting object provides properties for going in reverse to determine the
possible URLs.  Also included is a struct for performing a couple useful
inspections on local Git repositories, including determining the corresponding
GitHub owner & repository name.

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

There is also an accompanying binary package
[`ghrepo-cli`](https://crates.io/crates/ghrepo-cli) that provides a CLI command
named `ghrepo` for showing the GitHub repository for a directory, optionally
along with derived URLs.  Feel free to install it if you're interested!
