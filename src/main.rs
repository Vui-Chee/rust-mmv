extern crate clap;

mod ioutils;

use std::collections::HashMap;
use std::io::Result;

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
    // Check for duplicate files
    let mut set_files = HashMap::<&str, bool>::new();
    for file in files {
        if set_files.contains_key(file) {
            eprintln!("Duplicate source {}", file);
            return Ok(());
        }

        set_files.insert(file, true);
    }

    // Create temporary file
    // let fileprefix = "mmv-";
    let mut tmp = ioutils::temp_file("", "mmv-")?;

    // for file in set_files.keys() {
    // let file_with_newline = format!("{}\n", file);
    // let bytes_read = tmp.write(file_with_newline.as_bytes())?;
    // println!("Read {} bytes from {}", bytes_read, file);
    // }

    // Remove tmp file.
    // NOTE: the file is automatically closed when out of scope.
    // remove_file(dir)?;

    Ok(())
}
