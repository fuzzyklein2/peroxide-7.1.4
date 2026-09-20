use std::fs;
use std::io::{ self, Error, Write, stdout };
use std::path::{ PathBuf };
use std::sync::{ mpsc, OnceLock };
use std::thread;
use std::time::Duration;

use crossterm::{ execute, ExecutableCommand, QueueableCommand,
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode },
    cursor::{ MoveTo },
    style::{ Color, Print, PrintStyledContent, self, StyledContent, Stylize },
    event::{ self, Event, KeyCode }
};

use json;
use maudio;

use crate::CONFIGURATION;
use crate::files;
use crate::logging::{ error, warn, info, debug, trace, init_log };

pub fn get_most_recent_song_list() -> Result<Vec<String>, io::Error> {
    let session_dir = PathBuf::from(CONFIGURATION.get()
                                                 .unwrap()
                                                 .value["session folder"]
                                                 .to_string());
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
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

pub trait Refresh {
    fn refresh(&mut self) -> Result<(), Error>;
}

pub struct Label {
    pub position: Point,
    pub width: usize,
    pub text: String,
    pub color: Color,
}

impl Label {
    pub fn new(p: Point, s: String) -> Self {
        let w = s.len();
        Self { position: p, width: w, text: s, color: Color::White }
    }

    pub fn with_color(p: Point, s: String, c: Color) -> Self {
        let w = s.len();
        Self { position: p, width: w, text: s, color: c }
    }

    pub fn erase(&mut self) -> Result<(), Error> {
        self.text = " ".repeat(self.width);
        self.refresh()?;
        Ok(())
    }

    pub fn set(&mut self, s: String) -> Result<(), Error> {
        self.erase();
        self.text = s;
        self.refresh()?;
        Ok(())
    }
}

impl Refresh for Label {
    fn refresh(&mut self) -> Result<(), Error> {
        // TODO: refresh the label
        execute!( stdout(),
                  MoveTo(self.position.x, self.position.y),
                  PrintStyledContent(self.text.clone().with(self.color)),
                )?;
        Ok(())
    }
}

pub fn run() -> Result<(), Error> {
    let status_label_text = "Status".to_owned();
    let title_pos = Point::new(1, 1);
    let mut title_label = Label::new(title_pos, "🌿  PEROXIDE".to_owned());
    let status_label_pos = Point::new(1, 3);
    let mut status_label = Label::with_color(status_label_pos, status_label_text, Color::Cyan);
    let status_pos = Point::new(9, 3);
    let mut status = Label::new(status_pos, "Stopped".to_owned());
    
    execute!( stdout(),
              Clear(ClearType::All),
              // MoveTo(1, 3),
              // PrintStyledContent(status_label),
              // Print(": Stopped"),
              // MoveTo(1, 5),
              // Print("")
            )?;
    
    title_label.refresh()?;
    status_label.refresh()?;

    execute!( stdout(),
              Print(":"),
            )?;

    status.refresh()?;

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
