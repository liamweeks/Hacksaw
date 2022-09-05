#![warn(clippy::all, clippy::pedantic)]
mod editor;
mod terminal;
use editor::Editor;
pub use editor::Position;
mod document;
mod row;
//pub use::terminal::Terminal;
pub use document::Document;
pub use row::Row;


fn main() {
    Editor::default().run();
}
