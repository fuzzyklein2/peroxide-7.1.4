use std::fs;
use std::io::Error;
use std::path::PathBuf;

use clap::Parser;
use json::JsonValue;

use crate::FILE_SYSTEM;
use crate::CONFIGURATION;
use crate::INPUT;
use crate::ARGUMENTS;

use crate::files::FileSystem;
use crate::getargs::{ Args, get_piped_input };
use crate::utilities::program_name;

#[derive(Debug)]
pub struct JSON {
    pub value: JsonValue,
}

impl JSON {
    pub fn new(p: &PathBuf) -> Self {
        let contents = fs::read_to_string(p).unwrap();

        Self {
            value: json::parse(&contents).unwrap(),
        }
    }
}

pub fn configure() -> Result<(), Error> {
    let prog_name = program_name()?;
    let args = Args::parse();
    FILE_SYSTEM.set(FileSystem::new()).unwrap();
    CONFIGURATION.set(JSON::new(&FILE_SYSTEM.get().unwrap().config_file)).unwrap();
    ARGUMENTS.set(args).unwrap();
    
    if let Some(input) = get_piped_input() {
        INPUT.set(input).unwrap();
    }
    Ok(())
}