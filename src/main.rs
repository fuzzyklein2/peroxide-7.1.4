#![allow(warnings)]
use std::io::{ Error };
use std::sync::{ OnceLock };

mod config;
mod constants;
mod files;
mod getargs;
mod logging;
mod utilities;

use config::{ configure, JSON };
use constants::{ BASE_DIR };
use files::{ cwd, FileSystem, home };
use getargs::{ Args };
use logging::{ debug, init_log };

mod peroxide;

static FILE_SYSTEM: OnceLock<FileSystem> = OnceLock::new();
static CONFIGURATION: OnceLock<JSON> = OnceLock::new();
static INPUT: OnceLock<String> = OnceLock::new();
static ARGUMENTS: OnceLock<Args> = OnceLock::new();

fn main() -> Result<(), Error> {
    configure()?;
    init_log()?;

    let s = cwd().unwrap();
    debug(&format!("Current working directory: {}", s.display()));
    let s = BASE_DIR.display();
    debug(&format!("Base directory: {s}"));
    let home = home().unwrap();
    debug(&format!("User home directory: {}", home.display()));
    debug(&format!("Arguments: {:#?}", ARGUMENTS.get().unwrap().args));


    debug(&format!(r#"File System:
Configuration file: {}
Log file:           {}
"#, FILE_SYSTEM.get().unwrap().config_file.display(),
    FILE_SYSTEM.get().unwrap().log_file.display()
));

    debug(&format!(r#"Configuration:
{}
"#, json::stringify_pretty(CONFIGURATION.get().unwrap().value.clone(), 4)
));
    


    peroxide::run()?;
    Ok(())
}
