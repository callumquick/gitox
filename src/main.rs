#[macro_use]
extern crate clap;
use clap::{Arg, SubCommand};

mod data;

fn main() {
    let matches = clap_app!(gitox =>
        (version: "0.1.0")
        (author: "Callum Ward <wards.callum@gmail.com")
        (about: "Git clone written in Rust for education")
        (@subcommand init =>
            (about: "Initialize the repository")
        )
    )
    // Some subcommands cannot be implemented using the macro syntax because
    // they contain hyphens in the name
    .subcommand(
        SubCommand::with_name("hash-file")
            .help("Hash a file into a stored object")
            .arg(Arg::with_name("FILE").help("File to hash").required(true)),
    )
    .get_matches();

    match matches.subcommand() {
        ("init", _) => data::init(),
        ("hash-file", Some(submatches)) => {
            println!("Gonna hash {}", submatches.value_of("FILE").unwrap());
        }
        _ => println!("{}", matches.usage()),
    }
}
