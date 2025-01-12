use std::io;

mod todo;
use todo::Todo;
mod db;
mod editor;
mod preview;
mod date_picker;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut todo = Todo::new();
    let result = todo.run(terminal);

    ratatui::restore();
    result
}
