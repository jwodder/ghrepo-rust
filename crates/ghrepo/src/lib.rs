#![cfg_attr(docsrs, feature(doc_cfg))]
//! `ghrepo` extracts a GitHub repository's owner & name from various GitHub
//! URL formats (or just from a string of the form `OWNER/REPONAME` or
//! `REPONAME`), and the resulting object provides properties for going in
//! reverse to determine the possible URLs.  Also included is a struct for
//! performing a couple useful inspections on local Git repositories, including
//! determining the corresponding GitHub owner & repository name.
//!
//! Features
//! ========
//!
//! The `ghrepo` crate has the following optional feature:
//!
//! - `serde` — Enables serializing & deserializing the `GHRepo` type with
//!   [`serde`]
//!
//! Example
//! =======
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

mod deser;
mod parser;
use crate::parser::{parse_github_url, split_name, split_owner, split_owner_name};
use std::cmp::Ordering;
use std::env;
use std::error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::str::{self, FromStr};

/// Error returned when trying to construct a [`GHRepo`] with invalid arguments
/// or parse an invalid repository spec
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// Returned by [`GHRepo::from_str`], [`GHRepo::from_url`],
    /// [`GHRepo::from_str_with_owner`], or the `TryFrom<String> for GHRepo`
    /// impl if given a string that is not a valid GitHub repository URL or
    /// specifier; the field is the string in question.
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidSpec(s) => write!(f, "invalid GitHub repository spec: {s:?}"),
            ParseError::InvalidOwner(s) => write!(f, "invalid GitHub repository owner: {s:?}"),
            ParseError::InvalidName(s) => write!(f, "invalid GitHub repository name: {s:?}"),
        }
    }
}

impl error::Error for ParseError {}

/// A container for a GitHub repository spec, consisting of a repository
/// *owner* and a repository *name* (sometimes also called the "repo"
/// component).
///
/// A `GHRepo` instance can be constructed in the following ways:
///
/// - From an owner and name with [`GHRepo::new()`]
/// - From a [`String`] of the form `{owner}/{name}` via the `TryFrom<String>`
///   trait
/// - From a GitHub URL with [`GHRepo::from_url()`]
/// - From a GitHub URL or a string of the form `{owner}/{name}` via the
///   [`FromStr`] trait or with `s.parse::<GHRepo>()`
/// - From a GitHub URL, a string of the form `{owner}/{name}`, or a bare
///   repository name with the owner defaulting to a given value with
///   [`GHRepo::from_str_with_owner()`]
///
/// Comparisons and conversions to strings all treat `GHRepo` instances as
/// strings of the form `{owner}/{name}` (sometimes called the repository's
/// "full name").
///
/// When the `serde` feature is enabled, `GHRepo` instances can be serialized &
/// deserialized with the `serde` library.  Serialization produces a string of
/// the form `{owner}/{name}`, and deserialization accepts any string of a form
/// accepted by [`GHRepo::from_str`].
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GHRepo {
    fullname: String,
    slash_pos: usize,
}

impl GHRepo {
    /// Construct a [`GHRepo`] with the given owner and repository name
    ///
    /// # Errors
    ///
    /// If `owner` is not a valid GitHub owner name, or if `name` is not a
    /// valid GitHub repository name, returns [`ParseError`].
    pub fn new(owner: &str, name: &str) -> Result<Self, ParseError> {
        if !is_valid_owner(owner) {
            Err(ParseError::InvalidOwner(owner.to_string()))
        } else if !is_valid_name(name) {
            Err(ParseError::InvalidName(name.to_string()))
        } else {
            Ok(GHRepo {
                fullname: format!("{owner}/{name}"),
                slash_pos: owner.len(),
            })
        }
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
        if is_valid_name(s) {
            GHRepo::new(owner, s)
        } else {
            GHRepo::from_str(s)
        }
    }

    /// Retrieve the repository's owner's name
    pub fn owner(&self) -> &str {
        match self.fullname.get(..self.slash_pos) {
            Some(s) => s,
            None => unreachable!("slash_pos should be valid char index"),
        }
    }

    /// Retrieve the repository's base name
    pub fn name(&self) -> &str {
        match self.fullname.get(self.slash_pos + 1..) {
            Some(s) => s,
            None => unreachable!("slash_pos + 1 should be valid char index"),
        }
    }

    /// Convert to a reference to a string of the form `{owner}/{name}`
    pub fn as_str(&self) -> &str {
        &self.fullname
    }

