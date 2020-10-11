use std::collections::HashMap;
use std::path::PathBuf;

pub fn rename(files: HashMap<PathBuf, PathBuf>) {
    println!("{:?}", files);
}
