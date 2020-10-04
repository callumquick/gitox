use crate::base;
use crate::data::{self, ObjectType, Oid};
use std::collections::HashSet;
use std::fs;
use std::io::{Result, Write};
use std::process::exit;
use std::process::{Command, Stdio};

pub fn handle(matches: clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("init", Some(submatches)) => init(submatches),
        ("k", Some(submatches)) => gitk(submatches),
        ("hash-file", Some(submatches)) => hash_file(submatches),
        ("cat-file", Some(submatches)) => cat_file(submatches),
        ("write-tree", Some(submatches)) => write_tree(submatches),
        ("read-tree", Some(submatches)) => read_tree(submatches),
        ("commit", Some(submatches)) => commit(submatches),
        ("log", Some(submatches)) => log(submatches),
        ("checkout", Some(submatches)) => checkout(submatches),
        ("tag", Some(submatches)) => tag(submatches),
        ("branch", Some(submatches)) => branch(submatches),
        _ => {
            eprintln!("{}", matches.usage());
            exit(1);
        }
    }
}

fn init(_submatches: &clap::ArgMatches<'_>) -> Result<()> {
    data::init()
}

fn gitk(_submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let mut dot_input: Vec<String> = Vec::new();
    let mut oids: HashSet<Oid> = HashSet::new();

    dot_input.push("digraph commits {".to_string());

    for (ref_, oid) in data::iter_refs()? {
        if let Some(oid) = oid {
            dot_input.push(format!("\"{}\" [shape=note]", ref_));
            dot_input.push(format!("\"{}\" -> \"{}\"", ref_, oid));
            oids.insert(oid);
        }
    }

    for oid in base::iter_commits_and_parents(oids.into_iter())? {
        let commit = base::get_commit(&oid)?;
        dot_input.push(format!(
            "\"{}\" [shape=box style=filled label=\"{}\"]",
            oid,
            &oid[..10]
        ));
        if let Some(parent) = commit.parent {
            dot_input.push(format!("\"{}\" -> \"{}\"", oid, parent));
        }
    }

    dot_input.push("}".to_string());

    let proc = Command::new("dot")
        .arg("-Tgtk")
        .arg("/dev/stdin")
        .stdin(Stdio::piped())
        .spawn()?;
    proc.stdin
        .expect("'dot' did not wait to read stdin")
        .write(dot_input.join("\n").as_bytes())?;
    Ok(())
}

fn hash_file(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let oid = data::hash_object(
        &fs::read(submatches.value_of("FILE").unwrap())?,
        ObjectType::Blob,
    )?;
    println!("{}", oid);
    Ok(())
}

fn cat_file(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let oid = &base::get_oid(submatches.value_of("OID").unwrap())?;
    let object = data::get_object(&oid, None)?;
    print!("{}", String::from_utf8_lossy(&object.contents));
    Ok(())
}

fn write_tree(_submatches: &clap::ArgMatches<'_>) -> Result<()> {
    println!("{}", base::write_tree(".")?);
    Ok(())
}

fn read_tree(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    base::read_tree(&base::get_oid(submatches.value_of("OID").unwrap())?)
}

fn commit(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let message = submatches.value_of("message").unwrap();
    println!("{}", base::commit(message)?);
    Ok(())
}

fn log(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let oid = base::get_oid(submatches.value_of("OID").unwrap())?;
    for oid in base::iter_commits_and_parents([oid].iter().cloned())? {
        let commit = base::get_commit(&oid)?;

        println!("commit {}", oid);
        println!("    {}", commit.message);
        println!("");
    }
    Ok(())
}

fn checkout(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let oid = base::get_oid(submatches.value_of("OID").unwrap())?;
    let commit = base::get_commit(&oid)?;
    base::read_tree(&commit.tree)?;
    data::update_ref("HEAD", &oid)
}

fn tag(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let name = submatches.value_of("NAME").unwrap();
    let oid = base::get_oid(submatches.value_of("OID").unwrap())?;
    base::create_tag(name, &oid)
}

fn branch(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let name = submatches.value_of("NAME").unwrap();
    let start = base::get_oid(submatches.value_of("START").unwrap())?;
    base::create_branch(name, &start)?;
    println!("Branch '{}' created at {}", name, &start[..10]);
    Ok(())
}