    /// Returns the base URL for accessing the repository via the GitHub REST
    /// API; this is a string of the form
    /// `https://api.github.com/repos/{owner}/{name}`.
    pub fn api_url(&self) -> String {
        format!("https://api.github.com/repos/{}", self.fullname)
    }

    /// Returns the URL for cloning the repository over HTTPS
    pub fn clone_url(&self) -> String {
        format!("https://github.com/{}.git", self.fullname)
    }

    /// Returns the URL for cloning the repository via the native Git protocol
    pub fn git_url(&self) -> String {
        format!("git://github.com/{}.git", self.fullname)
    }

    /// Returns the URL for the repository's web interface
    pub fn html_url(&self) -> String {
        format!("https://github.com/{}", self.fullname)
    }

    /// Returns the URL for cloning the repository over SSH
    pub fn ssh_url(&self) -> String {
        format!("git@github.com:{}.git", self.fullname)
    }

    /// Parse a repository from a GitHub repository URL.  The following URL
    /// formats are recognized:
    ///
    /// - `[http[s]://[<username>[:<password>]@]][www.]github.com/<owner>/<name>[.git][/]`
    /// - `[http[s]://]api.github.com/repos/<owner>/<name>`
    /// - `git://github.com/<owner>/<name>[.git]`
    /// - `git@github.com:<owner>/<name>[.git]`
    /// - `ssh://git@github.com/<owner>/<name>[.git]`
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the given URL is not in one of the above
    /// formats
    pub fn from_url(s: &str) -> Result<Self, ParseError> {
        match parse_github_url(s) {
            Some((owner, name)) => GHRepo::new(owner, name),
            None => Err(ParseError::InvalidSpec(s.to_string())),
        }
    }
}

impl From<GHRepo> for String {
    /// Convert the repository to a string of the form `{owner}/{name}`
    fn from(value: GHRepo) -> String {
        value.fullname
    }
}

impl fmt::Debug for GHRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.fullname)
    }
}

impl fmt::Display for GHRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.fullname)
    }
}

impl PartialEq<str> for GHRepo {
    /// Compare the repository as though it were a string of the form
    /// `{owner}/{name}`
    fn eq(&self, other: &str) -> bool {
        self.fullname == other
    }
}

impl<'a> PartialEq<&'a str> for GHRepo {
    /// Compare the repository as though it were a string of the form
    /// `{owner}/{name}`
    fn eq(&self, other: &&'a str) -> bool {
        &self.fullname == other
    }
}

impl PartialOrd<str> for GHRepo {
    /// Compare the repository as though it were a string of the form
    /// `{owner}/{name}`
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        Some((*self.fullname).cmp(other))
    }
}

impl<'a> PartialOrd<&'a str> for GHRepo {
    /// Compare the repository as though it were a string of the form
    /// `{owner}/{name}`
    fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
        Some((*self.fullname).cmp(other))
    }
}

impl std::ops::Deref for GHRepo {
    type Target = str;

    /// Dereference to a string of the form `{owner}/{name}`
    fn deref(&self) -> &str {
        &self.fullname
    }
}

impl AsRef<str> for GHRepo {
    /// Convert to a reference to a string of the form `{owner}/{name}`
    fn as_ref(&self) -> &str {
        &self.fullname
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
        match split_owner_name(s) {
            Some((owner, _name, "")) => Ok(GHRepo {
                fullname: s.to_owned(),
                slash_pos: owner.len(),
            }),
            _ => GHRepo::from_url(s),
        }
    }
}

impl TryFrom<String> for GHRepo {
    type Error = ParseError;

