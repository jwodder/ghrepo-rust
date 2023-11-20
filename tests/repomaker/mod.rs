use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::{tempdir, TempDir};

pub struct RepoMaker {
    tmpdir: TempDir,
}

impl RepoMaker {
    pub fn new() -> Self {
        RepoMaker {
            tmpdir: tempdir().unwrap(),
        }
    }

    pub fn path(&self) -> &Path {
        self.tmpdir.path()
    }

    fn run<S: AsRef<OsStr>>(&self, args: &[S]) {
        let r = Command::new("git")
            .args(args)
            .current_dir(self.path())
            .status()
            .unwrap();
        assert!(r.success());
    }

    pub fn init(&self, branch: &str) {
        self.run(&["-c", &format!("init.defaultBranch={branch}"), "init"])
    }

    pub fn add_remote<S: AsRef<OsStr>>(&self, remote: &str, url: S) {
        self.run(&[
            "remote".as_ref(),
            "add".as_ref(),
            remote.as_ref(),
            url.as_ref(),
        ])
    }

    // Used by local_repo.rs but not cli.rs
    #[allow(unused)]
    pub fn set_upstream(&self, branch: &str, remote: &str) {
        self.run(&["config", &format!("branch.{branch}.remote"), remote])
    }

    // Used by local_repo.rs but not cli.rs
    #[allow(unused)]
    pub fn detach(&self) {
        fs::write(self.path().join("file.txt"), b"This is test text\n").unwrap();
        self.run(&["add", "file.txt"]);
        self.run(&["commit", "-m", "Add a file"]);
        fs::write(self.path().join("file2.txt"), b"This is also text\n").unwrap();
        self.run(&["add", "file2.txt"]);
        self.run(&["commit", "-m", "Add another file"]);
        self.run(&["checkout", "HEAD^"]);
    }
}
