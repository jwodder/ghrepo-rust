use clap::Parser;
use ghrepo::{run, Arguments};
use std::process::exit;

fn main() {
    let args = Arguments::parse();
    match run(args) {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("ghrepo: {}", e);
            exit(1);
        }
    }
}