    /// Construct a `GHRepo` from a `String` of the form `{owner}/{name}`.
    ///
    /// Note that, unlike [`GHRepo::from_str()`], this trait does not accept
    /// repository URLs.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if `s` is not a valid repository specifier
    fn try_from(s: String) -> Result<Self, ParseError> {
        match split_owner_name(&s) {
            Some((owner, _name, "")) => {
                let slash_pos = owner.len();
                Ok(GHRepo {
                    fullname: s,
                    slash_pos,
                })
            }
            _ => Err(ParseError::InvalidSpec(s)),
        }
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
    /// Returns a [`LocalRepoError`] if [`std::env::current_dir()`] failed
    pub fn for_cwd() -> Result<Self, LocalRepoError> {
        Ok(LocalRepo {
            path: env::current_dir().map_err(LocalRepoError::CurdirError)?,
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
            .status()
            .map_err(LocalRepoError::CouldNotExecute)?
            .success())
    }

    /// (Private) Run a Git command in the local repository and return the
    /// trimmed output
    fn read(&self, args: &[&str]) -> Result<String, LocalRepoError> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .stderr(Stdio::inherit())
            .output()
            .map_err(LocalRepoError::CouldNotExecute)?;
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
        match self.read(&["config", "--get", "--", &format!("branch.{branch}.remote")]) {
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

    /// Returned by [`LocalRepo::for_cwd()`] if [`std::env::current_dir()`]
    /// errored
    CurdirError(io::Error),

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalRepoError::CouldNotExecute(e) => {
                write!(f, "failed to execute Git command: {e}")
            }
            LocalRepoError::CommandFailed(r) => {
                write!(f, "Git command exited unsuccessfully: {r}")
            }
            LocalRepoError::CurdirError(e) => {
                write!(f, "could not determine current directory: {e}")
            }
            LocalRepoError::DetachedHead => {
                write!(f, "Git repository is in a detached HEAD state")
            }
            LocalRepoError::NoSuchRemote(remote) => {
                write!(f, "no such remote in Git repository: {remote:?}")
            }
            LocalRepoError::NoUpstream(branch) => {
                write!(
                    f,
                    "no upstream remote configured for Git branch: {branch:?}"
                )
            }
            LocalRepoError::InvalidUtf8(e) => {
                write!(f, "failed to decode output from Git command: {e}")
            }
            LocalRepoError::InvalidRemoteURL(e) => {
                write!(f, "repository remote URL is not a GitHub URL: {e}")
            }
        }
    }
}

impl error::Error for LocalRepoError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            LocalRepoError::CouldNotExecute(e) => Some(e),
            LocalRepoError::CommandFailed(_) => None,
            LocalRepoError::CurdirError(e) => Some(e),
            LocalRepoError::DetachedHead => None,
            LocalRepoError::NoSuchRemote(_) => None,
            LocalRepoError::NoUpstream(_) => None,
            LocalRepoError::InvalidUtf8(e) => Some(e),
            LocalRepoError::InvalidRemoteURL(e) => Some(e),
        }
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

/// Test whether a string is a valid GitHub user login or organization name.
///
/// As of 2017-07-23, trying to sign up to GitHub with an invalid username or
/// create an organization with an invalid name gives the message "Username may
/// only contain alphanumeric characters or single hyphens, and cannot begin or
/// end with a hyphen".  Additionally, trying to create a user named "none"
/// (case insensitive) gives the message "Username name 'none' is a reserved
/// word."  Unfortunately, there are a number of users who made accounts before
/// the current name restrictions were put in place, and so this method also
/// needs to accept names that contain underscores, contain multiple
/// consecutive hyphens, begin with a hyphen, and/or end with a hyphen.
///
/// As this function endeavors to accept all usernames that were valid at any
/// point, just because a name is accepted doesn't necessarily mean you can
/// create a user by that name on GitHub today.
///
/// # Example
///
/// ```
/// # use ghrepo::is_valid_owner;
/// assert!(is_valid_owner("octocat"));
/// assert!(is_valid_owner("octo-cat"));
/// assert!(!is_valid_owner("octo.cat"));
/// assert!(!is_valid_owner("octocat/repository"));
/// assert!(!is_valid_owner("none"));
/// ```
pub fn is_valid_owner(s: &str) -> bool {
    matches!(split_owner(s), Some((_, "")))
}

/// Test whether a string is a valid repository name.
///
/// Testing as of 2017-05-21 indicates that repository names can be composed of
/// alphanumeric ASCII characters, hyphens, periods, and/or underscores, with
/// the names `.` and `..` being reserved and names ending with `.git` (case
/// insensitive) forbidden.
///
/// # Example
///
/// ```
/// # use ghrepo::is_valid_name;
/// assert!(is_valid_name("my-repo"));
/// assert!(!is_valid_name("my-repo.git"));
/// assert!(!is_valid_name("octocat/my-repo"));
/// ```
pub fn is_valid_name(s: &str) -> bool {
    matches!(split_name(s), Some((_, "")))
}

/// Test whether a string is a valid repository specifier/full name of the form
/// `{owner}/{name}`.
///
/// # Example
///
/// ```
/// # use ghrepo::is_valid_repository;
/// assert!(is_valid_repository("octocat/my-repo"));
/// assert!(!is_valid_repository("octocat/my-repo.git"));
/// assert!(!is_valid_repository("my-repo"));
/// ```
pub fn is_valid_repository(s: &str) -> bool {
    matches!(split_owner_name(s), Some((_, _, "")))
}
