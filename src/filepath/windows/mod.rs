use super::char_at;

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

    if path_str.len() < 2 {
        return 0;
    }

    // Rust represents strings as UTF-8 internally.
    //
    // NOTE not all windows path characters are represented as UTF-8.
    // See https://docs.racket-lang.org/reference/windowspaths.html
    let path_bytes = path_str.as_bytes();
    // Drive letter
    let c: char = char_at(path_bytes, 0);

    // Check for volume names such as
    // "C:\".
    if char_at(path_bytes, 1) == ':' || ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') {
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
        return matches.end();
    }

    0
}

#[test]
fn volume_cases() {
    let path = Path::new("C:");
    assert_eq!(volume_name_len(&path), 2);
}

#[test]
fn unc_cases() {
    let path = Path::new("\\\\teela\\");
    assert_eq!(volume_name_len(&path), 0);
    let path = Path::new("\\\\teela\\admin\\folder");
    assert_eq!(volume_name_len(&path), 13);
    let path = Path::new("\\\\?\\REL\\..\\\\..");
    assert_eq!(volume_name_len(&path), 7);

    // Without trailing backslash
    let path = Path::new("\\\\first\\next");
    assert_eq!(volume_name_len(&path), 12);

    // File with extensions
    let path = Path::new("\\\\dir\\file.txt");
    assert_eq!(volume_name_len(&path), 14);

    // Directory with '.'
    let path = Path::new("\\\\some.dir\\file");
    assert_eq!(volume_name_len(&path), 15);
}

#[test]
fn no_volume_cases() {
    // Relative paths (do not contain volumn prefixes)
    let path = Path::new(".\\temp.txt");
    assert_eq!(volume_name_len(&path), 0);
    let path = Path::new("..\\Publications\\TravelBrochure.pdf");
    assert_eq!(volume_name_len(&path), 0);

    // Other edge cases
    let path = Path::new("\\\\\\");
    assert_eq!(volume_name_len(&path), 0);
    let path = Path::new("\\\\.");
    assert_eq!(volume_name_len(&path), 0);

    // This case
    let path = Path::new("\\abc\\");
    assert_eq!(volume_name_len(&path), 0);
}
