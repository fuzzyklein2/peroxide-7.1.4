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

use std::io::Error;

use maudio::audio::sample_rate::SampleRate;
use maudio::device::{CallBackDevice, Device};
use maudio::device::device_builder::DeviceBuilder;
use maudio::device::device_builder::DeviceBuilderOps;

use hw::logging::{ debug };

use crate::audio_device_callback;
use crate::Player;
use crate::PLAYER;

/// # ⚠️ Deprecated
/// 
/// # 💬 Description
/// 
/// Just creates a `Player` object and calls its `play` function.
pub fn run() -> Result<(), Error> {
    let mut device = DeviceBuilder::playback()
    .f32()
    .playback_channels(2)
    .sample_rate(SampleRate::Sr44100)
    .with_callback(audio_device_callback)
    .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

    let mut player = Player::new();
    debug("Running peroxide").expect("");
    player.init()?;

    *PLAYER.lock().unwrap() = player;

    device.device_start()
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

    PLAYER.lock().unwrap().play_list()?;

    // TODO: Implement the curses style interface in the comment below.
    Ok(())
}
