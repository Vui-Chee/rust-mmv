extern crate clap;

mod ioutils;

use std::collections::HashMap;
use std::env;
use std::fs::remove_file;
use std::io::{Result, Write};
use std::process::{Command, Stdio};
use std::str;

use clap::{App, Arg, Values};

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
    let mut set_paths = HashMap::<&str, bool>::new();
    for file in files {
        if set_paths.contains_key(file) {
            eprintln!("Duplicate source {}", file);
            return Ok(());
        }

        set_paths.insert(file, true);
    }

    // Create temporary file
    let (mut tmp, file_path) = ioutils::temp_file("", "mmv-")?;
    for path in set_paths.keys() {
        let path_with_newline = format!("{}\n", path);
        tmp.write(path_with_newline.as_bytes())?;
    }

    // Read EDITOR env and execute command
    let mut editor = env::var("EDITOR").unwrap();
    if editor.len() == 0 {
        editor = String::from("vi");
    }

    Command::new(editor)
        .args(&[&file_path])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("failed to execute process");

    // Remove tmp file.
    remove_file(file_path)?;

    Ok(())
}
