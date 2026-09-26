//! # 
// TODO: Get this file to run and run it with -h ==> SYNOPSIS
// TODO: Write some unit tests
// #[cfg(test)]
// mod tests;

use std::cmp::max;
use std::collections::{ HashMap, VecDeque };
use std::ffi::{ CString, OsString };
use std::fs;
use std::io::{ self, Error, ErrorKind, stdout };
use std::path::{ Path, PathBuf };
use std::sync::{ mpsc::{ channel, Sender, Receiver }, Mutex, OnceLock };
use std::thread;
use std::time::Duration;

use crossterm::{ execute, ExecutableCommand, QueueableCommand,
    event::{ KeyEvent },
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode },
    cursor::{ MoveTo },
    style::{ Color, Print, PrintStyledContent, self, StyledContent, Stylize },
    event::{ self, Event, KeyCode }
};

use json::{ JsonValue };
use maudio;
use midir::{ MidiInput, MidiInputConnection };

use crate::{ ARGUMENTS, CONFIGURATION, INPUT };
use crate::files;
use crate::files::{ read_lines };
// use crate::ARGUMENTS;
use crate::logging::{ error, warn, info, debug, trace, init_log };

pub const SONGS_DIR_NAME: &str = "songs";
pub const CLIPS_DIR_NAME: &str = "clips";
pub const SONG_FILE_NAME: &str = "song.json";

pub fn session_folder() -> Result<PathBuf, io::Error> {
    Ok(PathBuf::from(CONFIGURATION.get()
                                  .unwrap()
                                  .value["session folder"]
                                  .to_string()))
}

pub fn get_most_recent_song_list() -> Result<Vec<String>, io::Error> {
    let lists_dir = session_folder()?.join("lists");
    let mut dir_content: Vec<_> = fs::read_dir(lists_dir)?
        .collect::<Result<Vec<_>, _>>()?;
    // TODO: Find or write a function to determine if a directory is empty.
    if dir_content.is_empty() {
        error("Song lists directory is empty!");
    }
    let n = dir_content.len();
    let i = n - 1;
    dir_content.sort_by_key(|entry| entry.file_name());
    let song_list_file = &dir_content[i];
    files::read_lines(song_list_file.path())
}

trait FromFile: Sized {
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized;
}

trait FromString: Sized {
    fn from_string(s: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

trait FromJsonValue: Sized {
    fn from_json_value(js: &JsonValue) -> Result<Self, Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct Clip {
    samples: Vec<f32>,
    frames: usize,
    channels: u32,
    sample_rate: u32,
}

#[repr(C)]
pub struct AudioInfo {
    pub channels: u32,
    pub sample_rate: u32,
    pub frames: usize,
}

unsafe extern "C" {
    fn get_audio_info(
        filename: *const std::ffi::c_char,
        info: *mut AudioInfo,
    ) -> i32;
}

impl FromFile for Clip {
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut info = AudioInfo {
            channels: 0,
            sample_rate: 0,
            frames: 0,
        };

        let filename = CString::new(path.as_ref().to_string_lossy().as_bytes())
            .map_err(|_| Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid filename",
            ))?;
        
        let result = unsafe {
            get_audio_info(filename.as_ptr(), &mut info)
        };
        
        if result != 0 {
            error("Can't load audio clip!");
            Err(Error::new(ErrorKind::Other, "Cant load audio clip!"))
        }
        else {
            Ok(Self {
                samples: vec![0.0; info.frames as usize * info.channels as usize],
                frames: info.frames,
                channels: info.channels,
                sample_rate: info.sample_rate,
            })
        }
    }
}

#[derive(Debug)]
pub struct Pattern {
    repeat_count: ,
    value: JsonValue,
}

impl FromFile for Pattern {
    fn from_file(p: impl AsRef<Path>) -> Result<Self, Error> {
        let contents = fs::read_to_string(p).unwrap();
        // TODO: Test using an Option as Pattern::repeat_count
        Ok(Pattern::from_string(&contents)?)
    } // from_file
} // impl

impl FromString for Pattern {
    fn from_string(s: &str) -> Result<Self, Error> {
        let mut js = json::parse(&s).unwrap();
        let mut repeat_count = 1;
        for i in 0..js.len() {
            match js[i] {
                JsonValue::Number(n) => {
                    repeat_count = n.as_fixed_point_i64(0).unwrap() as u16;
                    if repeat_count == 0 { repeat_count = 10000; }
                    break;
                } // Number
                _ => { error("Bad pattern element"); }
            } // match
        } // for i
        Ok(
            Self {
                repeat_count,
                value: js,
            } // Self
        ) // Ok
    } // from_string
} // impl

// TODO: Implement Pattern::from_json_value

// pub struct Song {
//     pattern: Pattern,
// }

// pub struct SongList {
//     songs: Vec<Song>,
// }

// TODO: Separate Point, Label, etc. to a new crate, `terminal.rs`

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
    } // print
} // Label

