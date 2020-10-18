use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::filepath::clean;
use super::ioutils::next_random;

#[derive(Debug)]
pub struct Edge {
    pub src: PathBuf,
    pub dst: PathBuf,
}

pub fn rename(files: HashMap<PathBuf, PathBuf>) {
    match build_renames(files) {
        Ok(renames) => println!("{:?}", renames),
        Err(msg) => eprintln!("{}", msg),
    };
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
pub fn build_renames(files: HashMap<PathBuf, PathBuf>) -> Result<Vec<Edge>, String> {
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
