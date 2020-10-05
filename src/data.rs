use sha1::{Digest, Sha1};
use std::convert::TryFrom;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::str::FromStr;

pub const GIT_DIR: &str = ".gitox";
const OBJECT_DIR: &str = ".gitox/objects";
const REF_DIR: &str = ".gitox/refs";

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ObjectType::Blob => "blob",
                ObjectType::Tree => "tree",
                ObjectType::Commit => "commit",
            }
        )
    }
}

impl FromStr for ObjectType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            _ => Err(Error::new(
                ErrorKind::Other,
                "Parsed string cannot represent a known object type",
            )),
        }
    }
}

impl TryFrom<&[u8]> for ObjectType {
    type Error = Error;
    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_str(&String::from_utf8(bytes.to_vec()).unwrap())
    }
}

#[derive(Debug)]
pub struct Object {
    pub t: ObjectType,
    pub contents: Vec<u8>,
}

pub type Oid = String;

pub fn init() -> Result<()> {
    fs::create_dir_all(GIT_DIR)?;
    fs::create_dir_all(OBJECT_DIR)?;
    fs::create_dir_all(REF_DIR)?;
    Ok(())
}

pub fn hash_object(contents: &[u8], t: ObjectType) -> Result<Oid> {
    // Format of an object is its type, null byte then the contents
    let t_str = format!("{}", t);
    let data = [t_str.as_bytes(), b"\x00", contents].concat();
    let hash = Sha1::digest(&data);
    let oid = format!("{:x}", hash);

    fs::write(format!("{}/{oid}", OBJECT_DIR, oid = oid), data)?;
    Ok(oid)
}

pub fn get_object(oid: &Oid, expected: Option<ObjectType>) -> Result<Object> {
    let raw = fs::read(format!("{}/{oid}", OBJECT_DIR, oid = oid))?;

    // Object type is the first byte slice before a null byte
    let fields: Vec<&[u8]> = raw.splitn(2, |c| *c == b'\0').collect();
    let t_bytes = fields.get(0).unwrap();
    let contents = fields.get(1).unwrap();
    let t = ObjectType::try_from(*t_bytes)?;

    if let Some(expected) = expected {
        if expected != t {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Expected {:?}, retrieved {:?} object", expected, t),
            ));
        }
    };

    Ok(Object {
        t: t,
        contents: contents.to_vec(),
    })
}

pub struct RefValue {
    pub symbolic: bool,
    pub value: Option<String>,
}

fn get_ref_internal(ref_: &str, deref: bool) -> Result<(String, RefValue)> {
    let ref_object = format!("{}/{}", GIT_DIR, ref_);
    let ref_path = Path::new(&ref_object);
    let mut symbolic = false;
    let ref_value = match ref_path.exists() {
        false => None,
        true => Some(Oid::from_utf8_lossy(&fs::read(ref_path)?).to_string()),
    };
    let mut value = ref_value.clone();

    if let Some(ref_value) = ref_value {
        if let Some(sym_ref) = ref_value.strip_prefix("ref: ") {
            value = Some(sym_ref.to_string());
            symbolic = true;
            if deref {
                // Recursively dereference the symbolic ref
                return get_ref_internal(sym_ref, true);
            }
        }
    }

    Ok((
        ref_.to_string(),
        RefValue {
            symbolic,
            value: value,
        },
    ))
}

pub fn update_ref(ref_: &str, value: RefValue, deref: bool) -> Result<()> {
    let ref_ = get_ref_internal(ref_, deref).map(|(ref_, _)| ref_)?;

    let raw_value = value
        .value
        .expect("Cannot update a reference with an empty value");
    let raw_value: String = match value.symbolic {
        true => "ref: ".to_string() + &raw_value,
        false => raw_value,
    };

    let ref_object = format!("{}/{}", GIT_DIR, ref_);
    let ref_path = Path::new(&ref_object);
    fs::create_dir_all(ref_path.parent().unwrap())?;
    fs::write(ref_object, raw_value)
}

pub fn get_ref(ref_: &str, deref: bool) -> Result<RefValue> {
    get_ref_internal(ref_, deref).map(|(_, value)| value)
}

fn append_ref_paths(mut v: Vec<String>, dir: &Path) -> Result<Vec<String>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            v = append_ref_paths(v, entry.path().as_path())?;
        } else {
            v.push(
                entry
                    .path()
                    .strip_prefix(GIT_DIR)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
        }
    }
    Ok(v)
}

pub fn iter_refs(
    prefix: Option<&str>,
    deref: bool,
) -> Result<impl Iterator<Item = (String, RefValue)>> {
    let mut refnames: Vec<String> = Vec::new();
    let mut refs = Vec::new();
    refnames.push("HEAD".to_string());
    refnames = append_ref_paths(refnames, Path::new(REF_DIR))?;

    for refname in refnames {
        if let Some(prefix) = prefix {
            if !refname.starts_with(prefix) {
                continue;
            }
        }
        let value = get_ref(&refname, deref)?;
        refs.push((refname, value));
    }

    Ok(refs.into_iter())
}
