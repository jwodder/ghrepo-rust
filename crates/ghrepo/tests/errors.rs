use ghrepo::{LocalRepoError, ParseError};
use std::io::{Error, ErrorKind};

#[test]
fn test_display_parse_error_invalid_spec() {
    let e = ParseError::InvalidSpec("foo.bar".to_string());
    assert_eq!(e.to_string(), "invalid GitHub repository spec: \"foo.bar\"");
}

#[test]
fn test_display_parse_error_invalid_owner() {
    let e = ParseError::InvalidOwner("foo.bar".to_string());
    assert_eq!(
        e.to_string(),
        "invalid GitHub repository owner: \"foo.bar\""
    );
}

#[test]
fn test_display_parse_error_invalid_name() {
    let e = ParseError::InvalidName("foo.git".to_string());
    assert_eq!(e.to_string(), "invalid GitHub repository name: \"foo.git\"");
}

#[test]
fn test_display_local_repo_error_could_not_execute() {
    let e = LocalRepoError::CouldNotExecute(Error::from(ErrorKind::NotFound));
    assert_eq!(
        e.to_string(),
        format!(
            "failed to execute Git command: {}",
            Error::from(ErrorKind::NotFound)
        )
    );
}

#[test]
fn test_display_local_repo_error_curdir() {
    let e = LocalRepoError::CurdirError(Error::from(ErrorKind::NotFound));
    assert_eq!(
        e.to_string(),
        format!(
            "could not determine current directory: {}",
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
    assert_eq!(
        e.to_string(),
        "no such remote in Git repository: \"origin\""
    );
}

#[test]
fn test_display_local_repo_error_no_upstream() {
    let e = LocalRepoError::NoUpstream("main".to_string());
    assert_eq!(
        e.to_string(),
        "no upstream remote configured for Git branch: \"main\""
    );
}

#[test]
fn test_display_local_repo_error_parse_error() {
    let e = LocalRepoError::InvalidRemoteURL(ParseError::InvalidSpec("foo.bar".to_string()));
    assert_eq!(
        e.to_string(),
        "repository remote URL is not a GitHub URL: invalid GitHub repository spec: \"foo.bar\""
    );
}
