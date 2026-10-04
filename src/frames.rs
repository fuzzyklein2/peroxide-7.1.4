use std::io::Error;

use crate::{
    Pattern,
    traits::FromPattern,
};

pub struct Frame{
    pub pattern: Pattern,
    pub iteration: u16,
    pub current_item: u16,
    // pub repetitions: u16,
    // pub duration: f64, // milliseconds
}


impl FromPattern for Frame {
    fn from_pattern(&mut self, p: Pattern) -> Result<Self, Error> {
        Ok(
            Self {
                pattern: p.clone(),
                iteration: 0,
                current_item: 0,
                // repetitions: p.repeat_count,
            } // Self
        ) // Ok        
    } // from_json_value
} //impl
