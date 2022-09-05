use crate::Row;
use crate::Position;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let mut contents = fs::read_to_string(filename)?; // reads the file, if there is any
        let mut rows = Vec::new();

        for value in contents.lines() {
            rows.push(Row::from(value)); // prints the lines of the file to the terminal
        }

        Ok(Self {
            rows, // makes sure there are no errors
            filename: Some(filename.to_string())
        })
    }

    pub fn save(&self) {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;

            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            // The ? passes any errors that might occur to the caller.
        }
    }
}