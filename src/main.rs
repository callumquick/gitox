#[macro_use]
extern crate clap;
use clap::{Arg, SubCommand};
use std::fs;
use std::process::exit;

mod data;

fn main() -> std::io::Result<()> {
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
            .about("Hash a file into a stored object")
            .arg(Arg::with_name("FILE").help("File to hash").required(true)),
    )
    .subcommand(
        SubCommand::with_name("cat-file")
            .about("Retrieve a stored object file")
            .arg(
                Arg::with_name("OBJECT")
                    .help("Object to retrieve")
                    .required(true),
            ),
    )
    .get_matches();

    match matches.subcommand() {
        ("init", _) => data::init(),
        ("hash-file", Some(submatches)) => {
            let oid = data::hash_object(&fs::read(submatches.value_of("FILE").unwrap())?)?;
            println!("{}", oid);
            Ok(())
        }
        ("cat-file", Some(submatches)) => {
            let contents = data::get_object(submatches.value_of("OBJECT").unwrap())?;
            print!("{}", String::from_utf8_lossy(&contents));
            Ok(())
        }
        _ => {
            eprintln!("{}", matches.usage());
            exit(1);
        }
    }
}
