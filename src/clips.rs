use std::ffi::{ CString, };
use std::io::{ Error, ErrorKind };
use std::path::Path;

use hw::logging::error;

use crate::traits::FromFile;

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

