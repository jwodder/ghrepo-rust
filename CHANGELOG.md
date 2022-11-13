v0.3.0 (in development)
-----------------------
- Adjust `Display` format of `LocalRepoError::CommandFailed` to use the std
  `ExitStatus` Display
- Do not suppress stderr from executed Git commands (except for the command run
  by `LocalRepo::is_git_repo()`)
- CLI: Do not emit a redundant error message when `git remote get-url` fails
- Drop serde, serde-json, fancy-regex, and lazy-static dependencies
- Remove the `GH_OWNER_RGX` and `GH_NAME_RGX` constants

v0.2.1 (2022-10-19)
-------------------
- Restore command usage in `--help` output

v0.2.0 (2022-10-18)
-------------------
- Update clap dependency to 4.0

v0.1.0 (2022-07-08)
-------------------
Initial release
