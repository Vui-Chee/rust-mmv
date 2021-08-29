//! Temporary file creation module.
//!
//! Exposes temp_file() which allows a user to
//! create a temporary file using a custom prefix
//! which is followed by a randomly generated
//! 7-9 digit suffix.

use rand::prelude::*;

use std::env;
use std::fs::{create_dir, File, OpenOptions};
use std::io::{Error, ErrorKind, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// Returns a random 7-9 digit string.
pub fn next_random() -> String {
    let mut rng = rand::thread_rng();
    ((rng.gen::<f64>() * 1e18) % 1e9).to_string()
}

pub fn temp_file(dirname: &str, pattern: &str) -> Result<(File, String)> {
    let mut dir = PathBuf::from(dirname);
    if dirname.len() == 0 {
        dir = env::temp_dir();
    }

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
            .open(&filename);

        if !result.is_err() {
            tmp = result?;
            // Set read & write perms for owner of this file.
            let metadata = tmp.metadata()?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            tmp.set_permissions(permissions)?;

            return Ok((tmp, filename));
        }
    }

    // raise IO error
    Err(Error::new(ErrorKind::Other, "Failed to create temp file"))
}

#[allow(dead_code)]
pub fn temp_dir(dirname: &str, pattern: &str) -> Result<String> {
    let mut dir = PathBuf::from(dirname);
    if dirname.len() == 0 {
        dir = env::temp_dir();
    }

    for _i in 1..10000 {
        let rnd_dirname = format!("{}{}", pattern, next_random());
        let mut dirpath = dir.clone();
        dirpath.push(rnd_dirname);
        // Failed to create directory.
        if let Err(err) = create_dir(&dirpath) {
            if err.kind() == ErrorKind::AlreadyExists {
                // Continue creating a random directory.
                continue;
            } else {
                // Some other error.
                break;
            }
        }

        // Success directory created, create_dir() returns ().
        // Return string path for temporary directory.
        return Ok(String::from(dirpath.to_str().unwrap()));
    }

    Err(Error::new(
        ErrorKind::Other,
        "Failed to create temp directory",
    ))
}

#[cfg(test)]
mod tests {
    use super::temp_dir;
    use std::fs;

    #[test]
    fn create_temp_dir() {
        let result = temp_dir("", "mmv-");
        assert!(result.is_ok());
        let dirpath = result.unwrap();
        // Check directory exists.
        assert!(fs::canonicalize(&dirpath).is_ok());
        // Clean up
        assert!(fs::remove_dir(dirpath).is_ok());
    }
}
