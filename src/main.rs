#![allow(warnings)]
use std::io::{ self, Error, Write, stdout };
use std::path::{Path, PathBuf};
use std::sync::{ mpsc, OnceLock };
use std::thread;
use std::time::Duration;

// use clap::Parser;
use crossterm::{ execute, ExecutableCommand, QueueableCommand,
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode },
    cursor::{ MoveTo },
    style::{ Color, Print, PrintStyledContent, self, Stylize },
    event::{ self, Event, KeyCode }
};
use dirs;
use json::JsonValue;
use log::LevelFilter;

mod config;
mod constants;
mod getargs;
mod files;
mod logging;
mod utilities;

mod peroxide;

use config::{ configure, JSON };
use constants::{ BASE_DIR, FOLDER_PICT };
use files::{ cwd, FileSystem, home, pwd, read_lines };
use getargs::{ Args, get_piped_input };
use logging::{ error, warn, info, debug, trace, init_log };
use utilities::program_name;

use peroxide::get_most_recent_song_list;

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
