use std::sync::{LazyLock, Mutex};

// use crate::AudioDevice;
use crate::Player;

pub static PLAYER: LazyLock<Mutex<Player>> =
    LazyLock::new(|| Mutex::new(Player::new()));

// pub static DEVICE: LazyLock<Mutex<Option<AudioDevice>>> =
//     LazyLock::new(|| Mutex::new(None));
