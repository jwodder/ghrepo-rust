#![allow(clippy::items_after_test_module)]
use ghrepo::{GHRepo, ParseError};
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use std::str::FromStr;

#[test]
fn test_parts() {
    let r = GHRepo::new("octocat", "repository").unwrap();
    assert_eq!(r.owner(), "octocat");
    assert_eq!(r.name(), "repository");
}

#[test]
fn test_to_string() {
    let r = GHRepo::new("octocat", "repository").unwrap();
    assert_eq!(r.to_string(), "octocat/repository");
}

#[test]
fn test_pad() {
    let r = GHRepo::new("octocat", "repository").unwrap();
    assert_eq!(format!("{r:.^24}"), "...octocat/repository...");
}

#[test]
fn test_debug() {
    let r = GHRepo::new("octocat", "repository").unwrap();
    assert_eq!(format!("{r:?}"), r#""octocat/repository""#);
    assert_eq!(format!("{r:#?}"), r#""octocat/repository""#);
}

#[test]
fn test_misc_traits() {
    let r = GHRepo::new("octocat", "repository").unwrap();
    let s: &str = &r;
    assert_eq!(s, "octocat/repository");
    assert_eq!(r.as_ref(), "octocat/repository");
    assert_eq!(r.as_str(), "octocat/repository");
    assert_eq!(r, "octocat/repository");
    let s2 = String::from(r);
    assert_eq!(s2, "octocat/repository");
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

#[template]
#[rstest]
#[case("git://github.com/jwodder/headerparser", "jwodder", "headerparser")]
#[case("git://github.com/jwodder/headerparser.git", "jwodder", "headerparser")]
#[case("git@github.com:jwodder/headerparser", "jwodder", "headerparser")]
#[case("git@github.com:jwodder/headerparser.git", "jwodder", "headerparser")]
#[case("GIT://GitHub.COM/jwodder/headerparser", "jwodder", "headerparser")]
#[case("git@github.com:joe-q-coder/my.repo.git", "joe-q-coder", "my.repo")]
#[case("git@GITHUB.com:joe-q-coder/my.repo.git", "joe-q-coder", "my.repo")]
#[case("ssh://git@github.com/jwodder/headerparser", "jwodder", "headerparser")]
#[case(
    "ssh://git@github.com/jwodder/headerparser.git",
    "jwodder",
    "headerparser"
)]
#[case("ssh://git@github.com/-/test", "-", "test")]
#[case("SSH://git@GITHUB.COM/-/test", "-", "test")]
#[case(
    "https://api.github.com/repos/jwodder/headerparser",
    "jwodder",
    "headerparser"
)]
#[case(
    "http://api.github.com/repos/jwodder/headerparser",
    "jwodder",
    "headerparser"
)]
#[case("api.github.com/repos/jwodder/headerparser", "jwodder", "headerparser")]
#[case("https://api.github.com/repos/none-/-none", "none-", "-none")]
#[case("HttpS://api.github.com/repos/none-/-none", "none-", "-none")]
#[case("http://api.github.com/repos/none-/-none", "none-", "-none")]
#[case("Http://api.github.com/repos/none-/-none", "none-", "-none")]
#[case("Api.GitHub.Com/repos/jwodder/headerparser", "jwodder", "headerparser")]
#[case("https://github.com/jwodder/headerparser", "jwodder", "headerparser")]
#[case(
    "https://github.com/jwodder/headerparser.git",
    "jwodder",
    "headerparser"
)]
#[case(
    "https://github.com/jwodder/headerparser.git/",
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
#[case("github.com/jwodder/headerparser.git", "jwodder", "headerparser")]
#[case("github.com/jwodder/headerparser.git/", "jwodder", "headerparser")]
#[case("github.com/jwodder/headerparser/", "jwodder", "headerparser")]
#[case("www.github.com/jwodder/headerparser", "jwodder", "headerparser")]
#[case("https://github.com/jwodder/none.git", "jwodder", "none")]
#[case("https://github.com/joe-coder/hello.world", "joe-coder", "hello.world")]
#[case("http://github.com/joe-coder/hello.world", "joe-coder", "hello.world")]
#[case("HTTPS://GITHUB.COM/joe-coder/hello.world", "joe-coder", "hello.world")]
#[case(
    "HTTPS://WWW.GITHUB.COM/joe-coder/hello.world",
    "joe-coder",
    "hello.world"
)]
#[case("https://github.com/-Jerry-/geshi-1.0.git", "-Jerry-", "geshi-1.0")]
#[case("https://github.com/-Jerry-/geshi-1.0.git/", "-Jerry-", "geshi-1.0")]
#[case("https://github.com/-Jerry-/geshi-1.0/", "-Jerry-", "geshi-1.0")]
#[case("https://www.github.com/-Jerry-/geshi-1.0", "-Jerry-", "geshi-1.0")]
#[case("github.com/-Jerry-/geshi-1.0", "-Jerry-", "geshi-1.0")]
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
#[case(
    "https://user%20name@github.com/octocat/Hello-World",
    "octocat",
    "Hello-World"
)]
#[case(
    "https://user+name@github.com/octocat/Hello-World",
    "octocat",
    "Hello-World"
)]
#[case(
    "https://~user.name@github.com/octocat/Hello-World",
    "octocat",
    "Hello-World"
)]
#[case("https://@github.com/octocat/Hello-World", "octocat", "Hello-World")]
#[case(
    "https://user:@github.com/octocat/Hello-World",
    "octocat",
    "Hello-World"
)]
#[case(
    "https://:pass@github.com/octocat/Hello-World",
    "octocat",
    "Hello-World"
)]
#[case("https://:@github.com/octocat/Hello-World", "octocat", "Hello-World")]
#[case(
    "https://user:pass:extra@github.com/octocat/Hello-World",
    "octocat",
    "Hello-World"
)]
fn repo_urls(#[case] url: &str, #[case] owner: &str, #[case] name: &str) {}

#[template]
#[rstest]
#[case("https://github.com/none/headerparser.git")]
#[case("https://github.com/joe.coder/hello-world")]
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
#[case("api.github.com/REPOS/jwodder/headerparser")]
#[case("https://api.github.com/REPOS/jwodder/headerparser")]
#[case("https://user name@github.com/octocat/Hello-World")]
#[case("https://user/name@github.com/octocat/Hello-World")]
#[case("https://user@name@github.com/octocat/Hello-World")]
#[case("my.username@github.com/octocat/Hello-World")]
#[case("my.username@www.github.com/octocat/Hello-World")]
#[case("my.username:hunter2@github.com/octocat/Hello-World")]
#[case("my.username:hunter2@www.github.com/octocat/Hello-World")]
#[case("https://x-access-token:1234567890@api.github.com/repos/octocat/Hello-World")]
#[case("x-access-token:1234567890@github.com/octocat/Hello-World")]
#[case("git@github.com/jwodder/headerparser")]
#[case("git@GITHUB.com:joe-q-coder/my.repo.GIT")]
#[case("GIT@github.com:joe-q-coder/my.repo.git")]
#[case("git@github.com/joe-q-coder/my.repo.git")]
#[case("ssh://git@github.com:jwodder/headerparser")]
#[case("ssh://git@github.com:jwodder/headerparser/")]
#[case("ssh://git@github.com/jwodder/headerparser/")]
#[case("git://github.com/jwodder/headerparser/")]
#[case("SSH://Git@GITHUB.COM/-/test")]
#[case("ssh://GIT@github.com/-/test")]
#[case("https://http://github.com/joe-coder/hello.world")]
#[case("https://github.com/-Jerry-/geshi-1.0.Git")]
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

#[rstest]
#[case("jwodder/headerparser", "jwodder", "headerparser")]
#[case("jwodder/none", "jwodder", "none")]
#[case("nonely/headerparser", "nonely", "headerparser")]
#[case("none-none/headerparser", "none-none", "headerparser")]
#[case("nonenone/headerparser", "nonenone", "headerparser")]
fn test_try_from_string(#[case] spec: String, #[case] owner: &str, #[case] name: &str) {
    let r = GHRepo::try_from(spec).unwrap();
    assert_eq!(r.owner(), owner);
    assert_eq!(r.name(), name);
}

#[apply(bad_repos)]
fn test_from_bad_str(#[case] spec: &str) {
    match GHRepo::from_str(spec) {
        Err(ParseError::InvalidSpec(s)) if s == spec => (),
        e => panic!("Got wrong result: {e:?}"),
    }
}

#[apply(bad_repos)]
#[case("git://github.com/jwodder/headerparser")]
#[case("git://github.com/jwodder/headerparser.git")]
#[case("git@github.com:jwodder/headerparser")]
#[case("git@github.com:jwodder/headerparser.git")]
#[case("GIT://GitHub.COM/jwodder/headerparser")]
#[case("git@github.com:joe-q-coder/my.repo.git")]
#[case("git@GITHUB.com:joe-q-coder/my.repo.git")]
#[case("ssh://git@github.com/jwodder/headerparser")]
#[case("ssh://git@github.com/jwodder/headerparser.git")]
#[case("ssh://git@github.com/-/test")]
#[case("SSH://git@GITHUB.COM/-/test")]
#[case("https://api.github.com/repos/jwodder/headerparser")]
#[case("http://api.github.com/repos/jwodder/headerparser")]
#[case("api.github.com/repos/jwodder/headerparser")]
#[case("https://api.github.com/repos/none-/-none")]
#[case("HttpS://api.github.com/repos/none-/-none")]
#[case("http://api.github.com/repos/none-/-none")]
#[case("Http://api.github.com/repos/none-/-none")]
#[case("Api.GitHub.Com/repos/jwodder/headerparser")]
#[case("https://github.com/jwodder/headerparser")]
#[case("https://github.com/jwodder/headerparser.git")]
#[case("https://github.com/jwodder/headerparser.git/")]
#[case("https://github.com/jwodder/headerparser/")]
#[case("https://www.github.com/jwodder/headerparser")]
#[case("http://github.com/jwodder/headerparser")]
#[case("http://www.github.com/jwodder/headerparser")]
#[case("github.com/jwodder/headerparser")]
#[case("github.com/jwodder/headerparser.git")]
#[case("github.com/jwodder/headerparser.git/")]
#[case("github.com/jwodder/headerparser/")]
#[case("www.github.com/jwodder/headerparser")]
#[case("https://github.com/jwodder/none.git")]
#[case("https://github.com/joe-coder/hello.world")]
#[case("http://github.com/joe-coder/hello.world")]
#[case("HTTPS://GITHUB.COM/joe-coder/hello.world")]
#[case("HTTPS://WWW.GITHUB.COM/joe-coder/hello.world")]
#[case("https://github.com/-Jerry-/geshi-1.0.git")]
#[case("https://github.com/-Jerry-/geshi-1.0.git/")]
#[case("https://github.com/-Jerry-/geshi-1.0/")]
#[case("https://www.github.com/-Jerry-/geshi-1.0")]
#[case("github.com/-Jerry-/geshi-1.0")]
#[case("https://x-access-token:1234567890@github.com/octocat/Hello-World")]
#[case("https://my.username@github.com/octocat/Hello-World")]
#[case("https://user%20name@github.com/octocat/Hello-World")]
#[case("https://user+name@github.com/octocat/Hello-World")]
#[case("https://~user.name@github.com/octocat/Hello-World")]
#[case("https://@github.com/octocat/Hello-World")]
#[case("https://user:@github.com/octocat/Hello-World")]
#[case("https://:pass@github.com/octocat/Hello-World")]
#[case("https://:@github.com/octocat/Hello-World")]
#[case("https://user:pass:extra@github.com/octocat/Hello-World")]
fn test_try_from_bad_string(#[case] spec: String) {
    match GHRepo::try_from(spec.clone()) {
        Err(ParseError::InvalidSpec(s)) if s == spec => (),
        e => panic!("Got wrong result: {e:?}"),
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
        e => panic!("Got wrong result: {e:?}"),
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

#[rstest]
#[case("Zoctocat/hello-world", "octocat/hello-world")]
#[case("n/z", "octocat/hello-world")]
#[case("octoca-t/hello-world", "octocat/hello-world")]
#[case("octocat/Zello-world", "octocat/hello-world")]
#[case("octocat/hello-world", "octocat/repository")]
#[case("octocat/hello-world", "p/a")]
fn test_ord(#[case] lesser: &str, #[case] greater: &str) {
    let lesser_repo = GHRepo::from_str(lesser).unwrap();
    let greater_repo = GHRepo::from_str(greater).unwrap();
    assert!(lesser_repo < greater_repo);
    assert!(lesser_repo < greater);
}
