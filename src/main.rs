use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::Rect,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut todo = Todo::default();
    let result = todo.run(terminal);

    ratatui::restore();
    result
}

#[derive(Default)]
pub struct Todo {
    exit: bool,
}

impl Todo {
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
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                self.exit = true;
            }
        }

        Ok(())
    }
}

impl Widget for &Todo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("here");
        let title_bottom = Line::from("dragons");

        let block = Block::bordered()
            .title(title.left_aligned())
            .title_bottom(title_bottom.right_aligned())
            .border_set(border::ROUNDED);

        Paragraph::new(Text::from("be"))
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let todo = Todo::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 4));

        todo.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "╭here──────────────╮",
            "│be                │",
            "│                  │",
            "╰───────────dragons╯",
        ]);

        assert_eq!(buf, expected);
    }
}
