#![warn(warnings)]

//! Parse GitHub repository URLs & specifiers
//!
//! `ghrepo` extracts a GitHub repository's owner & name from various GitHub
//! URL formats (or just from a string of the form `OWNER/REPONAME` or
//! `REPONAME`), and the resulting object provides properties for going in
//! reverse to determine the possible URLs.  Also included is a struct for
//! performing a couple useful inspections on local Git repositories, including
//! determining the corresponding GitHub owner & repository name.
//!
//! ```
//! # use std::error::Error;
//! # use std::str::FromStr;
//! # use ghrepo::GHRepo;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let repo = GHRepo::new("octocat", "repository")?;
//! assert_eq!(repo.owner(), "octocat");
//! assert_eq!(repo.name(), "repository");
//! assert_eq!(repo.to_string(), "octocat/repository");
//! assert_eq!(repo.html_url(), "https://github.com/octocat/repository");
//!
//! let repo2 = GHRepo::from_str("octocat/repository")?;
//! assert_eq!(repo, repo2);
//!
//! let repo3 = GHRepo::from_str("https://github.com/octocat/repository")?;
//! assert_eq!(repo, repo3);
//! #     Ok(())
//! # }
//! ```

#[macro_use]
extern crate lazy_static;

use clap::Parser;
use regex::Regex;
use serde_json::json;
use std::env;
use std::error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::str::{self, FromStr};

#[cfg(test)]
use rstest_reuse;

/// Regular expression for a valid GitHub username or organization name.  As of
/// 2017-07-23, trying to sign up to GitHub with an invalid username or create
/// an organization with an invalid name gives the message "Username may only
/// contain alphanumeric characters or single hyphens, and cannot begin or end
/// with a hyphen".  Additionally, trying to create a user named "none" (case
/// insensitive) gives the message "Username name 'none' is a reserved word."
///
/// Unfortunately, there are a number of users who made accounts before the
/// current name restrictions were put in place, and so this regex also needs
/// to accept names that contain underscores, contain multiple consecutive
/// hyphens, begin with a hyphen, and/or end with a hyphen.
///
/// Note that this regex does not check that the owner name is not "none", as
/// the `regex` crate does not support lookaround; for full validation, use
/// [`GHRepo::is_valid_owner()`].
const GH_OWNER_RGX: &str = r"[-_A-Za-z0-9]+";

/// Regular expression for a valid GitHub repository name.  Testing as of
/// 2017-05-21 indicates that repository names can be composed of alphanumeric
/// ASCII characters, hyphens, periods, and/or underscores, with the names
/// ``.`` and ``..`` being reserved and names ending with ``.git`` forbidden.
///
/// Note that this regex does not check that the name does not end with ".git",
/// as the `regex` crate does not support lookaround; for full validation, use
/// [`GHRepo::is_valid_name()`].
const GH_REPO_RGX: &str = r"(?:\.?[-A-Za-z0-9_][-A-Za-z0-9_.]*?|\.\.[-A-Za-z0-9_.]+?)";

lazy_static! {
    /// Convenience regular expression for `<owner>/<name>`, including named
    /// capturing groups
    static ref OWNER_NAME: String = format!(r"(?P<owner>{})/(?P<name>{})", GH_OWNER_RGX, GH_REPO_RGX);
}

/// Error returned when trying to construct a [`GHRepo`] with invalid arguments
/// or parse an invalid repository spec
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// Returned by [`GHRepo::from_str`], [`GHRepo::from_url`], or
    /// [`GHRepo::from_str_with_owner`] if given a string that is not a valid
    /// GitHub repository URL or specifier; the field is the string in
    /// question.
    InvalidSpec(String),

    /// Returned by [`GHRepo::new`] or [`GHRepo::from_str_with_owner`] if given
    /// an invalid GitHub repository owner name; the field is the owner name in
    /// question.
    InvalidOwner(String),

    /// Returned by [`GHRepo::new`] if given an invalid GitHub repository name;
    /// the field is the name in question.
    InvalidName(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidSpec(s) => write!(f, "Invalid GitHub repository spec: {:?}", s),
            ParseError::InvalidOwner(s) => write!(f, "Invalid GitHub repository owner: {:?}", s),
            ParseError::InvalidName(s) => write!(f, "Invalid GitHub repository name: {:?}", s),
        }
    }
}

impl error::Error for ParseError {}

/// A container for a GitHub repository's owner and base name.
///
/// A `GHRepo` instance can be constructed in the following ways:
///
/// - From an owner and name with [`GHRepo::new()`]
/// - From a GitHub URL with [`GHRepo::from_url()`]
/// - From a GitHub URL or a string of the form `{owner}/{name}` with
///   [`GHRepo::from_str`]
/// - From a GitHub URL, a string of the form `{owner}/{name}`, or a bare
///   repository name with the owner defaulting to a given value with
///   [`GHRepo::from_str_with_owner()`]
///
/// Displaying a `GHRepo` instance produces a repository "fullname" of the form
/// `{owner}/{name}`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GHRepo {
    owner: String,
    name: String,
}

