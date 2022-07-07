#[cfg(test)]
extern crate rstest_reuse;

use ghrepo::{GHRepo, ParseError};
use rstest::rstest;
use rstest_reuse::{apply, template};
use std::str::FromStr;

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
fn test_new_bad_owner() {
    assert_eq!(
        GHRepo::new("None", "repo.git"),
        Err(ParseError::InvalidOwner(String::from("None")))
    );
}

#[test]
fn test_new_bad_repo() {
    assert_eq!(
        GHRepo::new("Noners", "repo.git"),
        Err(ParseError::InvalidName(String::from("repo.git")))
    );
}
