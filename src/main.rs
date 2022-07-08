#![warn(warnings)]

use clap::Parser;
use ghrepo::{LocalRepo, LocalRepoError};
use serde_json::json;
use std::process::exit;

/// Show current GitHub repository
#[derive(Debug, Parser)]
#[clap(version)]
struct Arguments {
    /// Output JSON
    #[clap(short = 'J', long)]
    json: bool,

    /// Parse the GitHub URL from the given remote
    #[clap(short, long, default_value = "origin")]
    remote: String,

    /// Path to a clone of a GitHub repo  [default: current directory]
    dirpath: Option<String>,
}

fn main() {
    let args = Arguments::parse();
    match run(&args) {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("ghrepo: {}", e);
            exit(1);
        }
    }
}

fn run(args: &Arguments) -> Result<String, LocalRepoError> {
    let lr = match &args.dirpath {
        Some(p) => LocalRepo::new(&p),
        None => LocalRepo::for_cwd()?,
    };
    let gr = lr.github_remote(&args.remote)?;
    if args.json {
        let data = json!({
            "owner": gr.owner(),
            "name": gr.name(),
            "fullname": gr.to_string(),
            "api_url": gr.api_url(),
            "clone_url": gr.clone_url(),
            "git_url": gr.git_url(),
            "html_url": gr.html_url(),
            "ssh_url": gr.ssh_url(),
        });
        Ok(serde_json::to_string_pretty(&data).unwrap())
    } else {
        Ok(gr.to_string())
    }
}
