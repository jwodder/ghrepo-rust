#[macro_use]
extern crate lazy_static;

use clap::Parser;
use regex::Regex;
use serde_json::json;
use std::error;
use std::fmt;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::str::{self, FromStr};

#[cfg(test)]
use rstest_reuse;

// Regular expression for a valid GitHub username or organization name.  As of
// 2017-07-23, trying to sign up to GitHub with an invalid username or create
// an organization with an invalid name gives the message "Username may only
// contain alphanumeric characters or single hyphens, and cannot begin or end
// with a hyphen".  Additionally, trying to create a user named "none" (case
// insensitive) gives the message "Username name 'none' is a reserved word."
//
// Unfortunately, there are a number of users who made accounts before the
// current name restrictions were put in place, and so this regex also needs to
// accept names that contain underscores, contain multiple consecutive hyphens,
// begin with a hyphen, and/or end with a hyphen.
const GH_OWNER_RGX: &str = r"[-_A-Za-z0-9]+";

// Regular expression for a valid GitHub repository name.  Testing as of
// 2017-05-21 indicates that repository names can be composed of alphanumeric
// ASCII characters, hyphens, periods, and/or underscores, with the names ``.``
// and ``..`` being reserved and names ending with ``.git`` forbidden.
const GH_REPO_RGX: &str = r"(?:\.?[-A-Za-z0-9_][-A-Za-z0-9_.]*?|\.\.[-A-Za-z0-9_.]+?)";

lazy_static! {
    /// Convenience regular expression for `<owner>/<name>`, including named
    /// capturing groups
    static ref OWNER_NAME: String = format!(r"(?P<owner>{})/(?P<name>{})", GH_OWNER_RGX, GH_REPO_RGX);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidSpec(String),
    InvalidOwner(String),
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GHRepo {
    owner: String,
    name: String,
}

impl GHRepo {
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

    pub fn is_valid_owner(s: &str) -> bool {
        lazy_static! {
            static ref RGX: Regex = Regex::new(format!("^{GH_OWNER_RGX}$").as_str()).unwrap();
        }
        RGX.is_match(s) && s.to_ascii_lowercase() != "none"
    }

    pub fn is_valid_name(s: &str) -> bool {
        lazy_static! {
            static ref RGX: Regex = Regex::new(format!("^{GH_REPO_RGX}$").as_str()).unwrap();
        }
        RGX.is_match(s) && !s.to_ascii_lowercase().ends_with(".git")
    }

    pub fn from_str_with_owner(s: &str, owner: &str) -> Result<Self, ParseError> {
        if GHRepo::is_valid_name(s) {
            GHRepo::new(owner, s)
        } else {
            GHRepo::from_str(s)
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn api_url(&self) -> String {
        format!("https://api.github.com/repos/{}/{}", self.owner, self.name)
    }

    pub fn clone_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.owner, self.name)
    }

    pub fn git_url(&self) -> String {
        format!("git://github.com/{}/{}.git", self.owner, self.name)
    }

    pub fn html_url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.name)
    }

    pub fn ssh_url(&self) -> String {
        format!("git@github.com:{}/{}.git", self.owner, self.name)
    }

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
                Regex::new(format!(r"^git://github\.com/{}(?:\.git)?$", *OWNER_NAME).as_str()).unwrap(),
                Regex::new(format!(
                    r"^(?:ssh://)?git@github\.com:{}(?:\.git)?$",
                    *OWNER_NAME
                ).as_str())
                .unwrap(),
            ];
        }
        for crgx in &*GITHUB_URL_CREGEXEN {
            if let Some(caps) = crgx.captures(s) {
                return GHRepo::new(
                    caps.name("owner").unwrap().as_str(),
                    caps.name("name").unwrap().as_str(),
                );
            }
        }
        return Err(ParseError::InvalidSpec(s.to_string()));
    }
}

impl fmt::Display for GHRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

