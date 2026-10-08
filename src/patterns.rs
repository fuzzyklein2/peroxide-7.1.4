use std::fs;
use std::io;
use std::io::Error;
use std::path::Path;

use json::{ JsonValue };

use hw::logging::error;

use crate::traits::{
    FromFile,
    FromString,
    FromJsonValue,
    FromVector,
};

#[derive(Clone, Debug)]
pub struct Pattern {
    pub repeat_count: usize,
    pub value: JsonValue,
    // pub pause: bool,
    pub infinite: bool,
}

impl FromFile for Pattern {
    fn from_file(p: impl AsRef<Path>) -> Result<Self, Error> {
        let contents = fs::read_to_string(p).unwrap();
        // TODO: Test using an Option as Pattern::repeat_count
        Ok(Pattern::from_string(&contents)?)
    } // from_file
} // impl

impl FromString for Pattern {
    fn from_string(s: &str) -> Result<Self, Error> {
        let js = json::parse(&s).unwrap();
        Ok(Pattern::from_json_value(&js)?)
    } // from_string
} // impl

impl FromJsonValue for Pattern {
    fn from_json_value(js: &JsonValue) -> Result<Self, Error> {
        let mut repeat_count: usize = 1;
        let mut infinite = false;
        for i in 0..js.len() {
            match js[i] {
                JsonValue::Number(n) => {
                    repeat_count = n.as_fixed_point_i64(0).unwrap() as usize;
                    if repeat_count == 0 { infinite = true; }
                    break;
                } // Number
                _ => { error("Bad pattern element"); }
            } // match
        } // for i
        Ok(
            Self {
                repeat_count,
                value: js.clone(),
                infinite: infinite,
            } // Self
        ) // Ok        
    } // from_json_value
} //impl

impl FromVector<JsonValue> for Pattern {
    fn from_vector(&mut self, v: Vec<JsonValue>) -> Result<Self, Error> {
        let mut a = JsonValue::new_array();
        let mut repeat_count: u16 = 1;
        for value in v {
            match value {
                JsonValue::Number(n) => repeat_count = n.as_fixed_point_i64(0).unwrap() as u16,
                JsonValue::Short(s) => a.push(s.as_str())
                    .map_err(|e| io::Error::other(e.to_string()))?,
                JsonValue::Array(js) => a.push(js)
                    .map_err(|e| io::Error::other(e.to_string()))?,
                _ => {}
            } // match
        } // for value
        Ok (
            Self {
                repeat_count,
                value: a,
                infinite: false,
            } // Self
        ) //Ok
    } // from_vector
} // impl
