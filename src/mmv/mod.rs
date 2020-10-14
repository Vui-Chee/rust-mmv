use std::collections::HashMap;
use std::path::PathBuf;

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
            }
        }
    }

    Ok(())
}
