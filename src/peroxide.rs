//! # 💬 Description
//! 
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
//! ## 📦 [maudio](https://docs.rs/maudio/latest/maudio/)
//! 
//! Rust port of `miniaudio.h`
//! 
//! ## 📦 [midir](https://https://docs.rs/midir/latest/midir/)
//! 
//! MIDI support
//! 
//! # 🚧 Things To Do
//! 
//! ## `use` statements
//! 
//! Distribute these to whichever crates need them.

// TODO: Get this file to run and run it with -h ==> SYNOPSIS
// TODO: Write some unit tests
// #[cfg(test)]
// mod tests;
use std::io::{ Error };

use hw::logging::{ debug };

use crate::Player;

/// # ⚠️ Deprecated
/// 
/// # 💬 Description
/// 
/// Just creates a `Player` object and calls its `play` function.
pub fn run() -> Result<(), Error> {
    debug("Running peroxide");
    let mut player = Player::new();
    player.play();
    // TODO: Implement the curses style interface in the comment below.
    Ok(())
}
