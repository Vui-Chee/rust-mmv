use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn temp_file(dirname: &str) -> Result<File> {
    let mut dir = PathBuf::from(dirname);
    if dirname.len() == 0 {
        dir = env::temp_dir();
    }
    dir.push("tmp-file");

    // Creates new file if it does not exist.
    // Raises error if file already exists.
    let tmp = OpenOptions::new().write(true).create_new(true).open(dir)?;
    // Set read & write perms for owner of this file.
    let metadata = tmp.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    tmp.set_permissions(permissions)?;

    Ok(tmp)
}
