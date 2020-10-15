use std::path::Path;

pub static PATH_SEPARATOR: char = '\\';

fn char_at(bytes: &[u8], index: usize) -> char {
    bytes[index] as char
}

fn is_slash(c: char) -> bool {
    c == '\\' || c == '/'
}

pub fn is_path_separator(c: char) -> bool {
    c == PATH_SEPARATOR
}

/// volumeNameLen returns length of the leading volume name on Windows.
/// It returns 0 elsewhere.
pub fn volume_name_len(path: &Path) -> usize {
    let path_str = match path.to_str() {
        Some(path_str) => path_str,
        None => "",
    };

    if path_str.len() < 2 {
        return 0;
    }

    // Rust represents strings as UTF-8 internally.
    // Only works if characters contain ASCII characters only.
    let path_bytes = path_str.as_bytes();
    // Drive letter
    let c: char = char_at(path_bytes, 0);
    if char_at(path_bytes, 1) == ':' || ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') {
        return 2;
    }

    // is it UNC? https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247(v=vs.85).aspx
    if path_str.len() >= 5
        && is_slash(c)
        && is_slash(char_at(path_bytes, 1))
        && !is_slash(char_at(path_bytes, 2))
        && char_at(path_bytes, 2) != '.'
    {
        // first, leading `\\` and next shouldn't be `\`. its server name.
        for mut i in 3..path_str.len() - 1 {
            if is_slash(char_at(path_bytes, i)) {
                i += 1;
                if !is_slash(char_at(path_bytes, i)) {
                    if char_at(path_bytes, i) == '.' {
                        break;
                    }
                    while i < path_str.len() {
                        if is_slash(char_at(path_bytes, i)) {
                            return i;
                        }
                        i += 1;
                    }
                    return i;
                }
                break;
            }
        }
    }

    0
}