impl FromStr for GHRepo {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        lazy_static! {
            static ref RGX: Regex = Regex::new(format!("^{}$", *OWNER_NAME).as_str()).unwrap();
        }
        if let Some(caps) = RGX.captures(s) {
            return GHRepo::new(
                caps.name("owner").unwrap().as_str(),
                caps.name("name").unwrap().as_str(),
            );
        }
        return GHRepo::from_url(s);
    }
}

pub fn is_git_repo<P: AsRef<Path>>(dirpath: Option<P>) -> Result<bool, io::Error> {
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "--git-dir"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(p) = dirpath {
        cmd.current_dir(p);
    }
    return Ok(cmd.status()?.success());
}

#[derive(Debug)]
pub enum CurrentBranchError {
    CouldNotExecute(io::Error),
    CommandFailed(ExitStatus),
    InvalidUtf8(str::Utf8Error),
}

impl fmt::Display for CurrentBranchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrentBranchError::CouldNotExecute(e) => {
                write!(f, "Failed to execute Git command: {}", e)
            }
            CurrentBranchError::CommandFailed(r) => match r.code() {
                Some(rc) => write!(f, "Git command exited with return code {}", rc),
                None => write!(f, "Git command was killed by a signal"),
            },
            CurrentBranchError::InvalidUtf8(e) => {
                write!(f, "Failed to decode output from Git command: {}", e)
            }
        }
    }
}

impl error::Error for CurrentBranchError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            CurrentBranchError::CouldNotExecute(e) => Some(e),
            CurrentBranchError::CommandFailed(_) => None,
            CurrentBranchError::InvalidUtf8(e) => Some(e),
        }
    }
}

impl From<io::Error> for CurrentBranchError {
    fn from(e: io::Error) -> CurrentBranchError {
        CurrentBranchError::CouldNotExecute(e)
    }
}

impl From<str::Utf8Error> for CurrentBranchError {
    fn from(e: str::Utf8Error) -> CurrentBranchError {
        CurrentBranchError::InvalidUtf8(e)
    }
}

pub fn get_current_branch<P: AsRef<Path>>(
    dirpath: Option<P>,
) -> Result<String, CurrentBranchError> {
    let mut cmd = Command::new("git");
    cmd.args(["symbolic-ref", "--short", "-q", "HEAD"]);
    if let Some(p) = dirpath {
        cmd.current_dir(p);
    }
    let out = cmd.output()?;
    if out.status.success() {
        Ok(str::from_utf8(&out.stdout)?.trim().to_string())
    } else {
        Err(CurrentBranchError::CommandFailed(out.status))
    }
}

#[derive(Debug)]
pub enum LocalRepoError {
    CouldNotExecute(io::Error),
    CommandFailed(ExitStatus),
    InvalidUtf8(str::Utf8Error),
    InvalidRemote(ParseError),
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
            LocalRepoError::InvalidUtf8(e) => {
                write!(f, "Failed to decode output from Git command: {}", e)
            }
            LocalRepoError::InvalidRemote(e) => {
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
            LocalRepoError::InvalidUtf8(e) => Some(e),
            LocalRepoError::InvalidRemote(e) => Some(e),
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
        LocalRepoError::InvalidRemote(e)
    }
}

pub fn get_local_repo<P: AsRef<Path>>(
    dirpath: Option<P>,
    remote: &str,
) -> Result<GHRepo, LocalRepoError> {
    let mut cmd = Command::new("git");
    cmd.args(["remote", "get-url", "--", remote]);
    if let Some(p) = dirpath {
        cmd.current_dir(p);
    }
    let out = cmd.output()?;
    if out.status.success() {
        Ok(GHRepo::from_url(str::from_utf8(&out.stdout)?.trim())?)
    } else {
        Err(LocalRepoError::CommandFailed(out.status))
    }
}

/// Show current GitHub repository
#[derive(Debug, Parser)]
#[clap(version)]
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

