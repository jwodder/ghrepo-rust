use ghrepo::{LocalRepo, LocalRepoError};
use lexopt::{Arg, Parser};
use std::path::PathBuf;
use std::process::exit;

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Run {
        json: bool,
        remote: String,
        dirpath: Option<PathBuf>,
    },
    Help,
    Version,
}

impl Command {
    fn from_parser(mut parser: Parser) -> Result<Command, lexopt::Error> {
        let mut json = false;
        let mut remote = String::from("origin");
        let mut dirpath: Option<PathBuf> = None;
        while let Some(arg) = parser.next()? {
            match arg {
                Arg::Short('J') | Arg::Long("json") => {
                    json = true;
                }
                Arg::Short('r') | Arg::Long("remote") => {
                    remote = parser.value()?.into_string()?;
                }
                Arg::Short('h') | Arg::Long("help") => return Ok(Command::Help),
                Arg::Short('V') | Arg::Long("version") => return Ok(Command::Version),
                Arg::Value(val) if dirpath.is_none() => {
                    dirpath = Some(val.into());
                }
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(Command::Run {
            json,
            remote,
            dirpath,
        })
    }

    fn run(self) {
        match self {
            Command::Help => {
                println!(
                    "Usage: {} [-J|--json] [-r|--remote <REMOTE>] [<REPO PATH>]",
                    env!("CARGO_BIN_NAME")
                );
                println!();
                println!("Show current GitHub repository");
                println!();
                println!("Visit <https://github.com/jwodder/ghrepo-rust> for more information.");
                println!();
                println!("Options:");
                println!("  -J, --json        Output JSON");
                println!("  -r <REMOTE>, --remote <REMOTE>");
                println!("                    Parse the GitHub URL from the given remote [default: origin]");
                println!("  -h, --help        Display this help message and exit");
                println!("  -V, --version     Show the program version and exit");
            }
            Command::Version => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            }
            Command::Run {
                json,
                remote,
                dirpath,
            } => match run(dirpath, json, remote) {
                Ok(s) => println!("{s}"),
                Err(LocalRepoError::CommandFailed(rc)) => exit(rc.code().unwrap_or(1)),
                Err(LocalRepoError::NoSuchRemote(_)) => exit(2),
                Err(e) => {
                    eprintln!("ghrepo: {e}");
                    exit(1);
                }
            },
        }
    }
}

fn main() -> Result<(), lexopt::Error> {
    Command::from_parser(Parser::from_env())?.run();
    Ok(())
}

fn run(dirpath: Option<PathBuf>, json: bool, remote: String) -> Result<String, LocalRepoError> {
    let lr = match dirpath {
        Some(p) => LocalRepo::new(p),
        None => LocalRepo::for_cwd()?,
    };
    let gr = lr.github_remote(&remote)?;
    if json {
        // The various values here all consist entirely of printable ASCII
        // characters, excluding double-quote and backslash (as long as GitHub
        // owner & repo names continue to contain only those characters), so we
        // don't need any special JSON processing for escapes.
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
            gr,
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
