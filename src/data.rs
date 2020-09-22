use sha1::{Digest, Sha1};
use std::fs;

pub const GIT_DIR: &str = ".gitox";
const OBJECT_DIR: &str = ".gitox/objects";

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    Blob,
    Tree,
}

#[derive(Debug)]
pub struct Object {
    pub t: ObjectType,
    pub contents: Vec<u8>,
}

pub fn get_type_from_bytes(bytes: &[u8]) -> Option<ObjectType> {
    match bytes {
        b"blob" => Some(ObjectType::Blob),
        b"tree" => Some(ObjectType::Tree),
        _ => None,
    }
}

pub fn get_type_string(t: ObjectType) -> &'static str {
    match t {
        ObjectType::Blob => "blob",
        ObjectType::Tree => "tree",
    }
}

pub fn init() -> std::io::Result<()> {
    fs::create_dir_all(GIT_DIR)?;
    fs::create_dir_all(OBJECT_DIR)?;
    Ok(())
}

pub fn hash_object(contents: &[u8], t: ObjectType) -> std::io::Result<String> {
    let t_str = match t {
        ObjectType::Blob => "blob",
        ObjectType::Tree => "tree",
    };

    // Format of an object is its type, null byte then the contents
    let data = [t_str.as_bytes(), b"\x00", contents].concat();
    let hash = Sha1::digest(&data);
    let oid = format!("{:x}", hash);

    fs::write(format!("{}/{oid}", OBJECT_DIR, oid = oid), data)?;
    Ok(oid)
}

pub fn get_object(oid: &str, expected: Option<ObjectType>) -> std::io::Result<Object> {
    let raw = fs::read(format!("{}/{oid}", OBJECT_DIR, oid = oid))?;

    // Object type is the first byte slice before a null byte
    let fields: Vec<&[u8]> = raw.splitn(2, |c| *c == 0).collect();
    let t_bytes = fields.get(0).unwrap();
    let contents = fields.get(1).unwrap();
    let t = get_type_from_bytes(t_bytes).unwrap();

    if let Some(expected) = expected {
        if expected != t {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Expected {:?}, retrieved {:?} object", expected, t),
            ));
        }
    }

    Ok(Object {
        t: t,
        contents: contents.to_vec(),
    })
}
