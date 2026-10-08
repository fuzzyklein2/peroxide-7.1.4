use std::io::Error;

use crate::{
    Pattern,
    traits::FromPattern,
};

pub struct Frame {
    pub pattern: Pattern,
    pub current_item: usize,
    pub repeat_count: usize,
    pub current_repeat: usize,
    pub infinite: bool,
    pub pause: bool,
    // pub duration: f64, // milliseconds
}

impl FromPattern for Frame {
    fn from_pattern(p: Pattern) -> Self {
        let reps = p.repeat_count;
        let inf = p.infinite;
        // Ok(
            Self {
                pattern: p,
                current_item: 0,
                repeat_count: reps,
                current_repeat: 0,
                infinite: inf,
                pause: false,
            } // Self
        // ) // Ok        
    } // from_json_value
} //impl
