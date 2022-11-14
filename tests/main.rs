// Use "pub" to silence some "unused code" warnings
pub mod repomaker;

use ghrepo::GHRepo;
use repomaker::RepoMaker;
use std::process::Command;
use std::str;
use tempfile::tempdir;
use which::which;

#[test]
fn test_run() {
    if which("git").is_err() {
        return;
    }
    let repo = GHRepo::new("octocat", "repository").unwrap();
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("origin", &repo.ssh_url());
    let out = readcmd(&[maker.path().to_str().unwrap()]);
    assert_eq!(out, "octocat/repository");
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
    let out = readcmd(&["--json", maker.path().to_str().unwrap()]);
    assert_eq!(out, expected);
}

#[test]
fn test_run_empty() {
    if which("git").is_err() {
        return;
    }
    let tmp_path = tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ghrepo"))
        .arg(tmp_path.path())
        .output()
        .unwrap();
    assert_eq!(str::from_utf8(&out.stdout).unwrap(), "");
    assert_eq!(
        str::from_utf8(&out.stderr).unwrap().trim(),
        "fatal: not a git repository (or any of the parent directories): .git"
    );
    assert!(!out.status.success());
}

#[test]
fn test_run_no_such_remote() {
    if which("git").is_err() {
        return;
    }
    let maker = RepoMaker::new();
    maker.init("trunk");
    let out = Command::new(env!("CARGO_BIN_EXE_ghrepo"))
        .arg(maker.path())
        .output()
        .unwrap();
    assert_eq!(str::from_utf8(&out.stdout).unwrap(), "");
    assert_eq!(
        str::from_utf8(&out.stderr).unwrap().trim(),
        "error: No such remote 'origin'"
    );
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn test_run_invalid_url() {
    if which("git").is_err() {
        return;
    }
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("upstream", "https://git.example.com/repo.git");
    let out = Command::new(env!("CARGO_BIN_EXE_ghrepo"))
        .arg("-rupstream")
        .arg(maker.path())
        .output()
        .unwrap();
    assert_eq!(str::from_utf8(&out.stdout).unwrap(), "");
    assert_eq!(
        str::from_utf8(&out.stderr).unwrap().trim(),
        concat!(
            "ghrepo: Repository remote URL is not a GitHub URL:",
            " Invalid GitHub repository spec:",
            " \"https://git.example.com/repo.git\"",
        ),
    );
    assert_eq!(out.status.code(), Some(1));
}

fn readcmd(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_ghrepo"))
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success());
    return str::from_utf8(&out.stdout).unwrap().trim().to_string();
}
