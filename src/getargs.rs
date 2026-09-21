use clap::Parser;
use std::io::{ self, IsTerminal, Read };

#[derive(Debug)]
#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Enable debug logging
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,
    /// Enable verbose logging
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
    /// Enable warnings
    #[arg(short = 'w', long = "warn")]
    pub warnings: bool,
    /// Enable trace logging
    #[arg(short = 't', long = "trace")]
    pub trace: bool,
    /// Other arguments
    pub args: Vec<String>,
}

pub fn get_piped_input() -> Option<String> {
    if !io::stdin().is_terminal() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap();

        if input.ends_with('\n') {
            input.pop();
        }

        Some(input)
    } else {
        None
    }
}