impl Refresh for Label {
    fn refresh(&mut self) -> Result<(), Error> {
        // TODO: refresh the label
        execute!( stdout(),
                  MoveTo(self.position.x, self.position.y),
                  PrintStyledContent(self.text.clone().with(self.color)),
                )?; // execute
        Ok(())
    } // refresh
} // impl

#[derive(Debug)]
enum PlayerEvent {
    Pedal,
    Space,
    Escape,
    Rewind,
    Next,
} // PlayerEvent


pub struct Player {
    // Debug is not implemented for MidiInputConnection!
    clips: VecDeque<String>,
    cache: HashMap<OsString, Clip>,
    playing: bool,
    pedal: bool,
    sender: Sender<PlayerEvent>,
    receiver: Receiver<PlayerEvent>,
    midi_connection: Option<MidiInputConnection<()>>,
    song_list: Vec<String>,
    songs_folder: PathBuf,
    rewind: bool,
    next: bool,
} // Player

impl Player {
    pub fn new () -> Self {
        let(sender, receiver) = channel::<PlayerEvent>();

        Self {
            clips: VecDeque::new(),
            cache: HashMap::<OsString, Clip>::new(),
            playing: false,
            pedal: false,
            sender,
            receiver,
            midi_connection: None,
            song_list: Vec::new(),
            songs_folder: PathBuf::new(),
            rewind: false,
            next: false
        } // Self
    } // new

    pub fn init(&mut self) -> Result<(), Error> {
        self.get_song_list()?;
        debug(&format!("Searching for the `songs` folder..."));
        self.songs_folder = session_folder()?.join(SONGS_DIR_NAME);
        self.get_keyboard_events();
        self.start_midi();
        Ok(())
    } // init

    pub fn get_keyboard_events(&mut self) {
        let sender = self.sender.clone();
        thread::spawn(move || {
            loop {
                if event::poll(Duration::from_millis(50)).unwrap() {
                    if let Ok(Event::Key(key)) = event::read() {
                        match key.code {
                            KeyCode::Char(' ') => {
                                let _ = sender.send(PlayerEvent::Space);
                            }
                        
                            KeyCode::Esc => {
                                let _ = sender.send(PlayerEvent::Escape);
                            }
                        
                            _ => {}
                        } // match
                    } // if let Ok ...
                } // if event::poll
            } // loop
        }); // thread::spawn
    } // get_keyboard events
    
    pub fn start_midi(&mut self) {
        let sender = self.sender.clone();
        let midi = MidiInput::new("peroxide")
            .expect("Couldn't initialize MIDI");
    
        let ports = midi.ports();
    
        println!("MIDI inputs: {}", ports.len());
    
        for (i, port) in ports.iter().enumerate() {
            println!("{}: {}", i, midi.port_name(port).unwrap());
        } // for
    
        let port = ports.get(1)
            .expect("MIDI port 1 doesn't exist.");
    
        self.midi_connection = Some(
                midi.connect(
                port,
                "peroxide-input",
                move |_timestamp, message, _| {
                    match message {
                        [0xF8] | [0xFE] => {} // ignore
        
                        [0xB0, 64, value] if *value > 63 => {
                            let _ = sender.send(PlayerEvent::Pedal);
                        } // Pedal up
        
                        _ => {
                            println!("MIDI: {:?}", message);
                        } // _
                    } // match
                }, // move lambda
                (),
            ).expect("Couldn't open MIDI port 1")
        ); // Some
    
        println!("Successfully opened MIDI port 1");
    
    } // start_midi

    pub fn get_song_list(&mut self) -> Result<(), Error> {
        let arguments = &ARGUMENTS.get().unwrap().args;
        let nargs = arguments.len();
        if let Some(input) = INPUT.get() {
            self.song_list = input.lines().map(str::to_owned).collect();
        } else if nargs > 0 {
            self.song_list = read_lines(&arguments[0])?;
        } else {
            self.song_list = get_most_recent_song_list()?;
        }
        while self.song_list.last().is_some_and(|s| s.is_empty()) {
            self.song_list.pop();
        }
        trace(&format!("Songs: {:#?}", self.song_list));
        Ok(())

    }

