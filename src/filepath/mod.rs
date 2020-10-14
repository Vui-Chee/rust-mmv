//! Simplifed filepath module for cleaning input paths.
//!
//! This is a simplified smaller version of filepath
//! module used in Golang. For this use case, this will
//! only implement pathing cases for Unix systems.
//!
//! TODO
//! 1. Implement is_path_separator() for different os
//! 2. Implement from_slash()
//! 3. Implement clean(filepath: PathBuf)
//!

mod linux;
mod unix;
mod windows;

pub fn is_path_separator(c: char) -> bool {
    if cfg!(unix) {
        return unix::is_path_separator(c);
    } else if cfg!(linux) {
        return linux::is_path_separator(c);
    } else if cfg!(windows) {
        return windows::is_path_separator(c);
    }

    // None of the OS
    false
}
