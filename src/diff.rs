use crate::base::Tree;
use crate::data::Oid;
use std::collections::HashMap;
use std::io::Result;
use std::path::PathBuf;

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

pub fn diff_trees(t_from: Tree, t_to: Tree) -> Result<String> {
    let mut output = String::new();
    for (path, objects) in compare_trees(&[t_from, t_to])? {
        if objects.get(0) != objects.get(1) {
            output.push_str(&format!("changed: {}\n", path.to_str().unwrap()));
        }
    }
    Ok(output)
}