pub fn run(args: Arguments) -> Result<String, LocalRepoError> {
    let r = get_local_repo(args.dirpath, &args.remote)?;
    if args.json {
        let dict = json!({
            "owner": r.owner(),
            "name": r.name(),
            "fullname": r.to_string(),
            "api_url": r.api_url(),
            "clone_url": r.clone_url(),
            "git_url": r.git_url(),
            "html_url": r.html_url(),
            "ssh_url": r.ssh_url(),
        });
        Ok(serde_json::to_string_pretty(&dict).unwrap())
    } else {
        Ok(r.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::{apply, template};
    use std::str::FromStr;
    use tempfile::{tempdir, TempDir};
    use which::which;

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
        assert!(GHRepo::from_str(spec).is_err());
    }

    #[apply(repo_urls)]
    fn test_from_url(#[case] url: &str, #[case] owner: &str, #[case] name: &str) {
        let r = GHRepo::new(owner, name).unwrap();
        assert_eq!(GHRepo::from_url(url), Ok(r));
    }

    #[apply(bad_repos)]
    fn test_from_bad_url(#[case] url: &str) {
        assert!(GHRepo::from_url(url).is_err());
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

    fn mkrepo(branch: &str) -> TempDir {
        let path = tempdir().unwrap();
        let r = Command::new("git")
            .arg("-c")
            .arg(format!("init.defaultBranch={branch}"))
            .arg("init")
            .current_dir(path.path())
            .status()
            .unwrap();
        assert!(r.success());
        return path;
    }

    fn mkrepo_remote(branch: &str, remote: &str, remote_url: &str) -> TempDir {
        let path = mkrepo(branch);
        let r = Command::new("git")
            .args(["remote", "add", remote, remote_url])
            .current_dir(path.path())
            .status()
            .unwrap();
        assert!(r.success());
        return path;
    }

    #[test]
    fn test_is_git_repo_empty() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = tempdir().unwrap();
        assert!(!is_git_repo(Some(tmp_path.path())).unwrap());
    }

    #[test]
    fn test_is_git_repo_initted() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = mkrepo("main");
        assert!(is_git_repo(Some(tmp_path.path())).unwrap());
    }

    #[test]
    fn test_get_current_branch_empty() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = tempdir().unwrap();
        match get_current_branch(Some(tmp_path.path())) {
            Err(CurrentBranchError::CommandFailed(_)) => (),
            e => panic!("Git command did not fail; got: {:?}", e),
        }
    }

    #[test]
    fn test_get_current_branch() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = mkrepo("trunk");
        match get_current_branch(Some(tmp_path.path())) {
            Ok(b) if b == "trunk" => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_get_local_repo_empty() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = tempdir().unwrap();
        match get_local_repo(Some(tmp_path.path()), "origin") {
            Err(LocalRepoError::CommandFailed(_)) => (),
            e => panic!("Git command did not fail; got: {:?}", e),
        }
    }

    #[test]
    fn test_get_local_repo_no_remote() {
        if which("git").is_err() {
            return;
        }
        let tmp_path = mkrepo("trunk");
        match get_local_repo(Some(tmp_path.path()), "origin") {
            Err(LocalRepoError::CommandFailed(_)) => (),
            e => panic!("Git command did not fail; got: {:?}", e),
        }
    }

    #[test]
    fn test_get_local_repo() {
        if which("git").is_err() {
            return;
        }
        let repo = GHRepo::new("octocat", "repository").unwrap();
        let tmp_path = mkrepo_remote("trunk", "origin", &repo.ssh_url());
        match get_local_repo(Some(tmp_path.path()), "origin") {
            Ok(lr) if lr == repo => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }

    #[test]
    fn test_run() {
        if which("git").is_err() {
            return;
        }
        let repo = GHRepo::new("octocat", "repository").unwrap();
        let tmp_path = mkrepo_remote("trunk", "origin", &repo.ssh_url());
        match run(Arguments {
            json: false,
            remote: "origin".to_string(),
            dirpath: Some(tmp_path.path().to_str().unwrap().to_string()),
        }) {
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
        let tmp_path = mkrepo_remote("trunk", "origin", &repo.ssh_url());
        match run(Arguments {
            json: true,
            remote: "origin".to_string(),
            dirpath: Some(tmp_path.path().to_str().unwrap().to_string()),
        }) {
            Ok(s) if s == expected => (),
            e => panic!("Got wrong result: {:?}", e),
        }
    }
}
