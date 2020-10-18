use crate::base::Tree;
use crate::data::{self, ObjectType, Oid};
use std::collections::HashMap;
use std::io::{Result, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile;

fn get_none_vector<T>(len: usize) -> Vec<Option<T>> {
    let mut empty = Vec::new();
    for _ in 0..len {
        empty.push(None);
    }
    empty
}

pub fn compare_trees(trees: &[Tree]) -> Result<impl Iterator<Item = (PathBuf, Vec<Option<Oid>>)>> {
    let mut entries: HashMap<PathBuf, Vec<Option<Oid>>> = HashMap::new();

    for (i, tree) in trees.iter().enumerate() {
        for (path, oid) in tree.iter() {
            let oids = entries
                .entry(path.to_path_buf())
                .or_insert_with(|| get_none_vector(trees.len()));
            oids[i] = Some(oid.to_string());
        }
    }

    Ok(entries.into_iter())
}

pub fn iter_changed_files(
    t_from: Tree,
    t_to: Tree,
) -> Result<impl Iterator<Item = (PathBuf, String)>> {
    let mut output = Vec::new();
    for (path, objects) in compare_trees(&[t_from, t_to])? {
        let o_from = objects.get(0).unwrap();
        let o_to = objects.get(1).unwrap();
        if o_from != o_to {
            let action = if o_from.is_none() {
                "new file".to_string()
            } else if o_to.is_none() {
                "deleted".to_string()
            } else {
                "modified".to_string()
            };
            output.push((path, action));
        }
    }
    Ok(output.into_iter())
}

pub fn diff_trees(t_from: Tree, t_to: Tree) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    for (path, objects) in compare_trees(&[t_from, t_to])? {
        let o_from = objects.get(0).unwrap();
        let o_to = objects.get(1).unwrap();
        if o_from != o_to {
            output.append(&mut diff_blobs(o_from.as_ref(), o_to.as_ref(), Some(path))?);
        }
    }
    Ok(output)
}

pub fn diff_blobs(
    o_from: Option<&Oid>,
    o_to: Option<&Oid>,
    path: Option<PathBuf>,
) -> Result<Vec<u8>> {
    let mut f_from = tempfile::NamedTempFile::new()?;
    let mut f_to = tempfile::NamedTempFile::new()?;
    let path = path
        .map(|buf| buf.to_string_lossy().into_owned())
        .unwrap_or("blob".to_string());

    if o_from.is_some() {
        f_from.write_all(&data::get_object(o_from.unwrap(), Some(ObjectType::Blob))?.contents)?;
    }
    if o_to.is_some() {
        f_to.write_all(&data::get_object(o_to.unwrap(), Some(ObjectType::Blob))?.contents)?;
    }

    let output = Command::new("diff")
        .arg("--unified")
        .arg("--show-c-function")
        .arg("--label")
        .arg(format!("a/{}", path))
        .arg(f_from.path())
        .arg("--label")
        .arg(format!("b/{}", path))
        .arg(f_to.path())
        .stderr(Stdio::null())
        .output()?;

    Ok(output.stdout)
}
