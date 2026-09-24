use std::collections::{ HashMap, VecDeque };
use std::ffi::{ CString, OsString };
use std::fs;
use std::io::{ self, Error, stdout };
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
use midir::{ MidiInput };

use crate::{ ARGUMENTS, CONFIGURATION, INPUT };
use crate::files;
use crate::files::{ read_lines };
// use crate::ARGUMENTS;
use crate::logging::{ error, warn, info, debug, trace, init_log };

pub const SONGS_DIR_NAME: &str = "songs";
pub const CLIPS_DIR_NAME: &str = "clips";
pub const SONG_FILE_NAME: &str = "song.json";

static SONG_LIST: OnceLock<Vec<String>> = OnceLock::new();

static KYBD_SENDER: OnceLock<Sender<KeyEvent>> = OnceLock::new();
static KYBD_RECEIVER: OnceLock<Mutex<Receiver<KeyEvent>>> = OnceLock::new();

static MIDI_SENDER: OnceLock<Sender<bool>> = OnceLock::new();
static MIDI_RECEIVER: OnceLock<Mutex<Receiver<bool>>> = OnceLock::new();

pub fn session_folder() -> Result<PathBuf, io::Error> {
    Ok(PathBuf::from(CONFIGURATION.get()
                                  .unwrap()
                                  .value["session folder"]
                                  .to_string()))
}

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

trait FromFile: Sized {
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized;
}

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
        }
        
        Ok(Self {
            samples: vec![0.0; info.frames as usize * info.channels as usize],
            frames: info.frames,
            channels: info.channels,
            sample_rate: info.sample_rate,
        })

    }
}

#[derive(Debug)]
pub struct Pattern {
    value: JsonValue,
}

impl FromFile for Pattern {
    fn from_file(p: impl AsRef<Path>) -> Result<Self, Error> {
        // let filename = CString::new(path.as_ref().to_string_lossy().as_bytes())
        //     .map_err(|_| Error::new(
        //         std::io::ErrorKind::InvalidInput,
        //         "Invalid filename",
        //     ))?;
        
        let contents = fs::read_to_string(p).unwrap();

        Ok(Self {
            value: json::parse(&contents).unwrap(),
        })
    }
}

pub struct Song {
    pattern: Pattern,
}

