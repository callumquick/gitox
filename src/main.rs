#[macro_use]
extern crate clap;

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
    .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        data::init();
    }
}
