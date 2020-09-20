use std::fs;

const GIT_DIR: &str = ".gitox";

pub fn init() {
    fs::create_dir_all(GIT_DIR).unwrap();
}
