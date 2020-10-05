use crate::base;
use crate::data::{self, ObjectType, Oid};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Result, Write};
use std::process::exit;
use std::process::{Command, Stdio};

pub fn handle(matches: clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("init", Some(submatches)) => init(submatches),
        ("status", Some(submatches)) => status(submatches),
        ("k", Some(submatches)) => gitk(submatches),
        ("hash-file", Some(submatches)) => hash_file(submatches),
        ("cat-file", Some(submatches)) => cat_file(submatches),
        ("write-tree", Some(submatches)) => write_tree(submatches),
        ("read-tree", Some(submatches)) => read_tree(submatches),
        ("commit", Some(submatches)) => commit(submatches),
        ("log", Some(submatches)) => log(submatches),
        ("checkout", Some(submatches)) => checkout(submatches),
        ("reset", Some(submatches)) => reset(submatches),
        ("tag", Some(submatches)) => tag(submatches),
        ("branch", Some(submatches)) => branch(submatches),
        _ => {
            eprintln!("{}", matches.usage());
            exit(1);
        }
    }
}

fn init(_submatches: &clap::ArgMatches<'_>) -> Result<()> {
    base::init()
}

fn status(_submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let branch = base::get_branch_name()?;
    if let Some(branch) = branch {
        println!("On branch {}", branch);
    } else {
        let head = base::get_oid("HEAD")?;
        println!("HEAD detached at {}", &head[..10]);
    }
    Ok(())
}

fn gitk(_submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let mut dot_input: Vec<String> = Vec::new();
    let mut oids: HashSet<Oid> = HashSet::new();

    dot_input.push("digraph commits {".to_string());

    for (refname, refvalue) in data::iter_refs(None, false)? {
        if let Some(value) = refvalue.value {
            dot_input.push(format!("\"{}\" [shape=note]", refname));
            dot_input.push(format!("\"{}\" -> \"{}\"", refname, value));
            if !refvalue.symbolic {
                oids.insert(value);
            }
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

    // Construct a lookup from OID to refs which point to it in some way
    let mut refs: HashMap<String, Vec<String>> = HashMap::new();
    for (refname, refval) in data::iter_refs(None, true)? {
        if let Some(value) = refval.value {
            refs.entry(value).or_default().push(refname);
        }
    }

    for oid in base::iter_commits_and_parents([oid].iter().cloned())? {
        let ref_str = if let Some(oidrefs) = refs.get(&oid) {
            format!(" ({})", oidrefs.join(", "))
        } else {
            "".to_string()
        };
        let commit = base::get_commit(&oid)?;
        println!("commit {}{}", oid, ref_str);
        println!("    {}", commit.message);
        println!("");
    }
    Ok(())
}

fn checkout(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let name = submatches.value_of("COMMIT").unwrap();
    base::checkout(name)
}

fn reset(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let name = submatches.value_of("COMMIT").unwrap();
    let oid = base::get_oid(name)?;
    base::reset(oid)
}

fn tag(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let name = submatches.value_of("NAME").unwrap();
    let oid = base::get_oid(submatches.value_of("OID").unwrap())?;
    base::create_tag(name, &oid)
}

fn branch(submatches: &clap::ArgMatches<'_>) -> Result<()> {
    let name = submatches.value_of("NAME");
    if let Some(name) = name {
        let start = base::get_oid(submatches.value_of("START").unwrap())?;
        base::create_branch(name, &start)?;
        println!("Branch '{}' created at {}", name, &start[..10]);
    } else {
        let current = base::get_branch_name()?;
        for branch in base::iter_branch_names()? {
            let prefix = if Some(&branch) == current.as_ref() {
                "*"
            } else {
                " "
            };
            println!("{} {}", prefix, branch);
        }
    }
    Ok(())
}
