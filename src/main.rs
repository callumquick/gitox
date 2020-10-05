#[macro_use]
extern crate clap;
use clap::{Arg, SubCommand};

mod base;
mod cli;
mod data;

fn main() -> std::io::Result<()> {
    let matches = clap_app!(gitox =>
        (version: "0.1.0")
        (author: "Callum Ward <wards.callum@gmail.com")
        (about: "Git clone written in Rust for education")
        (@subcommand init =>
            (about: "Initialize the repository")
        )
        (@subcommand status =>
            (about: "Get repository status")
        )
        (@subcommand k =>
            (about: "Visualize the repository")
        )
        (@subcommand commit =>
            (about: "Record changes to the repository")
            (@arg message: -m <MESSAGE> "Message to record")
        )
        (@subcommand log =>
            (about: "Show commit logs")
            (@arg OID: default_value[HEAD] "Commit object to show the log for")
        )
        (@subcommand show =>
            (about: "Show commit object")
            (@arg OID: default_value[HEAD] "Commit object to show")
        )
        (@subcommand checkout =>
            (about: "Switch branches or restore working tree files")
            (@arg COMMIT: default_value[HEAD] "Commit or branch to checkout")
        )
        (@subcommand reset =>
            (about: "Reset working directory to commit")
            (@arg COMMIT: default_value[HEAD] "Commit to reset to")
        )
        (@subcommand tag =>
            (about: "Create tag object referencing a commit")
            (@arg NAME: +required "Tag name")
            (@arg OID: default_value[HEAD] "Commit to tag")
        )
        (@subcommand branch =>
            (about: "Create a new branch or show current branches")
            (@arg NAME: !required "Branch to create")
            (@arg START: default_value[HEAD] "Start the branch at a given commit")
        )
    )
    // Some subcommands cannot be implemented using the macro syntax because
    // they contain hyphens in the name or need to use other macro reserved
    // characters
    .subcommand(
        SubCommand::with_name("hash-file")
            .about("Hash a file into a stored object")
            .arg(Arg::with_name("FILE").help("File to hash").required(true)),
    )
    .subcommand(
        SubCommand::with_name("cat-file")
            .about("Retrieve a stored object file")
            .arg(
                Arg::with_name("OID")
                    .help("Object to retrieve")
                    .required(true),
            ),
    )
    .subcommand(
        SubCommand::with_name("write-tree")
            .about("Write current working directory to the object store"),
    )
    .subcommand(
        SubCommand::with_name("read-tree")
            .about("Extract tree object into the working directory")
            .arg(
                Arg::with_name("OID")
                    .help("Tree object to extract")
                    .required(true),
            ),
    )
    .get_matches();

    cli::handle(matches)
}
