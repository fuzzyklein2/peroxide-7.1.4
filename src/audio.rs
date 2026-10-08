use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
// use std::sync::{LazyLock, Mutex};
//
// use maudio::audio::sample_rate::SampleRate;
use maudio::device::{CallBackDevice, Device};
// use maudio::device::device_builder::DeviceBuilder;
// use maudio::device::device_builder::DeviceBuilderOps;
// use maudio::MaudioError;

// use crate::Player;
use crate::PLAYER;
use hw::logging::debug;

// pub static DEVICE: LazyLock<Mutex<Result<Device<f32>, MaudioError>>> =
// LazyLock::new(|| {
//     Mutex::new(
//         DeviceBuilder::playback()
//         .f32()
//         .playback_channels(2)
//         .sample_rate(SampleRate::Sr44100)
//         .with_callback(audio_device_callback)
//     )
// });

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
        return;
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
        // let samples_still_needed = requested_samples - first_sample_len;
        clip_name = player.next_clip();
        if clip_name.is_empty() {
            player.playing = false;
            output[first_sample_len..requested_samples].fill(0.0);
            return;
        } // if !current_clip
        // Fill the rest of output with the start of the next clip
    } // else
    return;
} // audio_callback

// pub struct AudioDevice {
//     device: Device<>,
// }

/*
 *    self.device = Some(
 *        DeviceBuilder::playback()
 *        .f32()
 *        .playback_channels(2)
 *        .sample_rate(SampleRate::Custom(audio_info.sample_rate))
 *        .with_callback(audio_callback));
 *    // self.current_clip = self.next_clip();
 *    self.device.as_mut().expect("Audio device error").device_start()
 *    .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
 */

/*
 *
 *              .with_callback(|_device, output| {
 *
 *                  let requested_samples = output.len();
 *
 *                  // info(&format!("`output` type: {}", std::any::type_name_of_val(output)));
 *                  info(&format!("Number of frames requested: {}",
 *                                output.len() / usize::try_from(audio_info.channels).unwrap()));
 *
 *
 *                  if (!self.playing) {
 *                      output.fill(0.0);
 *                      return;
 *                  } // if !self.playing
 *
 *                  else {
 *                      if self.current_clip.is_none() {
 *                          self.current_clip = self.next_clip();
 *
 *                          if self.current_clip.is_none() { // song is over
 *                              self.playing = false;
 *                              output.fill(0.0);
 *                              return;
 *                          }
 *
 *                          // self.current_sample_index = 0;
 *
 *                          let mut start = self.current_sample_index;
 *                          self.current_sample_index += requested_samples;
 *
 *                          if self.current_sample_index < self.cache[self.current_clip.as_ref().unwrap()].samples.len() {
 *                              output.copy_from_slice(&self.cache[self.current_clip.as_ref().unwrap()].samples.data[start..self.current_sample_index]);
 *                              // https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.copy_from_slice
 *                          } // if index < len()
 *                          else // Finish the current clip and start on the next one.
 *                          {
 *
 *                              output.copy_from_slice(&self.cache[self.current_clip.as_ref().unwrap()].samples.data[start..self.cache[self.current_clip.as_ref().unwrap()].samples.data.len()]);
 *                              let first_sample_len = self.cache[self.current_clip.as_ref().unwrap()].samples.len() - start;
 *                              let samples_still_needed = requested_samples - first_sample_len;
 *
 *                              self.current_clip = self.next_clip();
 *
 *                              if self.current_clip.is_none() { // song is over
 *                                  self.playing = false;
 *
 *                                  // Instead of output.fill(0.0); just fill the rest of output
 *                                  output[first_sample_len..requested_samples].fill(0.0);
 *
 *
 *                                  return;
 *                              } // current clip is none
 *
 *                              // Fill the rest of output with the start of the next clip
 *
 *                          } // Next clip needed to fill output
 *
 *
 *                      } // current_clip.is_none
 *                  } // else
 *              // while current_sample_index <
 *
 *              // output.fill(0.0);
 *              }).map_err(|e| Error::new(std::io::ErrorKind::Other, e))?,
 *      );
 *
 */
