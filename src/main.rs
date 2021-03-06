extern crate clap;

mod filepath;
mod ioutils;
mod macros;
mod mmv;

// Default modules
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{read_to_string, remove_file};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

// External modules
use clap::{App, Arg, Values};

static APP_NAME: &'static str = "mmv";

fn main() -> Result<(), String> {
    let file_args = Arg::new("files")
        .about("Files to rename")
        .required(true)
        .multiple(true);

    let matches = App::new("Rust mmv")
        .version("1.0")
        .about("Performs `mv` on multiple paths.")
        .arg(&file_args)
        .get_matches();

    let file_inputs: Option<Values> = matches.values_of(file_args.get_name());
    if let Some(file_values) = file_inputs {
        // Values is an iterator and will be consumed immediately
        // in any loop. So create a vector of paths for multiple
        // borrows.
        let files = file_values.into_iter().collect::<Vec<&str>>();
        run(files).unwrap_or_else(|msg| {
            eprintln!("{}", msg);
        });
    }

    Ok(())
}

pub fn run(files: Vec<&str>) -> Result<(), String> {
    // Check for duplicate paths
    let mut check_paths = HashSet::<&str>::new();
    for file in &files {
        let res = check_paths.insert(file);
        if !res {
            return Err(format!("Duplicate source {}", file));
        }
    }

    // Create temporary file
    let tmp_filename_prefix = format!("{}{}", APP_NAME, "-");
    let (mut tmp, tmp_file_path) = ioutils::temp_file("", &tmp_filename_prefix).unwrap();
    // Remove tmp file after run().
    defer!(remove_file(&tmp_file_path).unwrap_or_else(|msg| {
        eprintln!("Error removing tmp file:\n{}", msg);
    }));
    for path in &files {
        let path_with_newline = format!("{}\n", path);
        tmp.write(path_with_newline.as_bytes()).unwrap();
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
        return Err(format!("Error executing command:\n{}", cmd_err.to_string()));
    }

    // Read destination paths from tmp file.
    // (Happens after user updates paths with editor)
    //
    // PathBuf is used to pass ownership from main() into rename().
    // After this the paths data is no longer needed.
    let mut src_to_dst_map = HashMap::<PathBuf, PathBuf>::new();
    let contents = read_to_string(&tmp_file_path).unwrap();
    let edited_lines: Vec<&str> = contents
        .trim_end_matches("\n")
        .split_terminator("\n") // Normal split can result in [""]
        .filter(|path| (*path).replace(" ", "").len() > 0) // Clear empty paths
        .collect();

    // Raise error when user add/deletes a line from tmp file.
    if edited_lines.len() != files.len() {
        return Err(String::from("Do not add or delete lines."));
    }

    edited_lines
        .iter()
        .zip(files.into_iter())
        .for_each(|(dst, src)| {
            src_to_dst_map.insert(PathBuf::from(src), PathBuf::from(dst));
        });
    mmv::rename(&src_to_dst_map, None)?;

    Ok(())
}
