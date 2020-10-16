use std::path::Path;

pub static PATH_SEPARATOR: char = '/';

pub fn is_path_separator(c: char) -> bool {
    c == PATH_SEPARATOR
}

pub fn volume_name_len(path: &Path) -> u32 {
    0
}
