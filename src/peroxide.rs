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
use midir::{ Ignore, MidiInput };

use crate::{ ARGUMENTS, CONFIGURATION, INPUT };
use crate::files;
use crate::files::{ read_lines };
// use crate::ARGUMENTS;
use crate::logging::{ error, warn, info, debug, trace, init_log };

static SONG_LIST: OnceLock<Vec<String>> = OnceLock::new();

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

    pub fn print(&mut self) {
        print!("{}", self.text);
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
    let arguments = &ARGUMENTS.get().unwrap().args;
    let nargs = arguments.len();
    let mut song_list = Vec::<String>::new();
    if let Some(input) = INPUT.get() {
        song_list = input.lines().map(str::to_owned).collect();;
    } else if nargs > 0 {
        song_list = read_lines(&arguments[0])?;
    } else {
        song_list = get_most_recent_song_list()?;
    }
    while song_list.last().is_some_and(|s| s.is_empty()) {
        song_list.pop();
    }
    SONG_LIST.set(song_list);
    trace(&format!("Songs: {:#?}", SONG_LIST.get().unwrap()));

    /*

    let status_label_text = "Status".to_owned();
    let title_pos = Point::new(1, 1);
    let mut title_label = Label::new(title_pos, "🌿  PEROXIDE".to_owned());
    let status_label_pos = Point::new(1, 3);
    let mut status_label = Label::with_color(status_label_pos, status_label_text, Color::Cyan);
    let status_text_pos = Point::new(9, 3);
    let mut status_text = Label::new(status_text_pos, "Stopped".to_owned());
    
    // execute!( stdout(),
    //           Clear(ClearType::All),
    //           // MoveTo(1, 3),
    //           // PrintStyledContent(status_label),
    //           // Print(": Stopped"),
    //           // MoveTo(1, 5),
    //           // Print("")
    //         )?;
    
    title_label.refresh()?;
    status_label.refresh()?;

    execute!( stdout(),
              Print(":"),
            )?;

    status_text.refresh()?;

    println!("\n");

    */
        
    // enable_raw_mode()?;
    let (tx, rx) = mpsc::channel();
    let (midi_tx, midi_rx) = mpsc::channel();

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


    let midi = MidiInput::new("peroxide")
        .expect("Couldn't initialize MIDI");
    let ports = midi.ports();
    println!("MIDI inputs: {}", ports.len());
    
    for (i, port) in ports.iter().enumerate() {
        println!("{}: {}", i, midi.port_name(port).unwrap());
    }

    let port = ports.get(1).expect("MIDI port 1 doesn't exist.");
    
    let _connection = midi.connect(
        port,
        "peroxide-input",
        move |_timestamp, message, _| {
            match message {
                [0xF8] | [0xFE] => {}
    
                [0xB0, 64, value] if *value > 63 => {
                    let _ = midi_tx.send(true);
                }
    
                _ => {
                    println!("MIDI: {:?}", message);
                }
            }
        },
        (),
    ).expect("Couldn't open MIDI port 1");
    
    println!("Successfully opened MIDI port 1");
        
    let mut running = true;
    let mut playing = false;
    let mut status = "Stopped";
    let mut pedal_down = false;
    let mut stop_endless_repeat = true;
 
    while running {
        if let Ok(key) = rx.try_recv() {
            match key.code {
                KeyCode::Char(' ') => {
                    playing = !playing;
                    // if (playing) { status_text.set("Playing".to_owned()); }
                    // else { status_text.set("Stopped".to_owned()); }
                    // status_text.refresh()?;
                }
                /// **TODO:** This should escape the current song, rather than quit the program.
                KeyCode::Esc => {
                    running = false;
                }

                _ => {}
            }
        }

        if let Ok(pedal_up) = midi_rx.try_recv() {
            if pedal_up {
                stop_endless_repeat = true;
            }
        }

        // Do whatever else the main thread needs to do...
    }


    // disable_raw_mode()?;
    println!("\n");
    Ok(())
}
