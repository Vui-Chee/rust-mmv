use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::filepath::clean;
use super::ioutils::next_random;

#[derive(Debug)]
pub struct Edge {
    pub src: PathBuf,
    pub dst: PathBuf,
}

pub fn rename(files: HashMap<PathBuf, PathBuf>) -> Result<(), String> {
    match build_renames(files) {
        Ok(renames) => {
            for (i, rename) in renames.iter().enumerate() {
                if let Err(err) = do_rename(rename.src.as_path(), rename.dst.as_path()) {
                    eprintln!("{}", err.to_string());

                    // Only undo if there is more than 1 previous renames.
                    // Otherwise, j - 1 yields an overflow error (since i is usize).
                    if i >= 1 {
                        let mut j = i - 1;
                        loop {
                            // Undo on error not to leave the temporary files.
                            // This does not undo directory creation.
                            if let Err(_err) =
                                fs::rename(renames[j].dst.as_path(), renames[j].src.as_path())
                            {
                                break;
                            }

                            if j == 0 {
                                break;
                            }

                            j -= 1;
                        }
                    }
                }
            }

            Ok(())
        }
        Err(err) => Err(err.to_string()),
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
fn build_renames(files: HashMap<PathBuf, PathBuf>) -> Result<Vec<Edge>, String> {
    // Represents the reverse of files - where all edges are reversed.
    // Eg. A -> B becomes B -> A
    let mut rev = HashMap::<PathBuf, PathBuf>::new();

    // Stores similar to files except each src/dst are replaced with
    // canonicalize() paths.
    let mut file_map = HashMap::<PathBuf, PathBuf>::new();

    // Raise error if src/dst are repeated.
    // Also construct file_map and rev along on the way.
    for (src, dst) in files {
        let cleaned_src = clean(src.as_path());
        let cleaned_dst = clean(dst.as_path());

        if file_map.contains_key(&cleaned_src) {
            return Err(format!("Duplicate source {:?}", src));
        }

        if rev.contains_key(&cleaned_dst) {
            return Err(format!("Duplicate destination {:?}", dst));
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

    use super::super::ioutils::temp_dir;
    use std::collections::HashMap;
    use std::fs;
    use std::hash::Hash;
    use std::path::PathBuf;

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

    struct TestCase {
        pub name: String,
        pub files: HashMap<PathBuf, PathBuf>,
        pub contents: HashMap<PathBuf, String>,
        pub expected: HashMap<PathBuf, String>,
        pub count: u32,
    }

    impl TestCase {
        pub fn new(
            name: &str,
            count: u32,
            files: CaseInput,
            contents: CaseInput,
            expected: CaseInput,
        ) -> Self {
            TestCase {
                name: String::from(name),
                count,
                files: to_map::<PathBuf, PathBuf>(files),
                contents: to_map::<PathBuf, String>(contents),
                expected: to_map::<PathBuf, String>(expected),
            }
        }
    }

    #[test]
    fn one_file() {
        let _tc = TestCase::new(
            "one file",
            1,
            &[("foo", "bar")],
            &[("foo", "0")],
            &[("bar", "0")],
        );

        let result = temp_dir("", "mmv-");
        assert!(result.is_ok());
        let dirname = result.unwrap();
        // Remove temp dir.
        assert!(fs::remove_dir(dirname).is_ok());
    }
}
