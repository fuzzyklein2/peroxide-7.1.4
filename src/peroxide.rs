use std::fs;
use std::io::{ self, Error, Write, stdout };
use std::path::{ PathBuf };
use std::sync::{ mpsc, OnceLock };
use std::thread;
use std::time::Duration;

use crossterm::{ execute, ExecutableCommand, QueueableCommand,
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode },
    cursor::{ MoveTo },
    style::{ Color, Print, PrintStyledContent, self, Stylize },
    event::{ self, Event, KeyCode }
};

use json;
use maudio;

use crate::CONFIGURATION;
use crate::files;
use crate::logging::{ error, warn, info, debug, trace, init_log };

pub fn get_most_recent_song_list() -> Result<Vec<String>, io::Error> {
    let session_dir = PathBuf::from(CONFIGURATION.get().unwrap().value["session folder"].to_string());
    let lists_dir = session_dir.join("lists");
    let mut dir_content: Vec<_> = fs::read_dir(lists_dir)?
        .collect::<Result<Vec<_>, _>>()?;
    let n = dir_content.len();
    let i = n - 1;
    dir_content.sort_by_key(|entry| entry.file_name());
    let song_list_file = &dir_content[i];
    files::read_lines(song_list_file.path())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub struct Label {
    pub P: Point,
    pub w: i32,
}

pub fn run() -> Result<(), Error> {
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
    Ok(())
}

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
