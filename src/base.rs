use crate::data::{self, ObjectType, Oid, RefValue};
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::{Into, TryFrom};
use std::fs::{self, DirEntry};
use std::io::{Error, ErrorKind, Result};
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

pub fn init() -> Result<()> {
    data::init()?;
    data::update_ref(
        "HEAD",
        RefValue {
            symbolic: true,
            value: Some("refs/heads/master".to_string()),
        },
        true,
    )
}

fn write_tree_entry(dir_entry: DirEntry) -> Result<String> {
    let path = dir_entry.path();
    let filename = dir_entry.file_name().into_string().unwrap();
    return if path.is_dir() {
        Ok(format!(
            "{} {} {}",
            ObjectType::Tree,
            write_tree(&path)?,
            filename
        ))
    } else {
        Ok(format!(
            "{} {} {}",
            ObjectType::Blob,
            data::hash_object(&fs::read(&path)?, ObjectType::Blob)?,
            filename
        ))
    };
}

pub fn write_tree<P: AsRef<Path>>(dir: P) -> Result<String> {
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
    oid: Oid,
    name: String,
}

fn get_tree_entry(tree_entry: &str) -> Option<TreeEntry> {
    let fields: Vec<&str> = tree_entry.splitn(3, ' ').collect();
    let t_bytes = fields.get(0)?.as_bytes();
    Some(TreeEntry {
        t: ObjectType::try_from(t_bytes).unwrap(),
        oid: fields.get(1)?.to_string(),
        name: fields.get(2)?.to_string(),
    })
}

fn get_tree_entries(tree_oid: Option<&Oid>) -> Result<Vec<TreeEntry>> {
    if tree_oid.is_none() {
        return Ok(Vec::new());
    }
    let tree_oid = tree_oid.unwrap();
    let tree_contents = data::get_object(&tree_oid.to_string(), Some(ObjectType::Tree))?.contents;
    let tree_string = String::from_utf8_lossy(&tree_contents);
    Ok(tree_string
        .split("\n")
        .map(|line| get_tree_entry(line).unwrap())
        .collect())
}

pub type Tree = HashMap<PathBuf, Oid>;
pub fn get_tree(tree_oid: Option<&Oid>, base_path: PathBuf) -> Result<Tree> {
    let mut result = Tree::new();
    for entry in get_tree_entries(tree_oid)? {
        if entry.name == "." || entry.name == ".." || entry.name.contains('/') {
            return Err(Error::new(ErrorKind::Other, "Bad entry in tree object"));
        }
        let base_path = Path::new(&base_path);
        let path = base_path.join(entry.name);

        match entry.t {
            ObjectType::Blob => {
                let old_oid = result.insert(path, entry.oid.clone());
                if let Some(old_oid) = old_oid {
                    if old_oid != entry.oid {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "Tree object contains multiple object IDs for the same file",
                        ));
                    }
                }
            }
            ObjectType::Tree => {
                result.extend(get_tree(Some(&entry.oid), path)?);
            }
            _ => {
                // Other object types are not valid to be stored within tree
                // objects (commit etc)
                return Err(Error::new(
                    ErrorKind::Other,
                    "Tree object contained object ID for bad type (not blob, tree)",
                ));
            }
        }
    }
    Ok(result)
}