impl GHRepo {
    /// Construct a [`GHRepo`] with the given owner and repository name
    ///
    /// # Errors
    ///
    /// If `owner` is not a valid GitHub owner name, or if `name` is not a
    /// valid GitHub repository name, returns [`ParseError`].
    pub fn new(owner: &str, name: &str) -> Result<Self, ParseError> {
        if !GHRepo::is_valid_owner(owner) {
            Err(ParseError::InvalidOwner(owner.to_string()))
        } else if !GHRepo::is_valid_name(name) {
            Err(ParseError::InvalidName(name.to_string()))
        } else {
            Ok(GHRepo {
                owner: owner.to_string(),
                name: name.to_string(),
            })
        }
    }

    /// Test whether a string is a valid GitHub user login or organization
    /// name.
    ///
    /// Note that the restrictions on GitHub usernames have changed over time,
    /// and this function endeavors to accept all usernames that were valid at
    /// any point, so just because a name is accepted doesn't necessarily mean
    /// you can create a user by that name on GitHub today.
    ///
    /// ```
    /// # use ghrepo::GHRepo;
    /// assert!(GHRepo::is_valid_owner("octocat"));
    /// assert!(GHRepo::is_valid_owner("octo-cat"));
    /// assert!(!GHRepo::is_valid_owner("octo.cat"));
    /// assert!(!GHRepo::is_valid_owner("octocat/repository"));
    /// assert!(!GHRepo::is_valid_owner("none"));
    /// ```
    pub fn is_valid_owner(s: &str) -> bool {
        lazy_static! {
            static ref RGX: Regex = Regex::new(format!("^{GH_OWNER_RGX}$").as_str()).unwrap();
        }
        RGX.is_match(s) && s.to_ascii_lowercase() != "none"
    }

    /// Test whether a string is a valid repository name.
    ///
    /// Note that valid repository names do not include the ".git" suffix.
    ///
    /// ```
    /// # use ghrepo::GHRepo;
    /// assert!(GHRepo::is_valid_name("my-repo"));
    /// assert!(!GHRepo::is_valid_name("my-repo.git"));
    /// assert!(!GHRepo::is_valid_owner("octocat/my-repo"));
    /// ```
    pub fn is_valid_name(s: &str) -> bool {
        lazy_static! {
            static ref RGX: Regex = Regex::new(format!("^{GH_REPO_RGX}$").as_str()).unwrap();
        }
        RGX.is_match(s) && !s.to_ascii_lowercase().ends_with(".git")
    }

    /// Like [`GHRepo::from_str()`], except that if `s` is just a repository
    /// name without an owner, the owner will be set to `owner`
    ///
    /// # Errors
    /// Returns a [`ParseError`] for the same circumstances as
    /// [`GHRepo::from_str`], or if `s` is a valid repository name but `owner`
    /// is not a valid owner name
    ///
    /// # Example
    ///
    /// ```
    /// # use std::error::Error;
    /// # use ghrepo::GHRepo;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let repo = GHRepo::from_str_with_owner("octocat/repository", "foobar")?;
    /// assert_eq!(repo.owner(), "octocat");
    /// assert_eq!(repo.name(), "repository");
    ///
    /// let repo = GHRepo::from_str_with_owner("repository", "foobar")?;
    /// assert_eq!(repo.owner(), "foobar");
    /// assert_eq!(repo.name(), "repository");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn from_str_with_owner(s: &str, owner: &str) -> Result<Self, ParseError> {
        if GHRepo::is_valid_name(s) {
            GHRepo::new(owner, s)
        } else {
            GHRepo::from_str(s)
        }
    }

    /// Retrieve the repository's owner's name
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Retrieve the repository's base name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the base URL for accessing the repository via the GitHub REST
    /// API; this is a string of the form
    /// `https://api.github.com/repos/{owner}/{name}`.
    pub fn api_url(&self) -> String {
        format!("https://api.github.com/repos/{}/{}", self.owner, self.name)
    }

    /// Returns the URL for cloning the repository over HTTPS
    pub fn clone_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.owner, self.name)
    }

    /// Returns the URL for cloning the repository via the native Git protocol
    pub fn git_url(&self) -> String {
        format!("git://github.com/{}/{}.git", self.owner, self.name)
    }

    /// Returns the URL for the repository's web interface
    pub fn html_url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.name)
    }

    /// Returns the URL for cloning the repository over SSH
    pub fn ssh_url(&self) -> String {
        format!("git@github.com:{}/{}.git", self.owner, self.name)
    }

    /// Parse a GitHub repository URL.  The following URL formats are
    /// recognized:
    ///
    /// - `[https://[<username>[:<password>]@]][www.]github.com/<owner>/<name>[.git][/]`
    /// - `[https://]api.github.com/repos/<owner>/<name>`
    /// - `git://github.com/<owner>/<name>[.git]`
    /// - `[ssh://]git@github.com:<owner>/<name>[.git]`
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the given URL is not in one of the above
    /// formats
    pub fn from_url(s: &str) -> Result<Self, ParseError> {
        lazy_static! {
            static ref GITHUB_URL_CREGEXEN: [Regex; 4] = [
                Regex::new(format!(
                    r"^(?:https?://(?:[^@:/]+(?::[^@/]+)?@)?)?(?:www\.)?github\.com/{}(?:\.git)?/?$",
                    *OWNER_NAME,
                ).as_str())
                .unwrap(),
                Regex::new(format!(
                    r"^(?:https?://)?api\.github\.com/repos/{}$",
                    *OWNER_NAME
                ).as_str())
                .unwrap(),
                Regex::new(
                    format!(r"^git://github\.com/{}(?:\.git)?$", *OWNER_NAME).as_str()
                ).unwrap(),
                Regex::new(format!(
                    r"^(?:ssh://)?git@github\.com:{}(?:\.git)?$",
                    *OWNER_NAME
                ).as_str())
                .unwrap(),
            ];
        }
        for crgx in &*GITHUB_URL_CREGEXEN {
            if let Some(caps) = crgx.captures(s) {
                return match GHRepo::new(
                    caps.name("owner").unwrap().as_str(),
                    caps.name("name").unwrap().as_str(),
                ) {
                    r @ Ok(_) => r,
                    // If the string matched a URL regex but had a bad owner or
                    // name (e.g., an owner of "none"), ensure the returned
                    // error reports the full string rather than just the bad
                    // segment
                    Err(_) => Err(ParseError::InvalidSpec(s.to_string())),
                };
            }
        }
        Err(ParseError::InvalidSpec(s.to_string()))
    }
}

