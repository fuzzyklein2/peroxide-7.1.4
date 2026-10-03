use std::io::Error;
use std::path::Path;

use json::JsonValue;

pub trait FromFile: Sized {
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait FromString: Sized {
    fn from_string(s: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait FromJsonValue {
    fn from_json_value(js: &JsonValue) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait FromVector<T> {
    fn from_vector(&mut self, v: Vec<T>) -> Result<Self, Error>
    where
        Self: Sized;
}

