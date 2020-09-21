use crate::data::{self, ObjectType};
use std::fs;
use std::path::{Component, Path};

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

pub fn write_tree<P: AsRef<Path>>(dir: P) -> std::io::Result<String> {
    let mut tree_contents = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name().into_string().unwrap();

        if is_ignored(&path) {
            continue;
        }
        if path.is_dir() {
            tree_contents.push(format!("{} {}", write_tree(&path)?, filename));
        } else {
            tree_contents.push(format!(
                "{} {}",
                data::hash_object(&fs::read(&path)?, ObjectType::Blob)?,
                filename
            ));
        }
    }
    data::hash_object(tree_contents.join("\n").as_bytes(), ObjectType::Tree)
}
