use clap::Parser;
use ghrepo::{LocalRepo, LocalRepoError};
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
        Err(LocalRepoError::CommandFailed(rc)) => exit(rc.code().unwrap_or(1)),
        Err(LocalRepoError::NoSuchRemote(_)) => exit(2),
        Err(e) => {
            eprintln!("ghrepo: {}", e);
            exit(1);
        }
    }
}

fn run(args: &Arguments) -> Result<String, LocalRepoError> {
    let lr = match &args.dirpath {
        Some(p) => LocalRepo::new(p),
        None => LocalRepo::for_cwd()?,
    };
    let gr = lr.github_remote(&args.remote)?;
    if args.json {
        // The various values here all consist entirely of printable ASCII
        // characters (as long as GitHub owner & repo names continue to only
        // contain printable ASCII characters), so we don't need any special
        // JSON processing for escapes.
        Ok(format!(
            concat!(
                "{{\n",
                "    \"owner\": \"{}\",\n",
                "    \"name\": \"{}\",\n",
                "    \"fullname\": \"{}\",\n",
                "    \"api_url\": \"{}\",\n",
                "    \"clone_url\": \"{}\",\n",
                "    \"git_url\": \"{}\",\n",
                "    \"html_url\": \"{}\",\n",
                "    \"ssh_url\": \"{}\"\n",
                "}}"
            ),
            gr.owner(),
            gr.name(),
            gr.to_string(),
            gr.api_url(),
            gr.clone_url(),
            gr.git_url(),
            gr.html_url(),
            gr.ssh_url()
        ))
    } else {
        Ok(gr.to_string())
    }
}
