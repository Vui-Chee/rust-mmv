use std::path::Path;

pub static PATH_SEPARATOR: char = '/';

pub fn is_path_separator(c: char) -> bool {
    c == PATH_SEPARATOR
}

#[allow(dead_code)]
pub fn volume_name_len(_path: &Path) -> u32 {
    0
}
