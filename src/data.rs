use sha1::{Digest, Sha1};
use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub const GIT_DIR: &str = ".gitox";
const OBJECT_DIR: &str = ".gitox/objects";

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
    type Err = std::io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Parsed string cannot represent a known object type",
            )),
        }
    }
}

impl TryFrom<&[u8]> for ObjectType {
    type Error = std::io::Error;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_str(&String::from_utf8(bytes.to_vec()).unwrap())
    }
}

#[derive(Debug)]
pub struct Object {
    pub t: ObjectType,
    pub contents: Vec<u8>,
}

pub type Oid = String;

pub fn init() -> std::io::Result<()> {
    fs::create_dir_all(GIT_DIR)?;
    fs::create_dir_all(OBJECT_DIR)?;
    Ok(())
}

pub fn hash_object(contents: &[u8], t: ObjectType) -> std::io::Result<Oid> {
    // Format of an object is its type, null byte then the contents
    let t_str = format!("{}", t);
    let data = [t_str.as_bytes(), b"\x00", contents].concat();
    let hash = Sha1::digest(&data);
    let oid = format!("{:x}", hash);

    fs::write(format!("{}/{oid}", OBJECT_DIR, oid = oid), data)?;
    Ok(oid)
}

pub fn get_object(oid: &Oid, expected: Option<ObjectType>) -> std::io::Result<Object> {
    let raw = fs::read(format!("{}/{oid}", OBJECT_DIR, oid = oid))?;

    // Object type is the first byte slice before a null byte
    let fields: Vec<&[u8]> = raw.splitn(2, |c| *c == b'\0').collect();
    let t_bytes = fields.get(0).unwrap();
    let contents = fields.get(1).unwrap();
    let t = ObjectType::try_from(*t_bytes)?;

    if let Some(expected) = expected {
        if expected != t {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Expected {:?}, retrieved {:?} object", expected, t),
            ));
        }
    };

    Ok(Object {
        t: t,
        contents: contents.to_vec(),
    })
}

pub fn set_head(oid: &Oid) -> std::io::Result<()> {
    fs::write(format!("{}/HEAD", GIT_DIR), oid)
}

pub fn get_head() -> std::io::Result<Option<Oid>> {
    let head = format!("{}/HEAD", GIT_DIR);
    let head_path = Path::new(&head);
    Ok(match head_path.exists() {
        false => None,
        true => Some(Oid::from_utf8_lossy(&fs::read(head_path)?).to_string()),
    })
}
