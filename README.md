[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![CI Status](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/ghrepo-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/ghrepo-rust)
[![MIT License](https://img.shields.io/github/license/jwodder/ghrepo-rust.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/ghrepo-rust) | [crates.io](https://crates.io/crates/ghrepo) | [Documentation](https://docs.rs/ghrepo) | [Issues](https://github.com/jwodder/ghrepo-rust/issues) | [Changelog](https://github.com/jwodder/ghrepo-rust/blob/master/CHANGELOG.md)

`ghrepo` extracts a GitHub repository's owner & name from various GitHub URL
formats (or just from a string of the form `OWNER/REPONAME` or `REPONAME`), and
the resulting object provides properties for going in reverse to determine the
possible URLs.  Also included is a struct for performing a couple useful
inspections on local Git repositories, including determining the corresponding
GitHub owner & repository name.

When the `serde` feature is enabled, the `GHRepo` type will additionally be
serializable & deserializable with `serde`.

Installation
============

`ghrepo` requires version 1.65 of Rust or higher.  To use the `ghrepo` library
in your Cargo project, add the following to your `Cargo.toml`:

```toml
[dependencies]
# By default, ghrepo depends on some packages only needed for its command-line
# interface; disable the default features so that these dependencies aren't
# pulled in when using as a library.
ghrepo = { version = "0.5.0", default-features = false }
```

To use `ghrepo` with its `serde` feature, add the following instead:

```toml
[dependencies]
ghrepo = { version = "0.5.0", default-features = false, features = ["serde"] }
```

To install the `ghrepo` command on your system, use `cargo install`:

    cargo install ghrepo


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
