use std::fs;
use std::io;
use std::path::{ PathBuf };

use json;

use crate::CONFIGURATION;
use crate::files;

pub fn get_most_recent_song_list() -> Result<Vec<String>, io::Error> {
    let session_dir = PathBuf::from(CONFIGURATION.get().unwrap().value["session folder"].to_string());
    let lists_dir = session_dir.join("lists");
    let mut dir_content: Vec<_> = fs::read_dir(lists_dir)?
        .collect::<Result<Vec<_>, _>>()?;
    let n = dir_content.len();
    let i = n - 1;
    dir_content.sort_by_key(|entry| entry.file_name());
    let song_list_file = &dir_content[i];
    files::read_lines(song_list_file.path())
}