pub struct SongList {
    songs: Vec<Song>,
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

pub fn get_song_list() -> Result<(), Error> {
    let arguments = &ARGUMENTS.get().unwrap().args;
    let nargs = arguments.len();
    let mut song_list = Vec::<String>::new();
    if let Some(input) = INPUT.get() {
        song_list = input.lines().map(str::to_owned).collect();
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
    Ok(())
}

pub fn start_midi() -> midir::MidiInputConnection<()> {
    let midi = MidiInput::new("peroxide")
        .expect("Couldn't initialize MIDI");

    let ports = midi.ports();

    println!("MIDI inputs: {}", ports.len());

    for (i, port) in ports.iter().enumerate() {
        println!("{}: {}", i, midi.port_name(port).unwrap());
    }

    let port = ports.get(1)
        .expect("MIDI port 1 doesn't exist.");

    let connection = midi.connect(
        port,
        "peroxide-input",
        move |_timestamp, message, _| {
            match message {
                [0xF8] | [0xFE] => {}

                [0xB0, 64, value] if *value > 63 => {
                    let _ = MIDI_SENDER.get().unwrap().send(true);
                }

                _ => {
                    println!("MIDI: {:?}", message);
                }
            }
        },
        (),
    ).expect("Couldn't open MIDI port 1");

    println!("Successfully opened MIDI port 1");

    connection
}

pub struct Player {
    clips: VecDeque<String>,
    cache: HashMap<String, Clip>
}

impl Player {
    pub fn parse(&mut self, pattern: JsonValue) -> Result<(), Error> {
        match pattern {
            JsonValue::Array(a) => self.parse_items(&a),
            _ => {
                error("Pattern must be an array!");
                Ok(())
            }
        }
    }
    
    /// Parse a pattern recursively
    pub fn parse_items(&mut self, pattern: &[JsonValue]) -> Result<(), Error> {
        // ...
        debug(&format!("Parsing pattern: {:?}", pattern));
        let mut repeat_count = 1;

        for item in pattern {
            match item {
                JsonValue::Number(n) => { n.as_fixed_point_i64(0).unwrap() as usize;
                }
                JsonValue::Array(a) => { self.parse_items(&a)?; }
                JsonValue::Short(s) => { 
                    debug(&format!("Clip: {:?}", s));
                    self.clips.push_back(s.to_string());
                } // Short
                _ => {
                    error("Parsing error!");
                } // _ (error)
            } // match
        } // for        
        Ok(())
    }
}


pub fn run() -> Result<(), Error> {

    get_song_list()?;
    
    let (tx, rx) = channel::<KeyEvent>();
    KYBD_SENDER.set(tx).unwrap();
    KYBD_RECEIVER.set(Mutex::new(rx)).unwrap();

    let (midi_tx, midi_rx) = channel::<bool>();
    MIDI_SENDER.set(midi_tx).unwrap();
    MIDI_RECEIVER.set(Mutex::new(midi_rx)).unwrap();
    
    thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap() {
                if let Ok(Event::Key(key)) = event::read() {
                    let _ = KYBD_SENDER.get().unwrap().send(key);
                }
            }
        }
    });

    let _midi_connection = start_midi();
        
    let mut running = true;
    let mut playing = false;
    let mut status = "Stopped";
    let mut pedal_down = false;
    let mut stop_endless_repeat = true;
    let receiver = KYBD_RECEIVER.get().unwrap().lock().unwrap();

    debug(&format!("Searching for the `songs` folder..."));

    let SONGS_FOLDER = session_folder()?.join(SONGS_DIR_NAME);
    
    let mut clips_map = HashMap::<OsString, Clip>::new();

    for song_title in SONG_LIST.get().unwrap() {
        let song_folder = SONGS_FOLDER.join(song_title);
        let clips_folder = song_folder.join(CLIPS_DIR_NAME);

        let mut files: Vec<_> = fs::read_dir(clips_folder)?
            .collect::<Result<Vec<_>, _>>()?;

        // Load the clip from each file
        for f in files {
            let key: OsString = f.path().file_stem().expect("REASON").to_owned();
            
            match Clip::from_file(f.path()) {
                Ok(clip) => {
                    info(&format!("Clip loaded: {:?}", key));            
                    clips_map.insert(key, clip); 
                } // Ok
                Err(e) => {
                    error(&format!("Clip could not be loaded from file: {:?}", key)); 
                } // Err
            } // match
        } // for

        // Parse the song file
        let song_file = song_folder.join(SONG_FILE_NAME);
        debug(&format!("Opening song file: {:?}", song_file));
        let contents = fs::read_to_string(song_file).unwrap();
        debug(&format!("{:?}", contents));
        let js = json::parse(&contents).unwrap();
        
        let mut player = Player {
            clips: VecDeque::new(),
            cache: HashMap::new(),
        };
        
        player.parse(js)?;
    }

    
    
    // while running {
    //     if let Ok(key) = receiver.try_recv() {
    //         match key.code {
    //             KeyCode::Char(' ') => {
    //                 playing = !playing;
    //                 // if (playing) { status_text.set("Playing".to_owned()); }
    //                 // else { status_text.set("Stopped".to_owned()); }
    //                 // status_text.refresh()?;
    //                 println!("Spacebar event received.")
    //             }
    //             // **TODO:** This should escape the current song, rather than quit the program.
    //             KeyCode::Esc => {
    //                 running = false;
    //             }

    //             _ => {}
    //         }
    //     }

    //     if let Ok(pedal_up) = MIDI_RECEIVER.get().unwrap().lock().unwrap().try_recv() {
    //         if pedal_up {
    //             stop_endless_repeat = true;
    //             println!("Sustain pedal event received.");
    //         }
    //     }

        // Do whatever else the main thread needs to do...
    // }

    
    // disable_raw_mode()?;
    println!("\n");
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
