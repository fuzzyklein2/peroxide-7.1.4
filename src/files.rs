use std::fs;
use std::io;
use std::path::PathBuf;
// use std::path::{ Path, PathBuf };

// use hw::config::{ configure, JSON };
use hw::CONFIGURATION;
use hw::files::read_lines;
use hw::logging::error;

pub const SONGS_DIR_NAME: &str = "songs";
pub const CLIPS_DIR_NAME: &str = "clips";
pub const SONG_FILE_NAME: &str = "song.json";

pub fn session_folder() -> Result<PathBuf, io::Error> {
    Ok(PathBuf::from(CONFIGURATION.get()
                                  .unwrap()
                                  .value["session folder"]
                                  .to_string()))
}

pub fn get_most_recent_song_list() -> Result<Vec<String>, io::Error> {
    let lists_dir = session_folder()?.join("lists");
    let mut dir_content: Vec<_> = fs::read_dir(lists_dir)?
        .collect::<Result<Vec<_>, _>>()?;
    // TODO: Find or write a function to determine if a directory is empty.
    if dir_content.is_empty() {
        error("Song lists directory is empty!");
    }
    let n = dir_content.len();
    let i = n - 1;
    dir_content.sort_by_key(|entry| entry.file_name());
    let song_list_file = &dir_content[i];
    read_lines(song_list_file.path())
}

