use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;

// For this use case, there is no sharing among
// threads - so ARC not used.
lazy_static! {
    static ref RAND: Mutex<u32> = Mutex::new(0);
}

pub fn reseed() -> u32 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    let pid = process::id();

    (since_the_epoch + pid as u128) as u32
}

/// Returns a random 9-digit string.
pub fn next_random() -> String {
    let mut rand = RAND.lock().unwrap();
    if *rand == 0 {
        *rand = reseed();
    }
    // Raise to u64 to avoid overflow issues
    let r: u64 = (*rand as u64) * 1664525 + 1013904223; // constants from Numerical Recipes

    // Lock will be unlocked when rand goes out of scope.
    ((r as f64) % 1e9).to_string()
}

pub fn temp_file(dirname: &str, pattern: &str) -> Result<File> {
    let mut dir = PathBuf::from(dirname);
    if dirname.len() == 0 {
        dir = env::temp_dir();
    }

    let mut n_conflict = 0;
    let tmp: File;
    for _i in 1..10000 {
        let mut filename = "".to_owned();
        filename.push_str(dir.to_str().unwrap());
        filename.push_str(pattern);
        // Append random suffix to pattern
        filename.push_str(&next_random());

        // Creates new file if it does not exist.
        // Raises error if file already exists.
        let result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(filename);

        if result.is_err() {
            n_conflict += 1;
            if n_conflict > 10 {
                let mut rand = RAND.lock().unwrap();
                *rand = reseed();
                // Once out of this scope, lock should be released.
            }
        } else {
            tmp = result?;
            // Set read & write perms for owner of this file.
            let metadata = tmp.metadata()?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            tmp.set_permissions(permissions)?;
            return Ok(tmp);
        }
    }

    // raise IO error
    panic!("Failed to create temporary file.")
}
