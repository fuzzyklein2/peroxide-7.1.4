#![allow(warnings)]
use std::io::{ self, Error, Write, stdout };
use std::path::{Path, PathBuf};
use std::sync::{ mpsc, OnceLock };
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::{ execute, ExecutableCommand, QueueableCommand,
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode },
    cursor::{ MoveTo },
    style::{ Color, Print, PrintStyledContent, self, Stylize },
    event::{ self, Event, KeyCode }
};
use dirs;
use env_logger;
use json::JsonValue;
use log::LevelFilter;

mod config;
mod constants;
mod getargs;
mod files;
mod logging;
mod utilities;

mod peroxide;

use config::JSON;
use constants::{ BASE_DIR, FOLDER_PICT };
use files::{ cwd, FileSystem, home, pwd, read_lines };
use getargs::{ Args, get_piped_input };
use logging::{ error, warn, info, debug, trace, init_log };
use utilities::program_name;

use peroxide::get_most_recent_song_list;

static FILE_SYSTEM: OnceLock<FileSystem> = OnceLock::new();
static CONFIGURATION: OnceLock<JSON> = OnceLock::new();
static INPUT: OnceLock<String> = OnceLock::new();

static SONG_LIST: OnceLock<Vec<String>> = OnceLock::new();

fn main() -> Result<(), Error> {
    println!("Running the program");
    let prog_name = program_name()?;
    let args = Args::parse();
    FILE_SYSTEM.set(FileSystem::new()).unwrap();
    CONFIGURATION.set(JSON::new(&FILE_SYSTEM.get().unwrap().config_file)).unwrap();

    if let Some(input) = get_piped_input() {
        INPUT.set(input).unwrap();
    }

    let level = if args.trace {
        LevelFilter::Trace
    } else if args.debug {
        LevelFilter::Debug
    } else if args.verbose {
        LevelFilter::Info
    } else if args.warnings {
        LevelFilter::Warn
    } else {
        LevelFilter::Error
    };

    // env_logger::init();
    env_logger::Builder::new().filter_level(level).format(|buf, record| {
        writeln!(buf, "{} {}", record.level(), record.args())
    }).init();

    init_log()?;

    info(&format!("Running {}", prog_name));
    warn("This program is under construction!");
    debug(&format!("Debugging {}", prog_name));
    trace(&format!("Tracing {}", prog_name));
    // error("Danger, Will Robinson!");
    // pwd();
    let s = cwd().unwrap();
    debug(&format!("Current working directory: {}", s.display()));
    let s = BASE_DIR.display();
    debug(&format!("Base directory: {s}"));
    let home = home().unwrap();
    debug(&format!("User home directory: {}", home.display()));
    debug(&format!("Arguments: {:#?}", args.args));


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

    let nargs = args.args.len();
    let mut song_list = Vec::<String>::new();
    if let Some(input) = INPUT.get() {
        song_list = input.lines().map(str::to_owned).collect();;
    } else if nargs > 0 {
        song_list = read_lines(&args.args[0])?;
    } else {
        song_list = get_most_recent_song_list()?;
    }
    while song_list.last().is_some_and(|s| s.is_empty()) {
        song_list.pop();
    }
    SONG_LIST.set(song_list);
    trace(&format!("Songs: {:#?}", SONG_LIST.get().unwrap()));

/*
    let status_label = "Status".with(Color::Cyan);

    execute!( stdout(),
              Clear(ClearType::All),
              MoveTo(1, 1),
              Print("🌿  PEROXIDE"),
              MoveTo(1, 3),
              PrintStyledContent(status_label),
              Print(": Stopped"),
            )?;

    println!("\n");

    enable_raw_mode()?;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap() {
                if let Ok(Event::Key(key)) = event::read() {
                    if tx.send(key).is_err() {
                        break;
                    }
                }
            }
        }
    });



    let mut running = true;
    let mut playing = false;
    let mut status = "Stopped";

    while running {
        if let Ok(key) = rx.try_recv() {
            match key.code {
                KeyCode::Char(' ') => {
                    playing = !playing;
                    if (playing) { status = "Playing"; }
                    else { status = "Stopped"; }
                    execute!(stdout(),
                        MoveTo(9, 3),
                        Print(&status)
                    )?;
                }

                KeyCode::Esc => {
                    running = false;
                }

                _ => {}
            }
        }

        // Do whatever else the main thread needs to do...
    }


    disable_raw_mode()?;
    println!("\n");
*/

    Ok(())
}
