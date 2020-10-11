extern crate clap;

mod ioutils;
mod mmv;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{read_to_string, remove_file};
use std::io::{Result, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use clap::{App, Arg, Values};

static APP_NAME: &'static str = "mmv";

fn main() -> Result<()> {
    let file_args = Arg::new("files")
        .about("Files to rename")
        .required(true)
        .multiple(true);

    let matches = App::new("Rust mmv")
        .version("1.0")
        .about("Does awesome things")
        .arg(&file_args)
        .get_matches();

    let file_inputs: Option<Values> = matches.values_of(file_args.get_name());
    if let Some(files) = file_inputs {
        run(files)?;
    }

    Ok(())
}

pub fn run(files: Values) -> Result<()> {
    // Check for duplicate paths
    let original_len = files.len();
    let unique_paths: HashSet<_> = files.collect();
    if unique_paths.len() != original_len {
        eprintln!("Duplicate source(s)");
        return Ok(());
    }

    // Create temporary file
    let tmp_filename_prefix = format!("{}{}", APP_NAME, "-");
    let (mut tmp, tmp_file_path) = ioutils::temp_file("", &tmp_filename_prefix)?;
    for path in &unique_paths {
        let path_with_newline = format!("{}\n", path);
        tmp.write(path_with_newline.as_bytes())?;
    }

    // Read EDITOR env
    let default_editor = String::from("vi");
    let mut editor = env::var("EDITOR").unwrap_or(default_editor.to_owned());
    if editor.len() == 0 {
        editor = default_editor;
    }

    // Separate editor command from its args.
    let fields: Vec<&str> = editor.splitn(2, " ").collect();
    let mut args = Vec::<&str>::new();
    if fields.len() > 1 {
        args = fields[1].split_whitespace().collect();
    }
    args.push(&tmp_file_path);

    // Create and execute command.
    if let Err(cmd_err) = Command::new(fields[0]) // First item is editor command
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        // executes command
        .output()
    {
        // Executing command has errors.
        eprintln!("{}", cmd_err);
    }

    // Read destination paths from tmp file.
    // (Happens after user updates paths with editor)
    //
    // PathBuf is used to pass ownership from main() into rename().
    // After this the paths data is no longer needed.
    let mut src_to_dst_map = HashMap::<PathBuf, PathBuf>::new();
    let contents = read_to_string(&tmp_file_path)?;
    contents
        .trim_end_matches("\n")
        .split("\n")
        .zip(unique_paths.iter())
        .for_each(|(dst, src)| {
            src_to_dst_map.insert(PathBuf::from(src), PathBuf::from(dst));
        });
    mmv::rename(src_to_dst_map);

    // Remove tmp file whether or not cmd succeeds or fails.
    remove_file(tmp_file_path)?;

    Ok(())
}
