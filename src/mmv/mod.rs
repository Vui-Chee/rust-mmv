use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::ioutils::next_random;

pub fn rename(files: HashMap<PathBuf, PathBuf>) {
    match build_renames(files) {
        Ok(()) => (),
        Err(msg) => eprintln!("{}", msg),
    };
}

pub fn build_renames(files: HashMap<PathBuf, PathBuf>) -> Result<(), String> {
    // Represents the reverse of files - where all edges are reversed.
    // Eg. A -> B becomes B -> A
    let mut rev = HashMap::<PathBuf, PathBuf>::new();

    // Stores similar to files except each src/dst are replaced with
    // canonicalize() paths.
    let mut file_map = HashMap::<PathBuf, PathBuf>::new();

    // Raise error if src/dst are repeated.
    // Also construct file_map and rev along on the way.
    for (src, dst) in files {
        if let Ok(expanded_src) = src.canonicalize() {
            if file_map.contains_key(&expanded_src) {
                return Err(format!("Duplicate source {:?}", src));
            }

            if let Ok(expanded_dst) = dst.canonicalize() {
                if rev.contains_key(&expanded_dst) {
                    return Err(format!("Duplicate destination {:?}", dst));
                }

                file_map.insert(expanded_src.clone(), expanded_dst.clone());
                rev.insert(expanded_dst, expanded_src);
            } else {
                return Err(format!("No such file/directory {:?}", dst));
            }
        } else {
            return Err(format!("No such file/directory {:?}", src));
        }
    }

    // Remove redundant mappings from both HashMap.
    file_map.retain(|src, dst| src != dst);
    rev.retain(|src, dst| src != dst);

    // Find cyclic groups
    let mut vs = HashMap::<&PathBuf, i32>::new();
    let mut cycle = false;
    let mut i = 0;

    for (_, mut dst) in &file_map {
        if let Some(&group_num) = vs.get(dst) {
            if group_num > 0 {
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

        if cycle {
            println!("Yah detected a cycle!");
            let tmp = random_path(dst.parent().unwrap());
            println!("Random tmp {}", tmp);
        }
    }

    Ok(())
}

fn random_path(dir: &Path) -> String {
    // Keep running till a path string is generated
    // that does not exist in file system.
    loop {
        let new_path = dir.join(next_random());
        if let Err(_err) = fs::metadata(&new_path) {
            return new_path.to_str().unwrap().to_owned();
        }
    }
}
