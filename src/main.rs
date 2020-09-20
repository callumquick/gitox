use clap::{App, SubCommand};

mod data;

fn main() {
    let matches = App::new("gitox")
        .version("1.0")
        .author("Callum Ward <wards.callum@gmail.com")
        .about("Git clone written in Rust for education")
        .subcommand(SubCommand::with_name("init").about("Initialize the repository"))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        data::init();
    }
}
