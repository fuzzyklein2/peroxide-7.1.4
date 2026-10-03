use std::io::{Error, stdout};

use crossterm::{ execute, ExecutableCommand, QueueableCommand,
    event::{ KeyEvent },
    terminal::{ Clear, ClearType, disable_raw_mode, enable_raw_mode },
    cursor::{ MoveTo },
    style::{ Color, Print, PrintStyledContent, self, StyledContent, Stylize },
    event::{ self, Event, KeyCode }
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

pub trait Refresh {
    fn refresh(&mut self) -> Result<(), Error>;
}

pub struct Label {
    pub position: Point,
    pub width: usize,
    pub text: String,
    pub color: Color,
}

impl Label {
    pub fn new(p: Point, s: String) -> Self {
        let w = s.len();
        Self { position: p, width: w, text: s, color: Color::White }
    }

    pub fn with_color(p: Point, s: String, c: Color) -> Self {
        let w = s.len();
        Self { position: p, width: w, text: s, color: c }
    }

    pub fn erase(&mut self) -> Result<(), Error> {
        self.text = " ".repeat(self.width);
        self.refresh()?;
        Ok(())
    }

    pub fn set(&mut self, s: String) -> Result<(), Error> {
        self.erase();
        self.text = s;
        self.refresh()?;
        Ok(())
    }

    pub fn print(&mut self) {
        print!("{}", self.text);
    } // print
} // Label

impl Refresh for Label {
    fn refresh(&mut self) -> Result<(), Error> {
        // TODO: refresh the label
        execute!( stdout(),
                  MoveTo(self.position.x, self.position.y),
                  PrintStyledContent(self.text.clone().with(self.color)),
                )?; // execute
        Ok(())
    } // refresh
} // impl


    /*

    let status_label_text = "Status".to_owned();
    let title_pos = Point::new(1, 1);
    let mut title_label = Label::new(title_pos, "🌿  PEROXIDE".to_owned());
    let status_label_pos = Point::new(1, 3);
    let mut status_label = Label::with_color(status_label_pos, status_label_text, Color::Cyan);
    let status_text_pos = Point::new(9, 3);
    let mut status_text = Label::new(status_text_pos, "Stopped".to_owned());
    
    // execute!( stdout(),
    //           Clear(ClearType::All),
    //           // MoveTo(1, 3),
    //           // PrintStyledContent(status_label),
    //           // Print(": Stopped"),
    //           // MoveTo(1, 5),
    //           // Print("")
    //         )?;
    
    title_label.refresh()?;
    status_label.refresh()?;

    execute!( stdout(),
              Print(":"),
            )?;

    status_text.refresh()?;

    println!("\n");

    */
        
    // enable_raw_mode()?;