impl fmt::Display for GHRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

impl FromStr for GHRepo {
    type Err = ParseError;

    /// Parse a GitHub repository specifier.  This can be either a URL (as
    /// accepted by [`GHRepo::from_url()`]) or a string in the form
    /// `{owner}/{name}`.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if `s` is not a valid URL or repository
    /// specifier
    fn from_str(s: &str) -> Result<Self, ParseError> {
        lazy_static! {
            static ref RGX: Regex = Regex::new(format!("^{}$", *OWNER_NAME).as_str()).unwrap();
        }
        if let Some(caps) = RGX.captures(s) {
            return match GHRepo::new(
                caps.name("owner").unwrap().as_str(),
                caps.name("name").unwrap().as_str(),
            ) {
                r @ Ok(_) => r,
                // If the string has a bad owner or name (e.g., an owner of
                // "none"), ensure the returned error reports the full string
                // rather than just the bad segment
                Err(_) => Err(ParseError::InvalidSpec(s.to_string())),
            };
        }
        GHRepo::from_url(s)
    }
}

/// A local Git repository.
///
/// This struct provides a small number of methods for inspecting a local Git
/// repository, generally with the goal of determining the GitHub repository
/// that it's a clone of.
///
/// The custom methods all require Git to be installed in order to work.  I am
/// not certain of the minimal viable Git version, but they should work with
/// any Git as least as far back as version 1.7.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocalRepo {
    path: PathBuf,
}

impl LocalRepo {
    /// Create a [`LocalRepo`] for operating on the repository at or containing
    /// the directory `dirpath`.
    ///
    /// No validation is done as to whether `dirpath` is a Git repository or
    /// even an extant directory.
    pub fn new<P: AsRef<Path>>(dirpath: P) -> Self {
        LocalRepo {
            path: dirpath.as_ref().to_path_buf(),
        }
    }

    /// Create a [`LocalRepo`] for operating on the repository at or containing
    /// the current directory.
    ///
    /// The path to the current directory is saved at the time the function is
    /// called; thus, if the current directory changes later, the `LocalRepo`
    /// will continue to operate on the original directory.
    ///
    /// # Errors
    ///
    /// Returns failures from [`std::env::current_dir()`]
    pub fn for_cwd() -> Result<Self, io::Error> {
        Ok(LocalRepo {
            path: env::current_dir()?,
        })
    }

    /// Returns the path that was given to [`LocalRepo::new()`] or obtained by
    /// [`LocalRepo::for_cwd()`]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Tests whether the directory is either a Git repository or contained in
    /// one
    ///
    /// # Errors
    ///
    /// Returns a [`LocalRepoError`] if the invoked Git commit fails to execute
    pub fn is_git_repo(&self) -> Result<bool, LocalRepoError> {
        Ok(Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .current_dir(&self.path)
            .status()?
            .success())
    }

