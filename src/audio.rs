use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use maudio::device::{CallBackDevice};

use crate::PLAYER;

static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn audio_device_callback(_device: CallBackDevice, output: &mut [f32]) {
    let count = CALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
    if count < 12 {
        // debug(&format!("Callback count: {}", count + 1)).expect("");

    }
    let mut player = PLAYER.lock().unwrap();
    // debug(&format!("`PLAYER` unwrapped")).expect("");
    if !player.playing {
        // debug("`player.playing` is false").expect("");
        // Send silence for now
        output.fill(0.0);
        return;
    }
    let requested_samples = output.len();
    let mut clip_name = player.current_clip();
    if clip_name.is_empty() {
        clip_name = player.next_clip();
        if clip_name.is_empty() {
            output.fill(0.0);
            return;
        } // if !current_clip
    } // if !current_clip
    let start = player.current_sample_index;
    player.current_sample_index += requested_samples;
    if player.current_sample_index < player.cache[&clip_name].length() {
        output.copy_from_slice(&player.cache[&clip_name][start..player.current_sample_index]);
        return;
    } // if index < len()
    else {
        // Finsh the current clip and start on the next one
        output.copy_from_slice(&player.cache[&clip_name][start..player.cache[&clip_name].length()]);
        let first_sample_len = player.cache[&clip_name].length() - start;
        let samples_still_needed = requested_samples - first_sample_len;
        clip_name = player.next_clip();
        if clip_name.is_empty() {
            player.playing = false;
            output[first_sample_len..requested_samples].fill(0.0);
            return;
        } // if !current_clip
        // Fill the rest of output with the start of the next clip
        output[first_sample_len..requested_samples]
            .copy_from_slice(&player.cache[&clip_name][0..samples_still_needed]);
    } // else
    return;
} // audio_callback

