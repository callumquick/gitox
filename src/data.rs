use sha1::{Digest, Sha1};
use std::fs;

const GIT_DIR: &str = ".gitox";
const OBJECT_DIR: &str = ".gitox/objects";

pub fn init() -> std::io::Result<()> {
    fs::create_dir_all(GIT_DIR)?;
    fs::create_dir_all(OBJECT_DIR)?;
    Ok(())
}

pub fn hash_object(data: &[u8]) -> std::io::Result<String> {
    let hash = Sha1::digest(data);
    let oid = format!("{:x}", hash);
    fs::write(format!("{}/{oid}", OBJECT_DIR, oid = oid), data)?;
    Ok(oid)
}

pub fn get_object(oid: &str) -> std::io::Result<Vec<u8>> {
    fs::read(format!("{}/{oid}", OBJECT_DIR, oid = oid))
}
