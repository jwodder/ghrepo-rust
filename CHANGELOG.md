v0.6.0 (in development)
-----------------------
- Remove library installation instructions from README
- List all features in the library documentation
- Increased MSRV to 1.66

v0.5.0 (2023-04-27)
-------------------
- Increased MSRV to 1.65
- Convert error Displays to lowercase per Rust conventions
- The packages needed soley for the CLI are now behind a default `cli` feature

v0.4.0 (2023-02-15)
-------------------
- Added an optional `serde` feature that gives `GHRepo` `Serialize` and
  `Deserialize` implementations

v0.3.1 (2023-01-21)
-------------------
- Update lexopt dependency to v0.3.0

v0.3.0 (2022-11-15)
-------------------
- Adjust `Display` format of `LocalRepoError::CommandFailed` to use the std
  `ExitStatus` Display
- Do not suppress stderr from executed Git commands (except for the command run
  by `LocalRepo::is_git_repo()`)
- CLI: Do not emit a redundant error message when `git remote get-url` fails
- Drop serde, serde-json, fancy-regex, and lazy-static dependencies
- Remove the `GH_OWNER_RGX` and `GH_NAME_RGX` constants
- Properly follow RFC 3986 when parsing username & password fields in
  `www.github.com` URLs
- Correct the accepted format for URLs that start with `ssh://` (They need to
  separate the hostname from the path with a slash rather than a colon)
- Schemes & hostnames in URLs are now parsed case-insensitively
- Switch from clap to lexopt
- `LocalRepo::for_cwd()` now returns a new dedicated variant of
  `LocalRepoError` if the call to `std::env::current_dir()` fails

v0.2.1 (2022-10-19)
-------------------
- Restore command usage in `--help` output

v0.2.0 (2022-10-18)
-------------------
- Update clap dependency to 4.0

v0.1.0 (2022-07-08)
-------------------
Initial release
