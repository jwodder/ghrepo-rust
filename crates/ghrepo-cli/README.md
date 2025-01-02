[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![CI Status](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/ghrepo-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/ghrepo-rust)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.74-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ghrepo-rust.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/ghrepo-rust) | [crates.io](https://crates.io/crates/ghrepo-cli) | [Issues](https://github.com/jwodder/ghrepo-rust/issues) | [Changelog](https://github.com/jwodder/ghrepo-rust/blob/master/crates/ghrepo-cli/CHANGELOG.md)

The `ghrepo-cli` package provides a `ghrepo` command for for getting the GitHub
repository associated with a local Git repository, optionally along with
various derived URLs for the repository like the REST v3 API URL.

Installation
============

In order to install the `ghrepo` command, you first need to have [Rust and
Cargo installed](https://www.rust-lang.org/tools/install).  You can then build
the latest release of `ghrepo-cli` and install it in `~/.cargo/bin` by running:

    cargo install ghrepo-cli

Usage
=====

```text
ghrepo [<options>] [<dirpath>]
```

`ghrepo` retrieves the URL of the `origin` remote (or another remote specified
with the `--remote` option) of the Git repository located in `<dirpath>` (or
the current directory if no argument is given) and parses it to determine what
GitHub repository it points to.  By default, the command just outputs the
GitHub repository "fullname" (a string of the form `{owner}/{name}`), but if
the `-J` or `--json` option is supplied, a JSON object is output instead,
containing fields for the repository owner, name, fullname, and individual
URLs, like so:

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
