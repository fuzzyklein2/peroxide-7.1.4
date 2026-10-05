//! Audio looper controlled by the sustain pedal of a keyboard.
//! 
//! # 🧩 Dependencies
//! 
//! ## 📦 [`crossterm`](https://docs.rs/crossterm/latest/crossterm/)
//! 
//! `curses`-like terminal output
//! 
//! ## 📦 [`json`](https://docs.rs/json/latest/json/)
//! 
//! JSON file parsing
//! 
//! ## 📦 [`maudio`](https://docs.rs/maudio/latest/maudio/)
//! 
//! Rust port of `miniaudio.h`
//! 
//! ## 📦 [`midir`](https://https://docs.rs/midir/latest/midir/)
//! 
//! MIDI support
//! 
//! # 📖 Usage
//! 
//! ```
//! peroxide [OPTIONS] [ARGS]...
//! ```
//! 
//! ## Arguments:
//! 
//! ```
//!   [ARGS]...  Other arguments
//! ```
//! Song lists to play. Only one is currently supported. Piped input,
//! if present, is interpreted as a `NEWLINE` separated song list.
//! If there is piped input CLI arguments are ignored.
//! 
//! ## Options
//! 
//! ```
//!   -d, --debug    Enable debug logging
//!   -v, --verbose  Enable verbose logging
//!   -w, --warn     Enable warnings
//!   -t, --trace    Enable trace logging
//!   -h, --help     Print help
//!   -V, --version  Print version
//! ```
//! # 🚧 Things to Do
//! 
//! ## `player.rs`
//! * **Line 195**:  This is where the outermost pattern should be pushed onto the stack...
//! ## `files.rs`
//! * **Line 26**:  Find or write a function to determine if a directory is empty.
//! ## `terminal.rs`
//! * **Line 65**:  refresh the label
//! ## `peroxide.rs`
//! * **Line 22**:  Get this file to run and run it with -h ==> SYNOPSIS
//! * **Line 23**:  Write some unit tests
//! * **Line 41**:  Implement the curses style interface in the comment below.
//! ## `patterns.rs`
//! * **Line 26**:  Test using an Option as Pattern::repeat_count

pub mod clips;
pub use clips::AudioInfo;
pub use clips::Clip;

pub mod files;

pub mod frames;
pub use frames::Frame;

pub mod patterns;
pub use patterns::Pattern;

pub mod peroxide;

pub mod player;
pub use player::Player;

mod terminal;
pub mod tests;

pub mod traits;