use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::filepath::clean;
use super::ioutils::next_random;

static EMPTY_PATH_ERROR: &'static str = "Path cannot be empty.";

#[derive(Debug)]
pub struct Edge {
    pub src: PathBuf,
    pub dst: PathBuf,
}

pub fn rename(files: &HashMap<PathBuf, PathBuf>, dir: Option<&str>) -> Result<(), String> {
    let mut dir_path = "";
    if dir.is_some() {
        dir_path = dir.unwrap();
    }

    // Closure to prepend leading directory to existing paths.
    let prefix_paths = |src: &PathBuf, dst: &PathBuf| {
        (
            PathBuf::from(dir_path).join(src),
            PathBuf::from(dir_path).join(dst),
        )
    };

    match build_renames(files) {
        Ok(renames) => {
            for (i, rename) in renames.iter().enumerate() {
                let (src, dst) = prefix_paths(&rename.src, &rename.dst);

                if let Err(err) = do_rename(src.as_path(), dst.as_path()) {
                    // Only undo if there is more than 1 previous renames.
                    // Otherwise, j - 1 yields an overflow error (since i is usize).
                    if i >= 1 {
                        let mut j = i - 1;
                        loop {
                            // NOTE: dst is now the source path and vice-versa.
                            let (src, dst) = prefix_paths(&renames[j].dst, &renames[j].src);
                            // Undo on error not to leave the temporary files.
                            // This does not undo directory creation.
                            if let Err(_err) = fs::rename(src, dst) {
                                break;
                            }

                            if j == 0 {
                                break;
                            }

                            j -= 1;
                        }
                    }

                    return Err(err.to_string());
                }
            }

            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn do_rename(src: &Path, dst: &Path) -> Result<(), io::Error> {
    // rename() raises io error iff:
    // 1. src does not exist in fs
    // 2. dst directory does not exist in fs
    match fs::rename(src, dst) {
        Ok(_res) => Ok(()), // successful rename, do nothing else
        Err(_err) => {
            // src does not exist in fs.
            if let Err(err) = fs::metadata(src) {
                return Err(err);
            }

            // dst directory does not exist.
            if let Some(parent) = dst.parent() {
                // Use create_dir_all() to recursively construct all parent
                // directories if they do not exist.
                //
                // Eg. parent(abc/def/ghi) -> abc/def
                // So directories abc & def are created.
                if let Err(err) = fs::create_dir_all(parent) {
                    return Err(err);
                }
            }

            // Try renaming again after creating directorie(s).
            fs::rename(src, dst)
        }
    }
}

/// Returns a vector of edges which represents the movement from
/// source to destination file/dir location.
///
/// It does so by detecting cycles (Eg. A -> B -> C -> A) and adding
/// an additional node (called tmp for example) to form this new graph,
/// A -> B -> C -> tmp -> A.
///
/// So when adding back the edges to the output vector, the edges are pushed
/// in reverse so that the files can be `moved` without overriding the contents
/// of other files.
fn build_renames(files: &HashMap<PathBuf, PathBuf>) -> Result<Vec<Edge>, String> {
    // Represents the reverse of files - where all edges are reversed.
    // Eg. A -> B becomes B -> A
    let mut rev = HashMap::<PathBuf, PathBuf>::new();

    // Stores similar to files except each src/dst are replaced with
    // canonicalize() paths.
    let mut file_map = HashMap::<PathBuf, PathBuf>::new();

    // Raise error if src/dst are repeated.
    // Also construct file_map and rev along on the way.
    for (src, dst) in files {
        if src.to_str() == Some("") || dst.to_str() == Some("") {
            return Err(EMPTY_PATH_ERROR.to_string());
        }

        let cleaned_src = clean(src);
        let cleaned_dst = clean(dst);

        if file_map.contains_key(&cleaned_src) {
            return Err(format!("Duplicate source {}", cleaned_src.display()));
        }

        if rev.contains_key(&cleaned_dst) {
            return Err(format!("Duplicate destination {}", cleaned_dst.display()));
        }

        file_map.insert(cleaned_src.clone(), cleaned_dst.clone());
        rev.insert(cleaned_dst, cleaned_src);
    }

    // Remove redundant mappings from both HashMap.
    file_map.retain(|src, dst| src != dst);
    rev.retain(|src, dst| src != dst);

    // Find cyclic groups
    let mut rs = Vec::<Edge>::new(); // return value
    let mut vs = HashMap::<&PathBuf, i32>::new();
    let mut cycle = false;
    let mut i = 0;

    for (_, mut dst) in &file_map {
        if let Some(&group_num) = vs.get(dst) {
            if group_num > 0 {
                // Skip nodes that were already checked.
                continue;
            }
        }

        i += 1;

        // Detect cycle
        while let Some(dst_dst) = file_map.get(dst) {
            vs.insert(&dst, i); // Set the group number to i.
            dst = dst_dst;
            if let Some(&group_num) = vs.get(dst_dst) {
                if group_num > 0 {
                    cycle = group_num == i;
                    break;
                }
            }
        }

        let mut tmp: PathBuf = PathBuf::new();
        if cycle {
            if let Some(path) = dst.parent() {
                tmp = random_path(path);
                rs.push(Edge {
                    src: dst.to_owned(),
                    dst: tmp.to_owned(),
                });
            }
            // Breaks the cycle (in later loop).
            //
            // Basically decrements one of the nodes
            // in the cycle so that `vs[src] == i`
            // returns false after going through all
            // the nodes in the cycle.
            *vs.get_mut(dst).unwrap() -= 1;
        }

        loop {
            if let Some(src) = rev.get(dst) {
                if !cycle || *vs.get(src).unwrap() == i {
                    rs.push(Edge {
                        src: src.to_owned(),
                        dst: dst.to_owned(),
                    });

                    if !cycle {
                        vs.insert(dst, i);
                    }

                    dst = src;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if cycle {
            // Insert last edge
            rs.push(Edge {
                dst: dst.to_owned(),
                src: tmp,
            });
        }
    }

    Ok(rs)
}

fn random_path(dir: &Path) -> PathBuf {
    // Keep running till a path string is generated
    // that does not exist in file system.
    loop {
        let new_path = dir.join(next_random());
        if let Err(_err) = fs::metadata(&new_path) {
            return new_path;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::hash::Hash;
    use std::io;
    use std::path::PathBuf;
    use std::sync::Once;

    use super::super::filepath::clean;
    use super::super::ioutils::temp_dir;
    use super::{build_renames, rename, EMPTY_PATH_ERROR};

    static SETUP: Once = Once::new();

    const TESTS_DIR: &'static str = "mmv-tests";

    type CaseInput<'a> = &'a [(&'a str, &'a str)];

    fn to_map<'a, K, V>(items: CaseInput<'a>) -> HashMap<K, V>
    where
        K: Eq + Hash + From<&'a str>,
        V: From<&'a str>,
    {
        items
            .iter()
            .map(|&(key, value)| (key.into(), value.into()))
            .collect()
    }

    struct TestCase<'a> {
        pub files: HashMap<PathBuf, PathBuf>,
        pub contents: HashMap<PathBuf, String>,
        pub expected: HashMap<PathBuf, String>,
        pub count: usize,
        pub err: Option<&'a str>,
    }

    impl<'a> TestCase<'a> {
        pub fn new(
            count: usize,
            files: CaseInput,
            contents: CaseInput,
            expected: CaseInput,
            err: Option<&'a str>,
        ) -> Self {
            TestCase {
                count,
                files: to_map::<PathBuf, PathBuf>(files),
                contents: to_map::<PathBuf, String>(contents),
                expected: to_map::<PathBuf, String>(expected),
                err,
            }
        }

        pub fn setup(&self, dir: &str) -> io::Result<()> {
            for (file, content) in &self.contents {
                let loc = PathBuf::from(dir).join(file);
                fs::write(loc, content)?;
            }

            Ok(())
        }

        pub fn file_contents(&self, dir: &str) -> io::Result<HashMap<PathBuf, String>> {
            let mut output_map = HashMap::<PathBuf, String>::new();
            for entry in fs::read_dir(dir)? {
                let pathbuf = entry?.path();
                if pathbuf.is_dir() {
                    // Read all files in this directory
                    for (path, contents) in &self.file_contents(pathbuf.to_str().unwrap())? {
                        output_map.insert(clean(path), contents.clone());
                    }
                } else {
                    // Write file contents to output map
                    let cleaned_path = clean(pathbuf);
                    let read_result = fs::read(&cleaned_path);
                    let contents = String::from_utf8(read_result?);
                    if contents.is_ok() {
                        output_map.insert(cleaned_path, contents.unwrap());
                    } else {
                        eprintln!("Failed to read contents from {:?}", cleaned_path);
                        break;
                    }
                }
            }

            Ok(output_map)
        }

        pub fn check(&self) {
            // Create a single common directory for all temporary files/directories.
            let tmp_path = env::temp_dir().join(TESTS_DIR);

            SETUP.call_once(|| {
                // If exists, ignore error.
                fs::create_dir(&tmp_path).ok();
            });

            // Get fully resolved path to temporary folder.
            // If no canoncalize, then will not resolve symbolic links.
            let tmp_path = tmp_path.canonicalize().unwrap();

            // Create another temporary folder (per test case) at that location.
            // NOTE: temp_dir() will try making a unique directory if such
            // already exists.
            let dir_path = temp_dir(tmp_path.to_str().unwrap(), "mmv-").unwrap();
            // Write contents to each file to individual tmp folder location.
            assert!(self.setup(&dir_path).is_ok());

            // Build renames
            let renames = build_renames(&self.files);
            if renames.is_ok() {
                let edges = renames.unwrap();
                assert!(edges.len() == self.count);
            }

            // Rename files in temporary folder path, not TEST_DIR path!
            if let Err(err) = rename(&self.files, Some(&dir_path)) {
                if let Some(expected_err) = self.err {
                    assert_eq!(err, expected_err.to_string());
                }
            }

            // Read all file contents inside dir_path and check with expected result.
            let got = self.file_contents(&dir_path);
            assert!(got.is_ok());

            let mut stripped_output_map = HashMap::<PathBuf, String>::new();
            // Extract prefix path from keys in output map.
            for (path, content) in got.unwrap() {
                stripped_output_map.insert(
                    PathBuf::from(path.strip_prefix(&dir_path).unwrap()),
                    content,
                );
            }

            assert!(stripped_output_map == self.expected);

            // Remove temporary dir holding test case files.
            assert!(fs::remove_dir_all(dir_path).is_ok());
        }
    }

    #[test]
    fn one_file() {
        TestCase::new(1, &[("foo", "bar")], &[("foo", "0")], &[("bar", "0")], None).check();
    }

    #[test]
    fn two_files() {
        TestCase::new(
            2,
            &[("foo", "qux"), ("bar", "quux")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2")],
            &[("qux", "0"), ("quux", "1"), ("baz", "2")],
            None,
        )
        .check();
    }

    #[test]
    fn swap_two_files() {
        TestCase::new(
            3,
            &[("foo", "bar"), ("bar", "foo")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2")],
            &[("bar", "0"), ("foo", "1"), ("baz", "2")],
            None,
        )
        .check();
    }

    #[test]
    fn two_swaps() {
        TestCase::new(
            6,
            &[
                ("foo", "bar"),
                ("bar", "foo"),
                ("baz", "qux"),
                ("qux", "baz"),
            ],
            &[("foo", "0"), ("bar", "1"), ("baz", "2"), ("qux", "3")],
            &[("bar", "0"), ("foo", "1"), ("baz", "3"), ("qux", "2")],
            None,
        )
        .check();
    }

    #[test]
    fn three_files() {
        TestCase::new(
            3,
            &[("foo", "bar"), ("bar", "baz"), ("baz", "qux")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2")],
            &[("bar", "0"), ("baz", "1"), ("qux", "2")],
            None,
        )
        .check();
    }

    #[test]
    fn cycle_three_files() {
        TestCase::new(
            4,
            &[("foo", "bar"), ("bar", "baz"), ("baz", "foo")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2")],
            &[("bar", "0"), ("baz", "1"), ("foo", "2")],
            None,
        )
        .check();
    }

    #[test]
    fn empty_source_path_error() {
        TestCase::new(
            0, // Does not matter
            &[("foo", "baz"), ("", "baz")],
            &[("foo", "0"), ("bar", "1")],
            &[("foo", "0"), ("bar", "1")],
            Some(EMPTY_PATH_ERROR),
        )
        .check();
    }

    #[test]
    fn empty_destination_path_error() {
        TestCase::new(
            0, // Does not matter
            &[("foo", "baz"), ("bar", "")],
            &[("foo", "0"), ("bar", "1")],
            &[("foo", "0"), ("bar", "1")],
            Some(EMPTY_PATH_ERROR),
        )
        .check();
    }

    #[test]
    fn same_destination_path_error() {
        TestCase::new(
            0, // Does not matter
            &[("foo", "baz"), ("bar", "baz"), ("baz", "qux")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2")],
            Some("Duplicate destination baz"),
        )
        .check();
    }

    #[test]
    fn clean_source_path() {
        TestCase::new(
            3,
            &[("foo", "bar"), ("bar/", "foo/")],
            &[("foo", "0"), ("bar", "1")],
            &[("foo", "1"), ("bar", "0")],
            None,
        )
        .check();
    }

    #[test]
    fn clean_path_same_source() {
        TestCase::new(
            0, // No matter
            &[("foo", "baz"), ("bar/../foo", "bar")],
            &[("foo", "0"), ("bar", "1")],
            &[("foo", "0"), ("bar", "1")],
            Some("Duplicate source foo"),
        )
        .check();
    }

    #[test]
    fn clean_path_same_destination() {
        TestCase::new(
            0, // No matter
            &[("foo", "baz"), ("bar", "foo/../baz")],
            &[("foo", "0"), ("bar", "1")],
            &[("foo", "0"), ("bar", "1")],
            Some("Duplicate destination baz"),
        )
        .check();
    }

    #[test]
    fn same_source_and_destination() {
        TestCase::new(
            0, // No matter
            &[("foo/", "foo"), ("bar/", "bar")],
            &[("foo", "0"), ("bar", "1")],
            &[("foo", "0"), ("bar", "1")],
            None,
        )
        .check();
    }

    #[test]
    fn same_destination_with_error() {
        TestCase::new(
            0, // No matter
            &[("foo/", "foo/"), ("bar/", "foo")],
            &[("foo", "0"), ("bar", "1")],
            &[("foo", "0"), ("bar", "1")],
            Some("Duplicate destination foo"),
        )
        .check();
    }

    #[test]
    fn undo_on_error() {
        TestCase::new(
            7,
            &[
                ("foo", "bar"),
                ("bar", "foo"),
                ("baz", "qux"),
                ("qux", "quux"),
                ("quux", "baz"),
            ],
            &[("foo", "0"), ("bar", "1"), ("baz", "2"), ("qux", "3")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2"), ("qux", "3")],
            Some("No such file or directory (os error 2)"), // No such file or directory
        )
        .check();
    }

    #[test]
    fn create_destination_directory() {
        TestCase::new(
            3,
            &[("foo", "x/foo"), ("bar", "x/bar"), ("baz", "a/b/c/baz")],
            &[("foo", "0"), ("bar", "1"), ("baz", "2")],
            &[("x/foo", "0"), ("x/bar", "1"), ("a/b/c/baz", "2")],
            None,
        )
        .check();
    }
}
