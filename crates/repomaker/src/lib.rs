use std::ffi::OsStr;
use std::fs;
use std::io::Result;
use std::path::Path;
use std::process::Command;
use tempfile::{tempdir, TempDir};

#[derive(Debug)]
pub struct RepoMaker {
    tmpdir: TempDir,
}

impl RepoMaker {
    pub fn new() -> Result<Self> {
        Ok(RepoMaker { tmpdir: tempdir()? })
    }

    pub fn path(&self) -> &Path {
        self.tmpdir.path()
    }

    fn run<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(&self, args: I) -> Result<()> {
        let r = Command::new("git")
            .args(args)
            .current_dir(self.path())
            .status()?;
        assert!(r.success(), "git command should have succeded");
        Ok(())
    }

    pub fn init(&self, branch: &str) -> Result<()> {
        self.run(["-c", &format!("init.defaultBranch={branch}"), "init"])
    }

    pub fn add_remote<S: AsRef<OsStr>>(&self, remote: &str, url: S) -> Result<()> {
        self.run([
            "remote".as_ref(),
            "add".as_ref(),
            remote.as_ref(),
            url.as_ref(),
        ])
    }

    pub fn set_upstream(&self, branch: &str, remote: &str) -> Result<()> {
        self.run(["config", &format!("branch.{branch}.remote"), remote])
    }

    pub fn detach(&self) -> Result<()> {
        fs::write(self.path().join("file.txt"), b"This is test text\n")?;
        self.run(["add", "file.txt"])?;
        self.run(["commit", "-m", "Add a file"])?;
        fs::write(self.path().join("file2.txt"), b"This is also text\n")?;
        self.run(["add", "file2.txt"])?;
        self.run(["commit", "-m", "Add another file"])?;
        self.run(["checkout", "HEAD^"])?;
        Ok(())
    }
}
