[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![CI Status](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ghrepo-rust/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/ghrepo-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/ghrepo-rust)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.74-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ghrepo-rust.svg)](https://opensource.org/licenses/MIT)

This is a Rust [workspace][] containing various packages for working with
GitHub repository URLs.

The packages are:

- [`ghrepo`][] — Rust library for parsing & constructing GitHub repository URLs
  & specifiers

- [`ghrepo-cli`][] — CLI command for showing the GitHub repository for a local
  Git repository

- [`repomaker`][] — Internal package for use in testing the other packages

[workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[`ghrepo`]: https://github.com/jwodder/ghrepo-rust/tree/master/crates/ghrepo
[`ghrepo-cli`]: https://github.com/jwodder/ghrepo-rust/tree/master/crates/ghrepo-cli
[`repomaker`]: https://github.com/jwodder/ghrepo-rust/tree/master/crates/repomaker
