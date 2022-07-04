#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::fmt;
use std::str::FromStr;

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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GHRepo {
    owner: String,
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidSpec(String),
    InvalidOwner(String),
    InvalidName(String),
}

impl GHRepo {
    pub fn new(owner: &str, name: &str) -> Result<Self, Error> {
        if !GHRepo::is_valid_owner(owner) {
            Err(Error::InvalidOwner(owner.to_string()))
        } else if !GHRepo::is_valid_name(name) {
            Err(Error::InvalidName(name.to_string()))
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
        RGX.is_match(s) && !s.ends_with(".git")
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

    pub fn from_url(s: &str) -> Result<Self, Error> {
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
        return Err(Error::InvalidSpec(s.to_string()));
    }
}

impl fmt::Display for GHRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

impl FromStr for GHRepo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
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

#[cfg(test)]
mod tests {
    use super::GHRepo;

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
}
