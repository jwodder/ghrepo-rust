#![cfg(feature = "cli")]
// Use "pub" to silence some "unused code" warnings
pub mod repomaker;

use assert_cmd::Command;
use ghrepo::GHRepo;
use repomaker::RepoMaker;
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
    maker.add_remote("origin", repo.ssh_url());
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg(maker.path())
        .assert()
        .success()
        .stdout("octocat/repository\n");
}

#[test]
fn test_run_noarg() {
    if which("git").is_err() {
        return;
    }
    let repo = GHRepo::new("octocat", "repository").unwrap();
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("origin", repo.ssh_url());
    Command::cargo_bin("ghrepo")
        .unwrap()
        .current_dir(maker.path())
        .assert()
        .success()
        .stdout("octocat/repository\n");
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
}\n";
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("origin", repo.ssh_url());
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg("--json")
        .arg(maker.path())
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn test_run_json_noarg() {
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
}\n";
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("origin", repo.ssh_url());
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg("--json")
        .current_dir(maker.path())
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn test_run_remote() {
    if which("git").is_err() {
        return;
    }
    let origin = GHRepo::new("octocat", "repository").unwrap();
    let upstream = GHRepo::new("sourcedog", "repository").unwrap();
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("origin", origin.ssh_url());
    maker.add_remote("upstream", upstream.clone_url());
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg("--remote")
        .arg("upstream")
        .arg(maker.path())
        .assert()
        .success()
        .stdout("sourcedog/repository\n");
}

#[test]
fn test_run_remote_noarg() {
    if which("git").is_err() {
        return;
    }
    let origin = GHRepo::new("octocat", "repository").unwrap();
    let upstream = GHRepo::new("sourcedog", "repository").unwrap();
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("origin", origin.ssh_url());
    maker.add_remote("upstream", upstream.clone_url());
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg("--remote")
        .arg("upstream")
        .current_dir(maker.path())
        .assert()
        .success()
        .stdout("sourcedog/repository\n");
}

#[test]
fn test_run_empty() {
    if which("git").is_err() {
        return;
    }
    let tmp_path = tempdir().unwrap();
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg(tmp_path.path())
        .assert()
        .failure()
        .stdout("")
        .stderr("fatal: not a git repository (or any of the parent directories): .git\n");
}

#[test]
fn test_run_no_such_remote() {
    if which("git").is_err() {
        return;
    }
    let maker = RepoMaker::new();
    maker.init("trunk");
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg(maker.path())
        .assert()
        .failure()
        .stdout("")
        .stderr("error: No such remote 'origin'\n");
}

#[test]
fn test_run_invalid_url() {
    if which("git").is_err() {
        return;
    }
    let maker = RepoMaker::new();
    maker.init("trunk");
    maker.add_remote("upstream", "https://git.example.com/repo.git");
    Command::cargo_bin("ghrepo")
        .unwrap()
        .arg("-rupstream")
        .arg(maker.path())
        .assert()
        .code(1)
        .stdout("")
        .stderr(concat!(
            "ghrepo: repository remote URL is not a GitHub URL:",
            " invalid GitHub repository spec:",
            " \"https://git.example.com/repo.git\"\n",
        ));
}
