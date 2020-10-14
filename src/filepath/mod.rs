//! Simplifed filepath module for cleaning input paths.
//!
//! This is a simplified smaller version of filepath
//! module used in Golang. For this use case, this will
//! only implement pathing cases for Unix systems.
//!
//! TODO
//! 1. Implement is_path_separator() for different os
//! 2. Implement from_slash()
//! 3. Implement clean(filepath: PathBuf)
//!

mod linux;
mod unix;
mod windows;

use std::path::{Path, PathBuf};

pub fn os_separator() -> char {
    if cfg!(unix) {
        return unix::PATH_SEPARATOR;
    } else if cfg!(linux) {
        return linux::PATH_SEPARATOR;
    } else if cfg!(windows) {
        return windows::PATH_SEPARATOR;
    }

    // default separator
    '/'
}

pub fn is_path_separator(c: char) -> bool {
    if cfg!(unix) {
        return unix::is_path_separator(c);
    } else if cfg!(linux) {
        return linux::is_path_separator(c);
    } else if cfg!(windows) {
        return windows::is_path_separator(c);
    }

    // None of the OS
    false
}

pub fn from_slash(path: &Path) -> PathBuf {
    if os_separator() == '/' {
        return path.to_path_buf();
    }

    let new_path = path
        .to_str()
        .unwrap()
        .replace("/", &os_separator().to_string());

    PathBuf::from(new_path)
}

#[cfg(test)]
mod filepath_test {
    use super::from_slash;
    use std::path::Path;

    #[test]
    fn test_from_slash() {
        let str_path = "/test_dir/file.index.html";
        let path = Path::new(str_path);
        let final_path = from_slash(path);

        if cfg!(windows) {
            let expected_str_path = "\\test_dir\\file.index.html";
            assert_eq!(final_path, Path::new(expected_str_path));
        } else {
            assert_eq!(path, final_path);
        }
    }
}