fn clear_dir<P: AsRef<Path>>(dir: P) -> Result<()> {
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

pub fn read_tree(tree_oid: &Oid) -> Result<()> {
    let base_path = Path::new(".").to_path_buf();
    clear_dir(&base_path)?;
    for (path, oid) in get_tree(Some(tree_oid), base_path)? {
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

pub struct Commit {
    pub tree: Oid,
    pub parent: Option<Oid>,
    pub message: String,
}

impl Into<String> for Commit {
    fn into(self) -> String {
        let mut commit = String::new();
        let mut commit_headers = Vec::new();
        commit_headers.push(format!("{} {}", ObjectType::Tree, self.tree));
        if let Some(parent) = self.parent {
            commit_headers.push(format!("parent {}", parent));
        }
        commit.push_str(&commit_headers.join("\n"));
        // Message separator is a blank line
        commit.push_str("\n\n");
        commit.push_str(&self.message);

        commit
    }
}

impl TryFrom<String> for Commit {
    type Error = Error;
    fn try_from(s: String) -> Result<Self> {
        let lines: Vec<&str> = s.split("\n").collect();
        let mut properties: HashMap<&str, &str> = HashMap::new();
        let mut finished_header = false;
        let mut message_lines: Vec<&str> = Vec::new();

        for line in lines {
            if line.is_empty() {
                finished_header = true;
            } else if !finished_header {
                let fields: Vec<&str> = line.splitn(2, " ").collect();
                if fields.len() != 2 {
                    return Err(Self::Error::new(
                        ErrorKind::Other,
                        "Commit file has corrupted property header",
                    ));
                }
                let key = fields.get(0).unwrap();
                let value = fields.get(1).unwrap();
                properties.insert(key, value);
            } else {
                message_lines.push(line);
            }
        }

        let message = message_lines.join("\n");

        if !properties.contains_key("tree") {
            return Err(Self::Error::new(
                ErrorKind::Other,
                "Commit file does not contain 'tree' field",
            ));
        }

        Ok(Commit {
            tree: properties.get("tree").unwrap().to_string(),
            parent: properties.get("parent").map(|s| s.to_string()),
            message: message,
        })
    }
}

pub fn commit(message: &str) -> Result<Oid> {
    let commit = Commit {
        tree: write_tree(".")?,
        parent: data::get_ref("HEAD", true)?.value,
        message: message.to_string(),
    };
    let commit_str: String = commit.into();
    let oid = data::hash_object(commit_str.as_bytes(), ObjectType::Commit)?;
    data::update_ref(
        "HEAD",
        RefValue {
            symbolic: false,
            value: Some(oid.clone()),
        },
        true,
    )?;
    Ok(oid)
}

pub fn get_commit(oid: &Oid) -> Result<Commit> {
    let commit = data::get_object(oid, Some(ObjectType::Commit))?;
    Ok(Commit::try_from(
        String::from_utf8_lossy(&commit.contents)
            .to_owned()
            .to_string(),
    )?)
}

pub fn is_branch(name: &str) -> Result<bool> {
    Ok(data::get_ref(&format!("refs/heads/{}", name), true)?
        .value
        .is_some())
}

pub fn checkout(name: &str) -> Result<()> {
    let oid = get_oid(name)?;
    let commit = get_commit(&oid)?;
    read_tree(&commit.tree)?;

    let head = if is_branch(name)? {
        RefValue {
            symbolic: true,
            value: Some(format!("refs/heads/{}", name)),
        }
    } else {
        RefValue {
            symbolic: false,
            value: Some(oid),
        }
    };

    data::update_ref("HEAD", head, false)
}

pub fn reset(oid: Oid) -> Result<()> {
    let commit = get_commit(&oid)?;
    read_tree(&commit.tree)?;

    data::update_ref(
        "HEAD",
        RefValue {
            symbolic: false,
            value: Some(oid),
        },
        true,
    )
}

pub fn create_tag(name: &str, oid: &Oid) -> Result<()> {
    let tag_path = format!("refs/tags/{}", name);
    data::update_ref(
        &tag_path,
        RefValue {
            symbolic: false,
            value: Some(oid.clone()),
        },
        true,
    )
}

pub fn create_branch(name: &str, oid: &Oid) -> Result<()> {
    let branch_path = format!("refs/heads/{}", name);
    data::update_ref(
        &branch_path,
        RefValue {
            symbolic: false,
            value: Some(oid.clone()),
        },
        true,
    )
}

pub fn get_branch_name() -> Result<Option<String>> {
    let head = data::get_ref("HEAD", false)?;
    if !head.symbolic {
        return Ok(None);
    }
    let head = head.value;
    match head {
        None => Ok(None),
        Some(refname) => Ok(refname
            .strip_prefix("refs/heads/")
            .map(|str_| str_.to_string())),
    }
}

pub fn iter_branch_names() -> Result<impl Iterator<Item = String>> {
    Ok(data::iter_refs(Some("refs/heads/"), true)?
        .map(|(refname, _)| refname.strip_prefix("refs/heads/").unwrap().to_string()))
}

/// Attempt to retrieve the OID from a reference, but otherwise return the
/// reference assuming it is itself an OID.
pub fn get_oid(ref_: &str) -> Result<Oid> {
    let ref_translations: HashMap<&str, &str> = [("@", "HEAD")].iter().cloned().collect();
    let ref_str: &str = ref_translations.get(ref_).unwrap_or(&ref_);

    let paths_to_try = [
        format!("{}", ref_str),
        format!("refs/{}", ref_str),
        format!("refs/tags/{}", ref_str),
        format!("refs/heads/{}", ref_str),
    ];

    for path in &paths_to_try {
        if let Some(value) = data::get_ref(&path, true)?.value {
            return Ok(value);
        }
    }

    if ref_.len() != 40 || ref_.chars().any(|c| !c.is_ascii_hexdigit()) {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Unknown name given: {}", ref_),
        ));
    }

    Ok(ref_.to_string())
}

// When using this function, the input Oids should be unique and non-relational
// (not the parent of another input Oid) to ensure that their order is preserved
// when they are output from the resulting iterator.
pub fn iter_commits_and_parents(
    oids: impl Iterator<Item = Oid>,
) -> Result<impl Iterator<Item = Oid>> {
    let mut commits: Vec<Oid> = Vec::new();
    let mut oidqueue: VecDeque<Oid> = oids.collect();
    let mut visited: HashSet<Oid> = HashSet::new();

    while !oidqueue.is_empty() {
        let oid = oidqueue.pop_front().unwrap();

        if !visited.insert(oid.clone()) {
            continue;
        }

        let commit = get_commit(&oid)?;
        if commit.parent.is_some() {
            // Return parent next
            oidqueue.push_front(commit.parent.unwrap());
        }
        commits.push(oid);
    }

    Ok(commits.into_iter())
}
