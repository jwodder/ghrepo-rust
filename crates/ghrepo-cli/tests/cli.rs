use assert_cmd::cargo::cargo_bin_cmd;
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    maker.add_remote("origin", repo.ssh_url()).unwrap();
    cargo_bin_cmd!("ghrepo")
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    maker.add_remote("origin", repo.ssh_url()).unwrap();
    cargo_bin_cmd!("ghrepo")
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    maker.add_remote("origin", repo.ssh_url()).unwrap();
    cargo_bin_cmd!("ghrepo")
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    maker.add_remote("origin", repo.ssh_url()).unwrap();
    cargo_bin_cmd!("ghrepo")
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    maker.add_remote("origin", origin.ssh_url()).unwrap();
    maker.add_remote("upstream", upstream.clone_url()).unwrap();
    cargo_bin_cmd!("ghrepo")
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    maker.add_remote("origin", origin.ssh_url()).unwrap();
    maker.add_remote("upstream", upstream.clone_url()).unwrap();
    cargo_bin_cmd!("ghrepo")
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
    cargo_bin_cmd!("ghrepo")
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    cargo_bin_cmd!("ghrepo")
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
    let maker = RepoMaker::new().unwrap();
    maker.init("trunk").unwrap();
    maker
        .add_remote("upstream", "https://git.example.com/repo.git")
        .unwrap();
    cargo_bin_cmd!("ghrepo")
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
