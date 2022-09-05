use termion::event::Key;
use crate::Document;
use crate::terminal::Terminal;
use crate::Row;
use std::env;
use termion::color;
use std::time::Duration;
use std::time::Instant;



const VERSION: &str = env!("CARGO_PKG_VERSION"); // Since cargo.toml has a version number, we can use env!() to retrieve it
const STATUS_BG_COLOUR: color::Rgb = color::Rgb(0, 255, 0);
const STATUS_FG_COLOUR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BLACK_COLOUR: color::Rgb = color::Rgb(0, 0, 0);


#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

struct StatusMessage {
    text: String,
    time: Instant
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
}

impl Editor {
    pub fn run(&mut self) {

        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }

            if self.should_quit {
                break;
            }

            if let Err(error) = self.process_keypad() {
                die(error);
            }
        }
    }
    pub fn default() -> Self {
        let args:  Vec<String> = env::args().collect();
        let mut initial_status = String::from("^Q: Quit \t ^S: Save \t ^P: Preferences [TBD]");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(file_name);

            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("ERROR: Could not open '{}'", file_name);
                // Maybe die() here?
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize the terminal"),
            cursor_position: Position {x: 0, y: 0},
            offset: Position::default(),
            document, // document is shorthand for document: document
            status_message: StatusMessage::from(initial_status)
        }
    }

    fn process_keypad(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match pressed_key{
            Key::Ctrl('q') => {
                self.should_quit = true;
                panic!("Program Terminated")
            },

            Key::Ctrl('s') => {

                if self.document.filename.is_none() {
                    self.document.filename = Some(self.prompt("Save as: ")?);
                }

                if self.document.save().is_ok() {
                    self.status_message = StatusMessage::from(String::from("File saved successfully".to_string()));
                } else {
                    self.status_message = StatusMessage::from(String::from("Error saving process".to_string()));
                }
            }

            Key::Char(c) => {
                // Inserting characters on a key-press
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right)
            }


            // Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }


            Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(()) // Everything is OK and nothing has been returned (We use this because we don't have try..catch)
    }

    fn prompt(&mut self, prompt: &str ) -> Result<String, std::io::Error> {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            if let Key::Char(c) = Terminal::read_key()? {
                if c == '\n' {
                    self.status_message = StatusMessage::from(String::new());
                    break;
                }
                if !c.is_control() {
                    result.push(c);
                }
            }
        }
        Ok(result)
    }

    fn scroll(&mut self) {
        let Position {x, y} = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = offset.x.saturating_sub(width).saturating_add(1);
        }

    }

    fn move_cursor(&mut self, key: Key) {
        let Position{mut x, mut y} = self.cursor_position;
        let height = self.document.len();
        let width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            },
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            },
            Key::Right => {
                //if /*x < width*/ true {
                    // Bit of a hack to stop the frozen cursor once it reaches the row's previous length, but it works for now
                    // TODO: Make the code more elegant and figure out if this is the proper solution
                    x = x.saturating_add(1)
                //}
            },
            _ => ()
        };

        self.cursor_position = Position{x, y}
    }

    pub fn draw_row(&self, row: &Row) {
        let start = self.offset.x;
        let width = self.terminal.size().width as usize;
        let end= start + width;
        let row = row.render(start, end);
        println!("{}\r", row)
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;

        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
                // println!("{}\r", &welcome_message[..width])
                /*
                The [...width] syntax means that we want to slice the string from its beginning until width.
                width has been calculated as the minimum of the screen width or the welcome message length,
                which makes sure that we are never slicing more of a string than what is already there.
                */
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message()
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hacksaw -- version {}\r", VERSION);
        let width = self.terminal.size().width as usize;

        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::clear_screen();
        Terminal::cursor_position(&Position::default());
        // print!("\x1b[2J"); // \x1b is the 'esc' char. The other three bytes are [2J
        /*
        This is called an escape sequence. Escapes sequences tell the terminal to do various text-formatting tasks,
        such as colouring text, moving the cursor, and clearing parts of the screen.
        2 says to clear the entire screen
        */
        if self.should_quit {
            Terminal::clear_screen();
            println!("Terminated Hacksaw\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y), // This used to be self.offset.x, changed because YOLO
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let mut filename = "[No Name]".to_string();

        if let Some(name) = &self.document.filename {
            filename = name.clone();
            filename.truncate(20);
        }


        status = format!("{} - {} lines", filename, self.document.len().saturating_sub(1));

        let line_indicator = format!("{} of {}", self.cursor_position.y.saturating_add(1),
                                     self.document.len().saturating_sub(1));
        let len = status.len() + line_indicator.len();

        if width > len {
            status.push_str(&" ".repeat(width - len - 1));
        }
        status = format!("{} {}", status, line_indicator);

        status.truncate(width );


        Terminal::set_fg_color(STATUS_FG_COLOUR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;

        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

}

fn die(error: std::io::Error) {
    Terminal::clear_screen();
}

