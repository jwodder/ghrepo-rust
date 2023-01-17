In Development
--------------
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
