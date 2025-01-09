use std::cell::RefCell;
use std::cmp;
use std::io;
use std::mem;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
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

enum EditType {
    Editing,
    Adding,
    Done,
}

pub struct Todo<'a> {
    db: Db,
    editor: Editor<'a>,
    tasks: Vec<Task>,
    current: usize,
    direction: Direction,
    scroll: RefCell<usize>,
    edit_type: EditType,
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
            edit_type: EditType::Done,
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
                            match self.edit_type {
                                EditType::Editing => {
                                    self.update_current(content);
                                }
                                EditType::Adding => {
                                    self.add_task(content);
                                }
                                _ => (),
                            }
                            self.edit_type = EditType::Done;
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
                    self.edit_type = EditType::Editing;
                    self.editor.start(&task.subject, &task.body, task.done);
                }
            }
            KeyCode::Char('k') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.switch(Direction::Up);
                } else {
                    self.current = self.current.saturating_sub(1);
                }
                self.direction = Direction::Up;
            }
            KeyCode::Char('j') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.switch(Direction::Down);
                } else {
                    if self.current < self.tasks.len().saturating_sub(1) {
                        self.current += 1;
                    }
                }
                self.direction = Direction::Down;
            }
            KeyCode::Char('a') => {
                self.edit_type = EditType::Adding;
                self.editor.start("", "", false);
            }
            KeyCode::Char('d') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.delete_current();
                } else {
                    self.done_current();
                }
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

    fn done_current(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.current) {
            task.done = !task.done;
            self.db.update_one(task);
            self.update();
        }
    }

    fn delete_current(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.current) {
            self.db.delete_one(task.id);
            self.update();
        }
    }

    fn add_task(&mut self, content: Content) {
        if let Some(_) = self.db.insert_one(&content.subject, &content.body) {
            self.update();
            self.current = self.tasks.len().saturating_sub(1);
        }
    }

    fn switch(&mut self, direction: Direction) {
        if let Some(current) = self.tasks.get_mut(self.current) {
            if let Some(mut next) = match direction {
                Direction::Up => self.db.get_prev(current.id),
                Direction::Down => self.db.get_next(current.id),
            } {
                mem::swap(&mut current.id, &mut next.id);
                self.db.update_one(&current);
                self.db.update_one(&next);
                self.current = match direction {
                    Direction::Up => self.current.saturating_sub(1),
                    Direction::Down => self.current + 1,
                };
                self.update();
            }
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
        let height = area.height as usize - 2;
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

        let mut done = false;
        let from = cmp::min(*scroll, len.saturating_sub(1));
        let to = cmp::min(*scroll + height, len);
        let lines: Vec<_> = self.tasks[from..to]
            .iter()
            .enumerate()
            .map(|(i, task)| {
                if i == self.current.saturating_sub(*scroll) {
                    let string = format!("{:<width$}", task.subject, width = area.width.into());
                    if task.done {
                        done = true;
                        Line::from(string.black().on_red())
                    } else {
                        Line::from(string.white().on_red())
                    }
                } else {
                    let subject = task.subject.clone();
                    if task.done {
                        Line::from(subject.red())
                    } else {
                        Line::from(subject)
                    }
                }
            })
            .collect();

        let keys = Line::from(vec![
            " ".into(),
            "j/k".red().into(),
            " select | ".into(),
            "^j/^k".red().into(),
            " move | ".into(),
            "a".red().into(),
            " add | ".into(),
            "d".red().into(),
            if done { " undo | " } else { " done | "}.into(),
            "^d".red().into(),
            " delete | ".into(),
            "enter".red().into(),
            " view | ".into(),
            "esc".red().into(),
            " exit ".into(),
        ]);
        let block = Block::new()
            .borders(Borders::ALL)
            .title(" todo ")
            .title_bottom(keys.right_aligned());
        Paragraph::new(lines).block(block).render(area, buf);
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
            vec!["┌ todo ────────────┐".into()],
            vec!["│".into(), "test_subject      ".white().on_red(), "│".into()],
            vec!["│                  │".into()],
            vec!["└".into(), "r".red(), " view | ".into(), "esc".red(), " exit ┘".into()],
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
