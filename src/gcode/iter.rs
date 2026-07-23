use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
};

use crate::gcode::{Error, Gcode, from_str};

pub struct GcodeIter {
    lines: Lines<BufReader<File>>,
}

impl GcodeIter {
    // Create a gcode iter object from a file object
    pub fn from_file(file: File) -> Self {
        let reader = BufReader::new(file);
        let lines = reader.lines();
        Self { lines }
    }
}

impl Iterator for GcodeIter {
    type Item = Gcode;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            // If it's not an error
            Some(Ok(line)) => match from_str::<Gcode>(&line) {
                Ok(gcode) => Some(gcode),

                // Get the next one if it's a comment or blank
                // I hope the stack doesn't get filled.
                Err(Error::BlankLine) | Err(Error::CommentOnly) => self.next(),

                // We just call unknown commands macros and call it a day
                Err(Error::UnknownCommand(cmd)) => Some(Gcode::Macro(cmd)),

                Err(e) => {
                    warn!("Error parsing G-Code: {e}");
                    None
                }
            },

            Some(Err(e)) => {
                warn!("Error reading G-Code file: {e}");
                None
            }

            None => {
                warn!("EOF");
                None
            }
        }
    }
}
