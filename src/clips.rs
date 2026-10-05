use std::ffi::{ CString, };
use std::io::{ Error, ErrorKind };
use std::path::{ Path, PathBuf };

use maudio::audio::formats::SampleBuffer;
use maudio::data_source::sources::decoder::DecoderBuilder;
use maudio::data_source::sources::decoder::DecoderOps;

use hw::logging::error;

use crate::traits::FromFile;

// #[derive(Debug)]
pub struct Clip {
    samples: SampleBuffer<f32>,
    frames: usize,
    channels: u32,
    sample_rate: u32,
    duration: f64,
}

impl std::fmt::Debug for Clip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Clip")
        .field("frames", &self.frames)
        .field("channels", &self.channels)
        .field("sample_rate", &self.sample_rate)
        .field("duration", &self.duration)
        .finish()
    }
}

#[repr(C)]
pub struct AudioInfo {
    pub channels: u32,
    pub sample_rate: u32,
    pub frames: usize,
}

unsafe extern "C" {
    pub fn get_audio_info(
        filename: *const std::ffi::c_char,
        info: *mut AudioInfo,
    ) -> i32;
}

/// TODO: Each clip should store its duration in a member.
impl FromFile for Clip {
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut info = AudioInfo {
            channels: 0,
            sample_rate: 0,
            frames: 0,
        };

        let path = path.as_ref();

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
        
        if result != 0 {
            error("Can't load audio clip!");
            return Err(Error::new(ErrorKind::Other, "Can't load audio clip!"));
        }

        // let path = PathBuf::from(filename.into_string()?);
        
        let mut decoder = DecoderBuilder::new_f32()
            .from_file(&path)
            .map_err(|e| Error::new(
                std::io::ErrorKind::Other,
                e,
            ))?;
        // decoder.read_pcm_frames_into(&self.samples);
        
        // else {
            Ok(Self {
                // samples: vec![0.0; info.frames as usize * info.channels as usize],
                samples: decoder
                    .read_pcm_frames(info.frames.try_into().unwrap())
                    .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?,
                frames: info.frames,
                channels: info.channels,
                sample_rate: info.sample_rate,
                duration: info.frames as f64 / info.sample_rate as f64 * 1000.0,
            })
        // }
    }
}

