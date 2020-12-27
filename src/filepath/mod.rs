//! Simplifed filepath module for cleaning input paths.
//! The aim is to able compare paths which are quite
//! similar to each other to return a common path
//! string shared by these paths.
//!
//! Eg. Directory `src`
//! src
//! ./src
//! ./src/
//! src/
//! All points to the same directory.
//!
//! This is a simplified smaller version of filepath
//! module used in Golang.
//!
//! To see actual golang implentation, please visit
//! https://golang.org/src/path/filepath/path.go?h=path.

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

pub fn volume_name_len<P: AsRef<Path>>(path: P) -> usize {
    if cfg!(windows) {
        return windows::volume_name_len(path.as_ref());
    }

    0
}

pub fn from_slash<P: AsRef<Path>>(path: P) -> PathBuf {
    if os_separator() == '/' {
        return path.as_ref().to_path_buf();
    }

    let new_path = path
        .as_ref()
        .to_str()
        .unwrap()
        .replace("/", &os_separator().to_string());

    PathBuf::from(new_path)
}

pub fn clean<P: AsRef<Path>>(path: P) -> PathBuf {
    let path_vec = path
        .as_ref()
        .to_str()
        .unwrap()
        .chars()
        .collect::<Vec<char>>();
    let vol_len = volume_name_len(path.as_ref());
    let path_without_vol = &path_vec[vol_len..path_vec.len()];

    if path_without_vol.is_empty() {
        if vol_len > 1 && path_vec[1] != ':' {
            // UNC pathing probably.
            return from_slash(path.as_ref());
        }

        // For empty paths, return prefix + ".".
        let mut new_path = path.as_ref().to_path_buf();
        new_path.push(".");
        return new_path;
    }

    let (mut r, mut dotdot) = (0, 0);
    let n = path_without_vol.len();
    let rooted = is_path_separator(path_without_vol[0]);

    // Actual implementation uses a lazy buffer to save space
    // by reusing the path.
    //
    // But for simplicity, I will use a simple vector for now.
    let mut out = Vec::<char>::new();

    if rooted {
        out.push(os_separator());
        r = 1;
        dotdot = 1;
    }

    while r < n {
        if is_path_separator(path_without_vol[r]) {
            // Empty path element
            r += 1;
        } else if path_without_vol[r] == '.'
            && (r + 1 == n || is_path_separator(path_without_vol[r + 1]))
        {
            // '.' element followed by '/' or the next char is the end of path.
            r += 1;
        } else if path_without_vol[r] == '.'
            && path_without_vol[r + 1] == '.'
            && (r + 2 == n || is_path_separator(path_without_vol[r + 2]))
        {
            // .. element: remove to last separator
            r += 2;
            if out.len() > dotdot + 1 {
                out.pop();
                while let Some(&last_char) = out.last() {
                    if out.len() <= dotdot || is_path_separator(last_char) {
                        break;
                    }
                    out.pop();
                }
            } else if !rooted {
                if out.len() > 0 {
                    if let Some(&last_char) = out.last() {
                        if !is_path_separator(last_char) {
                            out.push(os_separator());
                        }
                    }
                }
                out.push('.');
                out.push('.');
                dotdot = out.len();
            }
        } else {
            // Default

            if rooted && out.len() != 1 || !rooted && out.len() != 0 {
                // In lazybuf, append does neccessarily mean push new character
                // onto array but instead could mean reusing latest character.
                //
                // So only push separator if last character isn't.
                if let Some(&last_char) = out.last() {
                    if !is_path_separator(last_char) {
                        out.push(os_separator());
                    }
                }
            }

            // Copy non-separator characters.
            while r < n && !is_path_separator(path_without_vol[r]) {
                out.push(path_without_vol[r]);
                r += 1;
            }
        }
    }

    if out.len() == 0 {
        out.push('.');
    }

    // Remove any last separator since
    // expected clean path looks like this:
    // {path prefix}/{some name}
    if let Some(&last_char) = out.last() {
        if out.len() > 1 && is_path_separator(last_char) {
            out.pop();
        }
    }

    PathBuf::from(out.into_iter().collect::<String>())
}

#[test]
fn test_clean() {
    if cfg!(windows) {
        // TODO
    } else {
        let path_strs = [
            // Already clean
            ("abc", "abc"),
            ("abc/def", "abc/def"),
            ("a/b/c", "a/b/c"),
            (".", "."),
            ("..", ".."),
            ("../..", "../.."),
            ("../../abc", "../../abc"),
            ("/abc", "/abc"),
            ("/", "/"),
            // Empty is current dir
            ("", "."),
            // Remove trailing slash
            ("abc/", "abc"),
            ("abc/def/", "abc/def"),
            ("a/b/c/", "a/b/c"),
            ("./", "."),
            ("../", ".."),
            ("../../", "../.."),
            ("/abc/", "/abc"),
            // Remove doubled slash
            ("abc//def//ghi", "abc/def/ghi"),
            ("//abc", "/abc"),
            ("///abc", "/abc"),
            ("//abc//", "/abc"),
            ("abc//", "abc"),
            // Remove . elements
            ("abc/./def", "abc/def"),
            ("/./abc/def", "/abc/def"),
            ("abc/.", "abc"),
            // Remove..elements
            ("abc/def/ghi/../jkl", "abc/def/jkl"),
            ("abc/def/../ghi/../jkl", "abc/jkl"),
            ("abc/def/..", "abc"),
            ("abc/def/../..", "."),
            ("/abc/def/../..", "/"),
            ("abc/def/../../..", ".."),
            ("/abc/def/../../..", "/"),
            ("abc/def/../../../ghi/jkl/../../../mno", "../../mno"),
            ("/../abc", "/abc"),
            // Combinations
            ("abc/./../def", "def"),
            ("abc//./../def", "def"),
            ("abc/../../././../def", "../../def"),
        ];

        for (path_str, expected_output) in path_strs.iter() {
            let path = Path::new(path_str);
            assert_eq!(clean(path).to_str(), Some(*expected_output));
        }
    }
}

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
