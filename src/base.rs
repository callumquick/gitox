use crate::data::{self, ObjectType};
use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::path::{Component, Path, PathBuf};

fn is_ignored(path: &Path) -> bool {
    for component in path.components() {
        if let Component::Normal(segment) = component {
            if segment == data::GIT_DIR {
                return true;
            }
        }
    }
    false
}

fn write_tree_entry(dir_entry: DirEntry) -> std::io::Result<String> {
    let path = dir_entry.path();
    let filename = dir_entry.file_name().into_string().unwrap();
    return if path.is_dir() {
        Ok(format!(
            "{} {} {}",
            data::get_type_string(ObjectType::Tree),
            write_tree(&path)?,
            filename
        ))
    } else {
        Ok(format!(
            "{} {} {}",
            data::get_type_string(ObjectType::Blob),
            data::hash_object(&fs::read(&path)?, ObjectType::Blob)?,
            filename
        ))
    };
}

pub fn write_tree<P: AsRef<Path>>(dir: P) -> std::io::Result<String> {
    let mut tree_contents = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if is_ignored(&path) {
            continue;
        }

        tree_contents.push(write_tree_entry(entry)?)
    }
    data::hash_object(tree_contents.join("\n").as_bytes(), ObjectType::Tree)
}

struct TreeEntry {
    t: ObjectType,
    oid: String,
    name: String,
}

fn get_tree_entry(tree_entry: &str) -> Option<TreeEntry> {
    let fields: Vec<&str> = tree_entry.splitn(3, ' ').collect();
    let t_bytes = fields.get(0)?.as_bytes();
    Some(TreeEntry {
        t: data::get_type_from_bytes(t_bytes)?,
        oid: fields.get(1)?.to_string(),
        name: fields.get(2)?.to_string(),
    })
}

fn get_tree_entries(tree_oid: &str) -> std::io::Result<Vec<TreeEntry>> {
    let tree_contents = data::get_object(tree_oid, Some(ObjectType::Tree))?.contents;
    let tree_string = String::from_utf8_lossy(&tree_contents);
    Ok(tree_string
        .split("\n")
        .map(|line| get_tree_entry(line).unwrap())
        .collect())
}

fn get_tree(tree_oid: &str, base_path: PathBuf) -> std::io::Result<HashMap<PathBuf, String>> {
    let mut result = HashMap::new();
    for entry in get_tree_entries(tree_oid)? {
        if entry.name == "." || entry.name == ".." || entry.name.contains('/') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Bad entry in tree object",
            ));
        }
        let base_path = Path::new(&base_path);
        let path = base_path.join(entry.name);

        match entry.t {
            ObjectType::Blob => {
                let old_oid = result.insert(path, entry.oid.clone());
                if let Some(old_oid) = old_oid {
                    if old_oid != entry.oid {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Tree object contains multiple object IDs for the same file",
                        ));
                    }
                }
            }
            ObjectType::Tree => {
                result.extend(get_tree(&entry.oid, path)?);
            }
        }
    }
    Ok(result)
}

fn clear_dir<P: AsRef<Path>>(dir: P) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if is_ignored(&path) {
            continue;
        }

        if path.is_dir() {
            clear_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn read_tree(tree_oid: &str) -> std::io::Result<()> {
    let base_path = Path::new(".").to_path_buf();
    clear_dir(&base_path)?;
    for (path, oid) in get_tree(tree_oid, base_path)? {
        if let Some(parent) = path.parent() {
            if parent.is_dir() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(
            path,
            data::get_object(&oid, Some(ObjectType::Blob))?.contents,
        )?;
    }
    Ok(())
}