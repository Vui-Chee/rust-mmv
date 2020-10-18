use fancy_regex::Regex;

use std::path::Path;

pub static PATH_SEPARATOR: char = '\\';

// Windows uses backslashes for filesystem paths
// and forward slash for everything else.
//
// But for this use case, I am only interested in
// backslashes.
fn is_slash(c: char) -> bool {
    c == '\\'
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

    // Rust represents strings as UTF-8 internally.
    //
    // NOTE not all path characters may be represented as UTF-8.
    // See https://docs.racket-lang.org/reference/windowspaths.html
    let path_vec = path_str.chars().collect::<Vec<char>>();

    if path_vec.len() < 2 {
        return 0;
    }

    // Drive letter
    let c: char = path_vec[0];

    // Check for volume names such as
    // "C:\".
    if path_vec[0] == ':' || ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') {
        return 2;
    }

    // Get volume name length from UNC paths.
    // See https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247(v=vs.85).aspx
    //
    // UNC paths begin with \\ (two slashes) but note in tests will "\\\\" for escape.
    // The third position cannot be occupied with another '\' or '.'.
    //
    // UNC volume names looks something like "\\.*\.*".
    // Note '.' does not all characters but actually [^\.\\] - any character except
    // '.' or '\'.
    let re = Regex::new(r"\\\\[^\.\\][^\\]*\\[^\.\\][^\\]+").unwrap();
    if let Ok(Some(matches)) = re.find(path_str) {
        return path_str[0..matches.end()]
            .chars()
            .collect::<Vec<char>>()
            .len();
    }

    0
}

#[test]
fn test_volume_name_len() {
    let paths = [
        // non utf-8
        ("\\\\ふー\\バー", 7),
        // volumes
        ("C:", 2),
        // UNC cases
        ("\\\\teela\\", 0),
        ("\\\\teela\\admin\\folder", 13),
        ("\\\\?\\REL\\..\\\\..", 7),
        ("\\\\first\\next", 12),
        ("\\\\dir\\file.txt", 14),
        ("\\\\some.dir\\file", 15),
        // No volume cases
        (".\\temp.txt", 0),
        ("..\\Publications\\TravelBrochure.pdf", 0),
        ("\\\\\\", 0),
        ("\\\\.", 0),
        ("\\abc\\", 0),
    ];

    for (path, expected_len) in paths.iter() {
        assert_eq!(volume_name_len(Path::new(path)), *expected_len as usize);
    }
}