    /// (Private) Run a Git command in the local repository and return the
    /// trimmed output
    fn read(&self, args: &[&str]) -> Result<String, LocalRepoError> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()?;
        if out.status.success() {
            Ok(str::from_utf8(&out.stdout)?.trim().to_string())
        } else {
            Err(LocalRepoError::CommandFailed(out.status))
        }
    }

    /// Get the current branch of the repository
    ///
    /// # Errors
    ///
    /// Returns a [`LocalRepoError`] if the invoked Git commit fails to execute
    /// or returns a nonzero status, if the command's output is invalid UTF-8,
    /// or if the repository is in a detached `HEAD` state
    pub fn current_branch(&self) -> Result<String, LocalRepoError> {
        match self.read(&["symbolic-ref", "--short", "-q", "HEAD"]) {
            Err(LocalRepoError::CommandFailed(rc)) if rc.code() == Some(1) => {
                Err(LocalRepoError::DetachedHead)
            }
            r => r,
        }
    }

    /// Determines the GitHub repository that the local repository is a clone
    /// of by parsing the URL for the specified Git remote
    ///
    /// # Errors
    ///
    /// Returns a [`LocalRepoError`] if the invoked Git commit fails to execute
    /// or returns a nonzero status, if the command's output is invalid UTF-8,
    /// if the given remote does not exist, or if the URL for the given remote
    /// is not a valid GitHub URL
    pub fn github_remote(&self, remote: &str) -> Result<GHRepo, LocalRepoError> {
        match self.read(&["remote", "get-url", "--", remote]) {
            Ok(url) => Ok(GHRepo::from_url(&url)?),
            Err(LocalRepoError::CommandFailed(r)) if r.code() == Some(2) => {
                Err(LocalRepoError::NoSuchRemote(remote.to_string()))
            }
            Err(e) => Err(e),
        }
    }

    /// Determines the GitHub repository for the upstream remote of the given
    /// branch of the local repository
    ///
    /// # Errors
    ///
    /// Returns a [`LocalRepoError`] if the invoked Git commit fails to execute
    /// or returns a nonzero status, if the command's output is invalid UTF-8,
    /// if the branch does not have a remote configured, if the remote
    /// does not exist, or if the URL for the remote is not a valid GitHub URL
    pub fn branch_upstream(&self, branch: &str) -> Result<GHRepo, LocalRepoError> {
        match self.read(&[
            "config",
            "--get",
            "--",
            format!("branch.{branch}.remote").as_str(),
        ]) {
            Ok(upstream) => self.github_remote(&upstream),
            Err(LocalRepoError::CommandFailed(r)) if r.code() == Some(1) => {
                Err(LocalRepoError::NoUpstream(branch.to_string()))
            }
            Err(e) => Err(e),
        }
    }
}

/// Error returned when a [`LocalRepo`] method fails
#[derive(Debug)]
pub enum LocalRepoError {
    /// Returned when the Git command could not be executed
    CouldNotExecute(io::Error),

    /// Returned when the Git command returned nonzero
    CommandFailed(ExitStatus),

    /// Returned by [`LocalRepo::current_branch()`] if the repository is in a
    /// detached `HEAD` state
    DetachedHead,

    /// Returned by [`LocalRepo::github_remote()`] if the named remote does not
    /// exist.  The field is the name of the nonexistent remote.
    NoSuchRemote(String),

    /// Returned by [`LocalRepo::branch_upstream()`] if the given branch does
    /// not have an upstream remote configured.  (This includes the situation
    /// in which the branch does not exist.)  The field is the name of the
    /// queried branch.
    NoUpstream(String),

    /// Returned when the output from Git could not be decoded
    InvalidUtf8(str::Utf8Error),

    /// Returned when the remote URL is not a GitHub URL
    InvalidRemoteURL(ParseError),
}

impl fmt::Display for LocalRepoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LocalRepoError::CouldNotExecute(e) => {
                write!(f, "Failed to execute Git command: {}", e)
            }
            LocalRepoError::CommandFailed(r) => match r.code() {
                Some(rc) => write!(f, "Git command exited with return code {}", rc),
                None => write!(f, "Git command was killed by a signal"),
            },
            LocalRepoError::DetachedHead => {
                write!(f, "Git repository is in a detached HEAD state")
            }
            LocalRepoError::NoSuchRemote(remote) => {
                write!(f, "No such remote in Git repository: {}", remote)
            }
            LocalRepoError::NoUpstream(branch) => {
                write!(
                    f,
                    "No upstream remote configured for Git branch: {}",
                    branch
                )
            }
            LocalRepoError::InvalidUtf8(e) => {
                write!(f, "Failed to decode output from Git command: {}", e)
            }
            LocalRepoError::InvalidRemoteURL(e) => {
                write!(f, "Repository remote URL is not a GitHub URL: {}", e)
            }
        }
    }
}

impl error::Error for LocalRepoError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            LocalRepoError::CouldNotExecute(e) => Some(e),
            LocalRepoError::CommandFailed(_) => None,
            LocalRepoError::DetachedHead => None,
            LocalRepoError::NoSuchRemote(_) => None,
            LocalRepoError::NoUpstream(_) => None,
            LocalRepoError::InvalidUtf8(e) => Some(e),
            LocalRepoError::InvalidRemoteURL(e) => Some(e),
        }
    }
}

impl From<io::Error> for LocalRepoError {
    fn from(e: io::Error) -> LocalRepoError {
        LocalRepoError::CouldNotExecute(e)
    }
}

impl From<str::Utf8Error> for LocalRepoError {
    fn from(e: str::Utf8Error) -> LocalRepoError {
        LocalRepoError::InvalidUtf8(e)
    }
}

