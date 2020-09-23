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
        ("read-tree", Some(submatches)) => read_tree(submatches),
        ("commit", Some(submatches)) => commit(submatches),
        ("log", Some(submatches)) => log(submatches),
        ("checkout", Some(submatches)) => checkout(submatches),
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
    let object = data::get_object(&submatches.value_of("OID").unwrap().to_string(), None)?;
    print!("{}", String::from_utf8_lossy(&object.contents));
    Ok(())
}

fn write_tree(_submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    println!("{}", base::write_tree(".")?);
    Ok(())
}

fn read_tree(submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    base::read_tree(&submatches.value_of("OID").unwrap().to_string())
}

fn commit(submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    let message = submatches.value_of("message").unwrap();
    println!("{}", base::commit(message)?);
    Ok(())
}

fn log(submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    let mut parent = match submatches.value_of("OID") {
        Some(oid) => Some(oid.to_string()),
        None => data::get_head()?,
    };
    while let Some(oid) = parent {
        let commit = base::get_commit(&oid)?;

        println!("commit {}", oid);
        println!("    {}", commit.message);
        println!("");

        parent = commit.parent;
    }
    Ok(())
}

fn checkout(submatches: &clap::ArgMatches<'_>) -> std::io::Result<()> {
    let oid = submatches.value_of("OID").unwrap().to_string();
    let commit = base::get_commit(&oid)?;
    base::read_tree(&commit.tree)?;
    data::set_head(&oid)
}
