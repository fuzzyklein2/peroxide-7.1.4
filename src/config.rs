use std::fs;
use std::path::PathBuf;
use json::JsonValue;

#[derive(Debug)]
pub struct JSON {
    pub value: JsonValue,
}

impl JSON {
    pub fn new(p: &PathBuf) -> Self {
        let contents = fs::read_to_string(p).unwrap();

        Self {
            value: json::parse(&contents).unwrap(),
        }
    }
}