impl From<ParseError> for LocalRepoError {
    fn from(e: ParseError) -> LocalRepoError {
        LocalRepoError::InvalidRemoteURL(e)
    }
}

/// Show current GitHub repository
#[derive(Debug, Parser)]
#[clap(version)]
#[doc(hidden)]
pub struct Arguments {
    /// Output JSON
    #[clap(short = 'J', long)]
    pub json: bool,

    /// Parse the GitHub URL from the given remote
    #[clap(short, long, default_value = "origin")]
    pub remote: String,

    /// Path to a clone of a GitHub repo  [default: current directory]
    pub dirpath: Option<String>,
}

#[doc(hidden)]
/// The implementation of the command-line interface
pub fn run(args: &Arguments) -> Result<String, LocalRepoError> {
    let lr = match &args.dirpath {
        Some(p) => LocalRepo::new(&p),
        None => LocalRepo::for_cwd()?,
    };
    let gr = lr.github_remote(&args.remote)?;
    if args.json {
        let data = json!({
            "owner": gr.owner(),
            "name": gr.name(),
            "fullname": gr.to_string(),
            "api_url": gr.api_url(),
            "clone_url": gr.clone_url(),
            "git_url": gr.git_url(),
            "html_url": gr.html_url(),
            "ssh_url": gr.ssh_url(),
        });
        Ok(serde_json::to_string_pretty(&data).unwrap())
    } else {
        Ok(gr.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::{apply, template};
    use std::ffi::OsStr;
    use std::fs;
    use std::io::{Error, ErrorKind};
    use std::str::FromStr;
    use tempfile::{tempdir, TempDir};
    use which::which;

    #[cfg(unix)]
    use std::os::unix::ffi::OsStrExt;

    struct RepoMaker {
        tmpdir: TempDir,
    }

    impl RepoMaker {
        fn new() -> Self {
            RepoMaker {
                tmpdir: tempdir().unwrap(),
            }
        }

        fn path(&self) -> &Path {
            self.tmpdir.path()
        }

        fn run<S: AsRef<OsStr>>(&self, args: &[S]) {
            let r = Command::new("git")
                .args(args)
                .current_dir(self.path())
                .status()
                .unwrap();
            assert!(r.success());
        }

        fn init(&self, branch: &str) {
            self.run(&[
                "-c",
                format!("init.defaultBranch={branch}").as_str(),
                "init",
            ])
        }

        fn add_remote<S: AsRef<OsStr>>(&self, remote: &str, url: S) {
            self.run(&[
                "remote".as_ref(),
                "add".as_ref(),
                remote.as_ref(),
                url.as_ref(),
            ])
        }

        fn set_upstream(&self, branch: &str, remote: &str) {
            self.run(&["config", format!("branch.{branch}.remote").as_str(), remote])
        }

        fn detach(&self) {
            fs::write(self.path().join("file.txt"), b"This is test text\n").unwrap();
            self.run(&["add", "file.txt"]);
            self.run(&["commit", "-m", "Add a file"]);
            fs::write(self.path().join("file2.txt"), b"This is also text\n").unwrap();
            self.run(&["add", "file2.txt"]);
            self.run(&["commit", "-m", "Add another file"]);
            self.run(&["checkout", "HEAD^"]);
        }
    }

    #[test]
    fn test_to_string() {
        let r = GHRepo::new("octocat", "repository").unwrap();
        assert_eq!(r.to_string(), "octocat/repository");
    }

    #[test]
    fn test_api_url() {
        let r = GHRepo::new("octocat", "repository").unwrap();
        assert_eq!(
            r.api_url(),
            "https://api.github.com/repos/octocat/repository"
        );
        assert_eq!(r.api_url().parse::<GHRepo>(), Ok(r));
    }

    #[test]
    fn test_clone_url() {
        let r = GHRepo::new("octocat", "repository").unwrap();
        assert_eq!(r.clone_url(), "https://github.com/octocat/repository.git");
        assert_eq!(r.clone_url().parse::<GHRepo>(), Ok(r));
    }

    #[test]
    fn test_git_url() {
        let r = GHRepo::new("octocat", "repository").unwrap();
        assert_eq!(r.git_url(), "git://github.com/octocat/repository.git");
        assert_eq!(r.git_url().parse::<GHRepo>(), Ok(r));
    }

    #[test]
    fn test_html_url() {
        let r = GHRepo::new("octocat", "repository").unwrap();
        assert_eq!(r.html_url(), "https://github.com/octocat/repository");
        assert_eq!(r.html_url().parse::<GHRepo>(), Ok(r));
    }

    #[test]
    fn test_ssh_url() {
        let r = GHRepo::new("octocat", "repository").unwrap();
        assert_eq!(r.ssh_url(), "git@github.com:octocat/repository.git");
        assert_eq!(r.ssh_url().parse::<GHRepo>(), Ok(r));
    }

    #[rstest]
    #[case("steven-universe")]
    #[case("steven")]
    #[case("s")]
    #[case("s-u")]
    #[case("7152")]
    #[case("s-t-e-v-e-n")]
    #[case("s-t-eeeeee-v-e-n")]
    #[case("peridot-2F5L-5XG")]
    #[case("nonely")]
    #[case("none-one")]
    #[case("none-none")]
    #[case("nonenone")]
    #[case("none0")]
    #[case("0none")]
    // The following are actual usernames on GitHub that violate the current
    // username restrictions:
    #[case("-")]
    #[case("-Jerry-")]
    #[case("-SFT-Clan")]
    #[case("123456----")]
    #[case("FirE-Fly-")]
    #[case("None-")]
    #[case("alex--evil")]
    #[case("johan--")]
    #[case("pj_nitin")]
    #[case("up_the_irons")]
    fn test_good_owner(#[case] owner: &str) {
        assert!(GHRepo::is_valid_owner(owner));
    }

    #[rstest]
    #[case("steven.universe")]
    #[case("steven-universe@beachcity.dv")]
    #[case("steven-univerß")]
    #[case("")]
    #[case("none")]
    #[case("NONE")]
    fn test_bad_owner(#[case] owner: &str) {
        assert!(!GHRepo::is_valid_owner(owner));
    }

    #[rstest]
    #[case("steven-universe")]
    #[case("steven")]
    #[case("s")]
    #[case("s-u")]
    #[case("7152")]
    #[case("s-t-e-v-e-n")]
    #[case("s-t-eeeeee-v-e-n")]
    #[case("peridot-2F5L-5XG")]
    #[case("...")]
    #[case("-steven")]
    #[case("steven-")]
    #[case("-steven-")]
    #[case("steven.universe")]
    #[case("steven_universe")]
    #[case("steven--universe")]
    #[case("s--u")]
    #[case("git.steven")]
    #[case("steven.git.txt")]
    #[case("steven.gitt")]
    #[case(".gitt")]
    #[case("git")]
    #[case("-")]
    #[case("_")]
    #[case("---")]
    #[case(".---")]
    #[case(".steven")]
    fn test_good_name(#[case] name: &str) {
        assert!(GHRepo::is_valid_name(name));
    }

    #[rstest]
    #[case("steven-univerß")]
    #[case(".")]
    #[case("..")]
    #[case("...git")]
    #[case("..git")]
    #[case(".git")]
    #[case("")]
    #[case("steven.git")]
    #[case("steven.GIT")]
    #[case("steven.Git")]
    fn test_bad_name(#[case] name: &str) {
        assert!(!GHRepo::is_valid_name(name));
    }

    #[template]
    #[rstest]
    #[case("git://github.com/jwodder/headerparser", "jwodder", "headerparser")]
    #[case("git://github.com/jwodder/headerparser.git", "jwodder", "headerparser")]
    #[case("git@github.com:jwodder/headerparser", "jwodder", "headerparser")]
    #[case("git@github.com:jwodder/headerparser.git", "jwodder", "headerparser")]
    #[case("ssh://git@github.com:jwodder/headerparser", "jwodder", "headerparser")]
    #[case(
        "ssh://git@github.com:jwodder/headerparser.git",
        "jwodder",
        "headerparser"
    )]
    #[case(
        "https://api.github.com/repos/jwodder/headerparser",
        "jwodder",
        "headerparser"
    )]
    #[case("https://github.com/jwodder/headerparser", "jwodder", "headerparser")]
    #[case(
        "https://github.com/jwodder/headerparser.git",
        "jwodder",
        "headerparser"
    )]
    #[case("https://github.com/jwodder/headerparser/", "jwodder", "headerparser")]
    #[case(
        "https://www.github.com/jwodder/headerparser",
        "jwodder",
        "headerparser"
    )]
    #[case("http://github.com/jwodder/headerparser", "jwodder", "headerparser")]
    #[case(
        "http://www.github.com/jwodder/headerparser",
        "jwodder",
        "headerparser"
    )]
    #[case("github.com/jwodder/headerparser", "jwodder", "headerparser")]
    #[case("www.github.com/jwodder/headerparser", "jwodder", "headerparser")]
    #[case("https://github.com/jwodder/none.git", "jwodder", "none")]
    #[case(
        "https://x-access-token:1234567890@github.com/octocat/Hello-World",
        "octocat",
        "Hello-World"
    )]
    #[case(
        "https://my.username@github.com/octocat/Hello-World",
        "octocat",
        "Hello-World"
    )]
    fn repo_urls(#[case] url: &str, owner: &str, name: &str) {}

    #[template]
    #[rstest]
    #[case("https://github.com/none/headerparser.git")]
    #[case("/repo")]
    #[case("none/repo")]
    #[case("jwodder/headerparser.git")]
    #[case("jwodder/headerparser.GIT")]
    #[case("jwodder/headerparser.Git")]
    #[case("jwodder/")]
    #[case("headerparser")]
    #[case("https://api.github.com/repos/jwodder/headerparser.git")]
    #[case("https://api.github.com/repos/jwodder/headerparser.git/")]
    #[case("https://api.github.com/repos/jwodder/headerparser/")]
    fn bad_repos(#[case] url: &str) {}

    #[apply(repo_urls)]
    #[case("jwodder/headerparser", "jwodder", "headerparser")]
    #[case("jwodder/none", "jwodder", "none")]
    #[case("nonely/headerparser", "nonely", "headerparser")]
    #[case("none-none/headerparser", "none-none", "headerparser")]
    #[case("nonenone/headerparser", "nonenone", "headerparser")]
    fn test_from_str(#[case] spec: &str, #[case] owner: &str, #[case] name: &str) {
        let r = GHRepo::new(owner, name).unwrap();
        assert_eq!(GHRepo::from_str(spec), Ok(r));
    }

    #[apply(bad_repos)]
    fn test_from_bad_str(#[case] spec: &str) {
        match GHRepo::from_str(spec) {
            Err(ParseError::InvalidSpec(s)) if s == spec => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[apply(repo_urls)]
    fn test_from_url(#[case] url: &str, #[case] owner: &str, #[case] name: &str) {
        let r = GHRepo::new(owner, name).unwrap();
        assert_eq!(GHRepo::from_url(url), Ok(r));
    }

    #[apply(bad_repos)]
    fn test_from_bad_url(#[case] url: &str) {
        match GHRepo::from_url(url) {
            Err(ParseError::InvalidSpec(s)) if s == url => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[rstest]
    #[case("headerparser", "jwodder", "headerparser")]
    #[case("none", "jwodder", "none")]
    #[case("octocat/repository", "octocat", "repository")]
    #[case("https://github.com/octocat/repository", "octocat", "repository")]
    fn test_from_str_with_owner(#[case] spec: &str, #[case] owner: &str, #[case] name: &str) {
        let r = GHRepo::new(owner, name).unwrap();
        assert_eq!(GHRepo::from_str_with_owner(spec, "jwodder"), Ok(r));
    }

    #[test]
    fn test_local_repo_new() {
        let lr = LocalRepo::new("/path/to/repo");
        assert_eq!(lr.path().to_str().unwrap(), "/path/to/repo");
    }

    #[test]
    fn test_local_repo_for_cwd() {
        let lr = LocalRepo::for_cwd().unwrap();
        let cwd = env::current_dir().unwrap();
        assert_eq!(lr.path().to_path_buf(), cwd);
    }

    #[test]
    fn test_is_git_repo_empty() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = tempdir().unwrap();
        let lr = LocalRepo::new(tmp_path.path());
        assert!(!lr.is_git_repo().unwrap());
    }

    #[test]
    fn test_is_git_repo_initted() {
        if which("git").is_err() {
            return;
        }
        let maker = RepoMaker::new();
        maker.init("main");
        let lr = LocalRepo::new(maker.path());
        assert!(lr.is_git_repo().unwrap());
    }

    #[test]
    fn test_current_branch_empty() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = tempdir().unwrap();
        let lr = LocalRepo::new(tmp_path.path());
        match lr.current_branch() {
            Err(LocalRepoError::CommandFailed(_)) => (),
            e => panic!("Git command did not fail; got: {:?}", e),
        }
    }

    #[test]
    fn test_current_branch() {
        if which("git").is_err() {
            return;
        }
        let maker = RepoMaker::new();
        maker.init("trunk");
        let lr = LocalRepo::new(maker.path());
        match lr.current_branch() {
            Ok(b) if b == "trunk" => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_current_branch_detached() {
        if which("git").is_err() {
            return;
        }
        let maker = RepoMaker::new();
        maker.init("trunk");
        maker.detach();
        let lr = LocalRepo::new(maker.path());
        match lr.current_branch() {
            Err(LocalRepoError::DetachedHead) => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_github_remote_empty() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = tempdir().unwrap();
        let lr = LocalRepo::new(tmp_path.path());
        match lr.github_remote("origin") {
            Err(e @ LocalRepoError::CommandFailed(_)) => {
                assert_eq!(e.to_string(), "Git command exited with return code 128")
            }
            e => panic!("Git command did not fail; got: {:?}", e),
        }
    }

    #[test]
    fn test_github_remote_no_remote() {
        if which("git").is_err() {
            return;
        }
        let maker = RepoMaker::new();
        maker.init("trunk");
        let lr = LocalRepo::new(maker.path());
        match lr.github_remote("origin") {
            Err(LocalRepoError::NoSuchRemote(rem)) if rem == "origin" => (),
            e => panic!("Git command did not fail; got: {:?}", e),
        }
    }

    #[test]
    fn test_github_remote() {
        if which("git").is_err() {
            return;
        }
        let repo = GHRepo::new("octocat", "repository").unwrap();
        let maker = RepoMaker::new();
        maker.init("trunk");
        maker.add_remote("origin", &repo.ssh_url());
        let lr = LocalRepo::new(maker.path());
        match lr.github_remote("origin") {
            Ok(lr) if lr == repo => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_github_remote_invalid_url() {
        if which("git").is_err() {
            return;
        }
        let maker = RepoMaker::new();
        maker.init("trunk");
        maker.add_remote("upstream", "https://git.example.com/repo.git");
        let lr = LocalRepo::new(maker.path());
        match lr.github_remote("upstream") {
            Err(LocalRepoError::InvalidRemoteURL(_)) => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_github_remote_non_utf8_url() {
        if which("git").is_err() {
            return;
        }
        let maker = RepoMaker::new();
        maker.init("trunk");
        maker.add_remote("upstream", OsStr::from_bytes(b"../f\xF6\xF6.git"));
        let lr = LocalRepo::new(maker.path());
        match lr.github_remote("upstream") {
            Err(ref e @ LocalRepoError::InvalidUtf8(eu)) => assert_eq!(
                e.to_string(),
                format!("Failed to decode output from Git command: {eu}")
            ),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_branch_upstream_no_upstream() {
        if which("git").is_err() {
            return;
        }
        let maker = RepoMaker::new();
        maker.init("trunk");
        let lr = LocalRepo::new(maker.path());
        match lr.branch_upstream("trunk") {
            Err(LocalRepoError::NoUpstream(branch)) if branch == "trunk" => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_branch_upstream() {
        if which("git").is_err() {
            return;
        }
        let repo = GHRepo::new("octocat", "repository").unwrap();
        let maker = RepoMaker::new();
        maker.init("trunk");
        maker.add_remote("github", &repo.clone_url());
        maker.set_upstream("trunk", "github");
        let lr = LocalRepo::new(maker.path());
        match lr.branch_upstream("trunk") {
            Ok(r) if r == repo => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_run() {
        if which("git").is_err() {
            return;
        }
        let repo = GHRepo::new("octocat", "repository").unwrap();
        let maker = RepoMaker::new();
        maker.init("trunk");
        maker.add_remote("origin", &repo.ssh_url());
        let args = Arguments {
            json: false,
            remote: "origin".to_string(),
            dirpath: Some(maker.path().to_str().unwrap().to_string()),
        };
        match run(&args) {
            Ok(s) if s == "octocat/repository" => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_run_json() {
        if which("git").is_err() {
            return;
        }
        let repo = GHRepo::new("octocat", "repository").unwrap();
        let expected = "{
  \"owner\": \"octocat\",
  \"name\": \"repository\",
  \"fullname\": \"octocat/repository\",
  \"api_url\": \"https://api.github.com/repos/octocat/repository\",
  \"clone_url\": \"https://github.com/octocat/repository.git\",
  \"git_url\": \"git://github.com/octocat/repository.git\",
  \"html_url\": \"https://github.com/octocat/repository\",
  \"ssh_url\": \"git@github.com:octocat/repository.git\"
}";
        let maker = RepoMaker::new();
        maker.init("trunk");
        maker.add_remote("origin", &repo.ssh_url());
        let args = Arguments {
            json: true,
            remote: "origin".to_string(),
            dirpath: Some(maker.path().to_str().unwrap().to_string()),
        };
        match run(&args) {
            Ok(s) if s == expected => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_display_parse_error_invalid_spec() {
        let e = ParseError::InvalidSpec("foo.bar".to_string());
        assert_eq!(e.to_string(), "Invalid GitHub repository spec: \"foo.bar\"");
    }

    #[test]
    fn test_display_parse_error_invalid_owner() {
        let e = ParseError::InvalidOwner("foo.bar".to_string());
        assert_eq!(
            e.to_string(),
            "Invalid GitHub repository owner: \"foo.bar\""
        );
    }

    #[test]
    fn test_display_parse_error_invalid_name() {
        let e = ParseError::InvalidName("foo.git".to_string());
        assert_eq!(e.to_string(), "Invalid GitHub repository name: \"foo.git\"");
    }

    #[test]
    fn test_display_local_repo_error_could_not_execute() {
        let e = LocalRepoError::CouldNotExecute(Error::from(ErrorKind::NotFound));
        assert_eq!(
            e.to_string(),
            format!(
                "Failed to execute Git command: {}",
                Error::from(ErrorKind::NotFound)
            )
        );
    }

    #[test]
    fn test_display_local_repo_error_detached_head() {
        let e = LocalRepoError::DetachedHead;
        assert_eq!(e.to_string(), "Git repository is in a detached HEAD state");
    }

    #[test]
    fn test_display_local_repo_error_no_such_remote() {
        let e = LocalRepoError::NoSuchRemote("origin".to_string());
        assert_eq!(e.to_string(), "No such remote in Git repository: origin");
    }

    #[test]
    fn test_display_local_repo_error_no_upstream() {
        let e = LocalRepoError::NoUpstream("main".to_string());
        assert_eq!(
            e.to_string(),
            "No upstream remote configured for Git branch: main"
        );
    }

    #[test]
    fn test_display_local_repo_error_parse_error() {
        let e = LocalRepoError::InvalidRemoteURL(ParseError::InvalidSpec("foo.bar".to_string()));
        assert_eq!(e.to_string(), "Repository remote URL is not a GitHub URL: Invalid GitHub repository spec: \"foo.bar\"");
    }
}