    // pub fn parse(&mut self, pattern: Pattern) -> Result<(), Error> {
    //     // Wait for the sustain pedal to begin parsing (playing) the pattern
    //     if !playing {
    //         while !self.pedal {
    //             self.poll_events(); 
    //         } // while
    //     } // if !playing
    //     debug("Pedal event received!\nStarting playback...");
    //     self.pedal = false;
    //     self.playing = true;
        
    //     match pattern.value {
    //         JsonValue::Array(a) => self.parse_items(&a, pattern.repeat_count.unwrap()),
    //         _ => {
    //             error("Pattern must be an array!");
    //             Ok(())
    //         } // error
    //     } // match
    // } // parse
    
    pub fn parse(&mut self, pattern: Pattern) -> Result<(), Error> {
        // Wait for the sustain pedal to begin parsing (playing) the pattern
        if !self.playing {
            while !self.pedal {
                self.poll_events(); 
            } // while
        } // if !playing
        debug("Pedal event received!\nStarting playback...");
        self.pedal = false;
        self.playing = true;
        
        match pattern.value {
            JsonValue::Array(a) => self.parse_items(&a), &pattern.repeat_count),
            _ => {
                error("Pattern must be an array!");
                Ok(())
            } // error
        } // match
    } // parse

    /// Parse a pattern recursively
    pub fn parse_items(&mut self, pattern: &Pattern, repeat_count: &u16) -> Result<(), Error> {
        // ...
        debug(&format!("Parsing pattern: {:?}", pattern));
        let mut iteration = 0;
        if *repeat_count == 0 { repeat_count = 10000; }

        while iteration < pattern.repeat_count {
            for item in pattern {
                match item {
                    JsonValue::Number(n) => {
                        // repeat_count = n.as_fixed_point_i64(0).unwrap() as usize;
                        // if repeat_count == 0 { repeat_count = usize::MAX; }
                    } // Number
                    JsonValue::Array(a) => { self.parse_items(&a,)?; }
                    JsonValue::Short(s) => { 
                        debug(&format!("Clip: {:?}", s));
                        self.clips.push_back(s.to_string());
                    } // Short
                    _ => {
                        error("Parsing error!");
                    } // _ (error)
                } // match
            } // for 
            iteration += 1;
            // Finite repetitions need to actually be dealt with soon
            // This is probably where we need to pause and let the audio finish
            if self.pedal {
                self.pedal = false;
                break;
            } // if
        } // while
        Ok(())
    } // parse_items

    fn poll_events(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                PlayerEvent::Pedal => self.pedal = true,
                PlayerEvent::Space => self.pedal = true,
                PlayerEvent::Escape => self.playing = false,
                PlayerEvent::Rewind => self.rewind = true,
                PlayerEvent::Next => self.next = true,
            } // match
        } // while
    } // poll_events
    
    pub fn play(&mut self) -> Result<(), Error> {
        self.init()?;
        
        for song_title in self.song_list {
            info(&format!("Song: {}", song_title));
            let song_folder = self.songs_folder.join(song_title);
            let clips_folder = song_folder.join(CLIPS_DIR_NAME);

            let files: Vec<_> = fs::read_dir(clips_folder)?
                .collect::<Result<Vec<_>, _>>()?;

            for f in files {
                let key: OsString = f.path().file_stem().expect("REASON").to_owned();
                match Clip::from_file(f.path()) {
                    Ok(clip) => {
                        self.cache.insert(key, clip); 
                        info(&format!("Clip loaded: {:?}", key));            
                    } // Ok
                    Err(e) => {
                        error(&format!("Clip could not be loaded from file: {:?}: {}", key, e)); 
                    } // Err
                } // match
            } // for f
            let song_file = song_folder.join(SONG_FILE_NAME);
            let contents = fs::read_to_string(song_file)?;
            match json::parse(&contents) {
                Ok(js) => { self.parse(&Pattern::from_json_value(js))?; }
                Err(e) => { error("Error parsing JSON pattern!"); }
            } // match
        } // for song_title
        
        Ok(())
    } // play
} // impl Player


pub fn run() -> Result<(), Error> {
    let mut player = Player::new();
    player.play();
    // TODO: Implement the curses style interface in the comment below.
    Ok(())
}

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
