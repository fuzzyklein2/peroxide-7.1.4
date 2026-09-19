use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error};
use std::env;
use std::path::{Path, PathBuf};

use crate::constants::{ BASE_DIR, FOLDER_PICT };
use crate::logging::{ error, log_file_name };
use crate::utilities::program_name;

/// # Returns
///
/// `path` to the process working directory.
/// ⚠️ This is not the directory containing the executable or the source file.
pub fn cwd() -> Result<PathBuf, Error> {
    match std::env::current_dir() {
        Ok(path) => Ok(path),
        Err(e) => {
            error("Couldn't get current working directory: {e}");
            Err(e)
        }
    }
}

pub fn pwd() {
    println!("{}Current working directory: {}",FOLDER_PICT, cwd().expect("REASON").display());
}

pub fn home() -> Option<PathBuf> {
    match env::home_dir() {
        Some(path) => Some(path),
        None => {
            error("Impossible to get your home dir!"); None
        }
    }
}

#[derive(Debug)]
pub struct FileSystem {
    pub config_file: PathBuf,
    pub log_file: PathBuf,
    // config_file: dirs::config_dir().unwrap().join("peroxide/config.json"),
    // data_file: BASE_DIR.join(format!("data/{}.json", program_name())),
    // log_file: home().join(format!(".logs/{}"))
}

impl FileSystem {
    pub fn new() -> Self {
        println!("Initializing file system");
        let config_dir = dirs::config_dir().unwrap().join("peroxide");
        let config = config_dir.join("config.json");
        println!("Getting home directory, building log directory");
        let logdir = home().unwrap()
                           .join(format!(".log/{}",
                                         program_name().expect("REASON")
                                        )
                                );
        println!("Checking configuration file");
        if !config.exists() {
            fs::create_dir_all(&config_dir);
            fs::write(&config, "{\n    \"saved_logs\" : 5\n}\n")
                .expect("Couldn't create default configuration file!")
        }
        println!("Checking log directory");
        if !logdir.exists() {
            fs::create_dir_all(&logdir);
        }

        Self {
            config_file: config,
            log_file: logdir.join(log_file_name()),
        }
    }
}



pub fn read_lines<P: AsRef<Path>>(path: P) -> Result<Vec<String>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader.lines().collect()
}
