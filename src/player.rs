#![allow(warnings)]
use std::collections::{ HashMap, VecDeque };
use std::ffi::{ CString, OsString };
use std::fs;
use std::io::{ Error, ErrorKind };
use std::path::PathBuf;
// use std::path::{ Path, PathBuf };
use std::sync::{ mpsc::{ channel, Sender, Receiver }};
use std::thread;
use std::time::Duration;

use crossterm::{ execute, ExecutableCommand, QueueableCommand,
    event::{ KeyEvent },
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode },
    cursor::{ MoveTo },
    style::{ Color, Print, PrintStyledContent, self, StyledContent, Stylize },
    event::{ self, Event, KeyCode }
};

use json::JsonValue;

use maudio::device::Device;
use maudio::device::device_builder::DeviceBuilder;

use midir::{ MidiInput, MidiInputConnection };

use hw::{
    ARGUMENTS,
    INPUT,
    logging::{ debug, error, info, trace, warn },
    files::read_lines,
};

use crate::{
    AudioInfo,
    Clip,
    files::{
        CLIPS_DIR_NAME,
        get_most_recent_song_list,
        session_folder,
        SONGS_DIR_NAME,
        SONG_FILE_NAME,
    },
    Frame,
    get_audio_info,
    Pattern,
    traits::{
        FromFile,
        FromJsonValue,
        FromPattern,
    },
};

#[derive(Debug)]
enum PlayerEvent {
    Pedal,
    Space,
    Escape,
    Rewind,
    Next,
    // PatternFinished,
} // PlayerEvent

/*
let mut device = DeviceBuilder::playback()
    .f32()
    .playback_channels(2)
    .sample_rate(SampleRate::Sr48000)
    .with_callback(|_device, output| {
        output.fill(0.0);
    })?;

device.device_start()?;
*/

pub struct Player {
    // Debug is not implemented for MidiInputConnection!
    clips: VecDeque<String>,
    cache: HashMap<OsString, Clip>,
    frames: Vec<Frame>,
    playing: bool,
    pedal: bool,
    sender: Sender<PlayerEvent>,
    receiver: Receiver<PlayerEvent>,
    midi_connection: Option<MidiInputConnection<()>>,
    song_list: Vec<String>,
    songs_folder: PathBuf,
    rewind: bool,
    next: bool,
    sample_rate: u32,
    device: Option<Device>,
} // Player

impl Player {
    pub fn new () -> Self {
        let(sender, receiver) = channel::<PlayerEvent>();

        Self {
            clips: VecDeque::new(),
            cache: HashMap::<OsString, Clip>::new(),
            frames: Vec::<Frame>::new(),
            playing: false,
            pedal: false,
            sender,
            receiver,
            midi_connection: None,
            song_list: Vec::new(),
            songs_folder: PathBuf::new(),
            rewind: false,
            next: false,
            sample_rate: 0,
            device: None,
        } // Self
    } // new

    pub fn init(&mut self) -> Result<(), Error> {
        self.get_song_list()?;
        debug(&format!("Searching for the `songs` folder..."));
        self.songs_folder = session_folder()?.join(SONGS_DIR_NAME);
        self.get_keyboard_events();
        self.start_midi();
        // self.start_audio();
        self.play_list();
        
        let mut info = AudioInfo {
            channels: 0,
            sample_rate: 0,
            frames: 0,
        };

        let clips_folder = self.songs_folder
                           .join(self.song_list[0])
                           .join(CLIPS_DIR_NAME);

        let mut files: Vec<_> = fs::read_dir(clips_folder)?
            .collect::<Result<Vec<_>, _>>()?;

        let path = files[0];

        let filename = CString::new(
            path.to_str()
                .ok_or_else(|| Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid filename",
                ))?
        )?;
        
        let result = unsafe {
            get_audio_info(filename.as_ptr(), &mut info)
        };
                
        device = DeviceBuilder::playback()
            .f32()
            .playback_channels(2)
            .sample_rate(info.sample_rate)
            .with_callback(|_device, output| {
                output.fill(0.0);
            })?;
        
        device.device_start()?;
        
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
            JsonValue::Array(a) => self.parse_items(
                &Pattern::from_json_value(&json::from(a))?, 
                &pattern.repeat_count
            ), // parse_items
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
        let repeats = match *repeat_count {
            0 => 10000,
            n => n,
        };
        while iteration < repeats {
            for i in 0..pattern.value.len() {
                match &pattern.value[i] {
                    JsonValue::Number(n) => {
                        // repeat_count = n.as_fixed_point_i64(0).unwrap() as usize;
                        // if repeat_count == 0 { repeat_count = usize::MAX; }
                    } // Number
                    JsonValue::Array(a) => {
                        self.parse_items(
                            &Pattern::from_json_value(&JsonValue::Array(a.clone()))?,
                            &repeat_count
                        )?;
                    } // Array
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

    fn wait_for_pedal(&mut self) {
        while let Ok(event) = self.receiver.recv() {
            match event {
                PlayerEvent::Pedal | PlayerEvent::Space => {
                    self.pedal = true;
                    break;
                }
                _ => {}
                // PlayerEvent::Escape => {
                //     self.playing = false;
                //     break;
                // }
                // PlayerEvent::Rewind => self.rewind = true,
                // PlayerEvent::Next => self.next = true,
            }
        }
    }
    
    fn play_list(&mut self) -> Result<(), Error> {
        for song_title in self.song_list.clone() {
            let song_folder = session_folder()?
                              .join(SONGS_DIR_NAME)
                              .join(song_title);
            let song_file = song_folder.join(SONG_FILE_NAME);
            let clips_folder = song_folder.join(CLIPS_DIR_NAME);
            let song_pattern = Pattern::from_file(song_file)?;
            let song_frame = Frame::from_pattern(song_pattern);
            let clip_files: Vec<_> = fs::read_dir(clips_folder)?
                .collect::<Result<Vec<_>, _>>()?;

            self.frames.push(song_frame);

            for f in clip_files {
                let key: OsString = f.path().file_stem().expect("REASON").to_owned();
                match Clip::from_file(f.path()) {
                    Ok(clip) => {
                        self.cache.insert(key.clone(), clip); 
                        info(&format!("Clip loaded: {:?}", key));            
                    } // Ok
                    Err(e) => {
                        error(&format!("Clip could not be loaded from file: {:?}: {}", key, e)); 
                    } // Err
                } // match
            } // for f

            self.wait_for_pedal();
            self.play_song();
        } // for song
        Ok(())
    } // play_list

    fn play_song(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// ⚠️ Deprecated
    pub fn play(&mut self) -> Result<(), Error> {
        self.init()?;
        
        for song_title in self.song_list.clone() {
            info(&format!("Song: {}", song_title));
            let song_folder = self.songs_folder.join(song_title);
            let clips_folder = song_folder.join(CLIPS_DIR_NAME);

            let files: Vec<_> = fs::read_dir(clips_folder)?
                .collect::<Result<Vec<_>, _>>()?;

            for f in files {
                let key: OsString = f.path().file_stem().expect("REASON").to_owned();
                match Clip::from_file(f.path()) {
                    Ok(clip) => {
                        self.cache.insert(key.clone(), clip); 
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
                Ok(js) => { self.parse(Pattern::from_json_value(&js)?)?; }
                Err(e) => { error("Error parsing JSON pattern!"); }
            } // match
        } // for song_title
        
        Ok(())
    } // play
} // impl Player


