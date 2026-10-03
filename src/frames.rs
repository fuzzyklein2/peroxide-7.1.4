use crate::{
    Pattern,
};

pub struct Frame{
    pub pattern: Pattern,
    pub iteration: u16,
    pub repetitions: u16,
    pub duration: f64, // milliseconds
}