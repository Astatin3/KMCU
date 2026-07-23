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
        loop {
            let line_result = self.lines.next();

            let line = match line_result {
                Some(Ok(line)) => line,

                Some(Err(e)) => {
                    warn!("Error reading G-Code file: {e}");
                    return None;
                }

                None => {
                    warn!("EOF");
                    return None;
                }
            };

            match from_str::<Gcode>(&line) {
                // If it's not an error
                Ok(gcode) => return Some(gcode),

                // Get the next one if it's a comment or blank
                Err(Error::BlankLine) | Err(Error::CommentOnly) => continue,

                // We just call unknown commands macros and call it a day
                Err(Error::UnknownCommand(cmd)) => return Some(Gcode::Macro(cmd)),

                Err(e) => {
                    warn!("Error parsing G-Code: {e}");
                    return None;
                }
            }
        }
    }
}
