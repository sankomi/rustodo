use std::io;
use std::cell::RefCell;
use std::cmp;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Paragraph, Widget},
    DefaultTerminal, Frame,
};

use crate::{
    db::{Db, Task},
    editor::{Content, Editor},
};

enum Direction {
    Up,
    Down,
}

pub struct Todo<'a> {
    db: Db,
    editor: Editor<'a>,
    tasks: Vec<Task>,
    current: usize,
    direction: Direction,
    scroll: RefCell<usize>,
    exit: bool,
}

impl Todo<'_> {
    pub fn new() -> Self {
        let mut todo = Self {
            db: Db::new(),
            editor: Editor::new(),
            tasks: vec![],
            current: 0,
            direction: Direction::Down,
            scroll: RefCell::new(0),
            exit: false,
        };

        todo.update();

        todo
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
        frame.render_widget(&self.editor, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    if self.editor.handle_key_press_event(key_event) {
                        if let Some(content) = self.editor.get_content() {
                            self.update_current(content);
                        }
                    } else {
                        self.handle_key_press_event(key_event);
                    }
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
            KeyCode::Enter => {
                if let Some(task) = self.tasks.get(self.current) {
                    self.editor.start(&task.subject, &task.body);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.current = self.current.saturating_sub(1);
                self.direction = Direction::Up;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.current < self.tasks.len() - 1 {
                    self.current += 1;
                }
                self.direction = Direction::Down;
            }
            _ => (),
        };
    }

    fn update_current(&mut self, content: Content) {
        if let Some(task) = self.tasks.get_mut(self.current) {
            task.subject = content.subject;
            task.body = content.body;
            self.db.update_one(task);
            self.update();
        }
    }

    fn update(&mut self) {
        self.tasks = self.db.list();

        if self.current >= self.tasks.len() {
            self.current = self.tasks.len().saturating_sub(1);
        }
    }
}

impl Widget for &Todo<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let len = self.tasks.len();
        let height = area.height as usize;
        let mut scroll = self.scroll.borrow_mut();
        *scroll = if len <= height {
            0
        } else {
            match self.direction {
                Direction::Up => {
                    if self.current < *scroll {
                        self.current
                    } else if self.current >= *scroll + height {
                        self.current - height + 1
                    } else {
                        *scroll
                    }
                }
                Direction::Down => {
                    if self.current < *scroll + height {
                        *scroll
                    } else if self.current >= height {
                        self.current - height + 1
                    } else {
                        *scroll
                    }
                }
            }
        };

        let from = cmp::min(*scroll, len - 1);
        let to = cmp::min(*scroll + height, len);
        let lines: Vec<_> = self
            .tasks[from..to]
            .iter()
            .enumerate()
            .map(|(i, task)| {
                if i == self.current - *scroll {
                    let string = format!("{:<width$}", task.subject, width = area.width.into());
                    Line::from(string.white().on_red())
                } else {
                    Line::from(task.subject.clone())
                }
            })
            .collect();
        Paragraph::new(lines).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let mut todo = Todo::new();
        todo.tasks = vec![Task {
            id: 1,
            done: false,
            subject: String::from("test_subject"),
            body: String::from("test_body"),
            created: String::from("2024-12-29 14:00:00"),
        }];
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 4));
        todo.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "test_subject        ".white().on_red(),
            "                    ".into(),
            "                    ".into(),
            "                    ".into(),
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
