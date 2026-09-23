//! Define helper functions.
use std::env::current_exe;
use std::fs;
use std::io::Error;

use crate::FILE_SYSTEM;

pub fn program_name() -> Result<String, Error> {
    let exe_path = current_exe()?;

    let name = exe_path
        .file_stem()
        .ok_or_else(|| Error::other("could not determine program name"))?
        .to_string_lossy();

    Ok(name.split_once('-')
        .map_or(name.as_ref(), |(name, _)| name)
        .to_string())
}

pub fn rotate_log_files() -> Result<(), Box<dyn std::error::Error>> {
    println!("ROTATE_LOG_FILES: not yet implemented");
    let log_dir = FILE_SYSTEM.get().unwrap().log_file.parent().unwrap();
    println!("Log directory: {}", log_dir.to_str().unwrap());

    let mut files: Vec<_> = fs::read_dir(log_dir)?
        .collect::<Result<Vec<_>, _>>()?;

    files.sort_by_key(|entry| entry.file_name());

    for file in &files[0..(files.len()-5)] {
        fs::remove_file(file.path())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_program_name() {
        assert!(program_name().starts_with("hello"));
    }

}
