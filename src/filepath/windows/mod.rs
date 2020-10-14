pub static PATH_SEPARATOR: char = '\\';

pub fn is_path_separator(c: char) -> bool {
    c == PATH_SEPARATOR
}
