mod repomaker;
use ghrepo::{GHRepo, LocalRepo, LocalRepoError};
use repomaker::RepoMaker;
use std::env;
use tempfile::tempdir;
use which::which;

#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

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
        e => panic!("Git command did not fail; got: {e:?}"),
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
        e => panic!("Got wrong result: {e:?}"),
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
        e => panic!("Got wrong result: {e:?}"),
    }
}

#[test]
fn test_github_remote_empty() {
    if which("git").is_err() {
        return;
    }
    let tmp_path = tempdir().unwrap();
    let lr = LocalRepo::new(tmp_path.path());
    #[cfg(windows)]
    let expected_err = "Git command exited unsuccessfully: exit code: 128";
    #[cfg(not(windows))]
    let expected_err = "Git command exited unsuccessfully: exit status: 128";
    match lr.github_remote("origin") {
        Err(e @ LocalRepoError::CommandFailed(_)) => {
            assert_eq!(e.to_string(), expected_err);
        }
        e => panic!("Git command did not fail; got: {e:?}"),
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
        e => panic!("Git command did not fail; got: {e:?}"),
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
    maker.add_remote("origin", repo.ssh_url());
    let lr = LocalRepo::new(maker.path());
    match lr.github_remote("origin") {
        Ok(lr) if lr == repo => (),
        e => panic!("Got wrong result: {e:?}"),
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
        e => panic!("Got wrong result: {e:?}"),
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
            format!("failed to decode output from Git command: {eu}")
        ),
        e => panic!("Got wrong result: {e:?}"),
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
        e => panic!("Got wrong result: {e:?}"),
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
    maker.add_remote("github", repo.clone_url());
    maker.set_upstream("trunk", "github");
    let lr = LocalRepo::new(maker.path());
    match lr.branch_upstream("trunk") {
        Ok(r) if r == repo => (),
        e => panic!("Got wrong result: {e:?}"),
    }
}
