use crate::base::Tree;
use crate::data::{self, ObjectType, Oid};
use std::collections::HashMap;
use std::io::{Result, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile;

pub fn compare_trees(trees: &[Tree]) -> Result<impl Iterator<Item = (PathBuf, Vec<Oid>)>> {
    let mut entries: HashMap<PathBuf, Vec<Oid>> = HashMap::new();

    for tree in trees {
        for (path, oid) in tree.iter() {
            // TODO Maybe need to pad the Vec with None where the tree does not
            // contain a certain path?
            entries
                .entry(path.to_path_buf())
                .or_default()
                .push(oid.to_string());
        }
    }

    Ok(entries.into_iter())
}

pub fn diff_trees(t_from: Tree, t_to: Tree) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    for (path, objects) in compare_trees(&[t_from, t_to])? {
        let o_from = objects.get(0);
        let o_to = objects.get(1);
        if o_from != o_to {
            output.append(&mut diff_blobs(o_from, o_to, Some(path))?);
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
        .map(|buf| buf.to_str().unwrap().to_string())
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
