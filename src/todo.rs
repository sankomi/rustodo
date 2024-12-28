use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    widgets::{Paragraph, Widget},
    DefaultTerminal, Frame,
};

use crate::db::Db;

pub struct Todo {
    pub exit: bool,
}

impl Todo {
    pub fn new() -> Self {
        Self { exit: false }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    self.handle_key_press_event(key_event);
                }
            }
            _ => (),
        };

        Ok(())
    }

    fn handle_key_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.exit = true;
            }
            _ => (),
        };
    }
}

impl Widget for &Todo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("here be dragons!").render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let todo = Todo::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 4));
        todo.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "here be dragons!    ",
            "                    ",
            "                    ",
            "                    ",
        ]);

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_key_event() {
        let mut todo = Todo::new();
        todo.handle_key_press_event(KeyCode::Esc.into());
        assert!(todo.exit);
    }
}
