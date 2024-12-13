use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Layout, Rect, Constraint, Direction},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut todo = Todo::default();

    todo.list = vec![
        Stuff::new(String::from("do this")),
        Stuff::new(String::from("be there")),
        Stuff::new(String::from("stop that")),
        Stuff::new(String::from("see here")),
    ];
    todo.list.push(Stuff::new(String::from("sudo rm -rf /")));
    let rm = todo.list.get_mut(4).unwrap();
    rm.done = true;

    let result = todo.run(terminal);

    ratatui::restore();
    result
}

#[derive(Default)]
pub struct Todo {
    list: Vec<Stuff>,
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
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self.handle_key_event(key_event),
            _ => {},
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {},
        };
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &Todo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let todo: Vec<_> = self.list.iter().enumerate()
            .filter(|(_, stuff)| !stuff.done)
            .map(|(i, stuff)| {
                let string = format!(" {} - {}", i.to_string(), stuff.text.clone());
                Line::from(string)
            })
            .collect();
        let todo_title = Line::from(" todo ");
        let todo_block = Block::bordered()
            .title(todo_title.left_aligned())
            .border_set(border::ROUNDED);

        let done: Vec<_> = self.list.iter().enumerate()
            .filter(|(_, stuff)| stuff.done)
            .map(|(i, stuff)| {
                let string = format!(" {} - {} ", i.to_string(), stuff.text.clone());
                Line::from(string)
            })
            .collect();
        let done_title = Line::from(" done ");
        let done_block = Block::bordered()
            .title(done_title.left_aligned())
            .border_set(border::ROUNDED);

        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(area);

        Paragraph::new(todo)
            .block(todo_block)
            .render(split[0], buf);
        Paragraph::new(done)
            .block(done_block)
            .render(split[1], buf);
    }
}

#[derive(Debug)]
pub struct Stuff {
    done: bool,
    text: String,
}

impl Stuff {
    pub fn new(text: String) -> Self {
        Stuff {
            done: false,
            text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let mut todo = Todo::default();
        todo.list = vec![
            Stuff::new(String::from("todo")),
            Stuff::new(String::from("done")),
        ];
        let done = todo.list.get_mut(1).unwrap();
        done.done = true;

        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 4));

        todo.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "╭ todo ────╮╭ done ────╮",
            "│ 0 - todo ││ 1 - done │",
            "│          ││          │",
            "╰──────────╯╰──────────╯",
        ]);

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_handle_key_event() {
        let mut todo = Todo::default();
        todo.handle_key_event(KeyCode::Char('q').into());
        assert!(todo.exit);
    }
}
