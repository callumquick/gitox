use crate::base;
use crate::data;
use crate::data::ObjectType;
use std::fs;
use std::process::exit;

pub fn handle(matches: clap::ArgMatches) -> std::io::Result<()> {
    match matches.subcommand() {
        ("init", Some(submatches)) => init(submatches),
        ("hash-file", Some(submatches)) => hash_file(submatches),
        ("cat-file", Some(submatches)) => cat_file(submatches),
        ("write-tree", Some(submatches)) => write_tree(submatches),
        _ => {
            eprintln!("{}", matches.usage());
            exit(1);
        }
    }
}

fn init(_submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    data::init()
}

fn hash_file(submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    let oid = data::hash_object(
        &fs::read(submatches.value_of("FILE").unwrap())?,
        ObjectType::Blob,
    )?;
    println!("{}", oid);
    Ok(())
}

fn cat_file(submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    let object = data::get_object(submatches.value_of("OBJECT").unwrap(), None)?;
    print!("{}", String::from_utf8_lossy(&object.contents));
    Ok(())
}

fn write_tree(_submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    println!("{}", base::write_tree(".")?);
    Ok(())
}
