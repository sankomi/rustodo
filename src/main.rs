use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Layout, Rect, Constraint, Direction},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget, Padding, Clear},
    DefaultTerminal, Frame,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

mod db;
use db::Db;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut todo = Todo::new();

    let result = todo.run(terminal);

    ratatui::restore();
    result
}

pub struct Todo {
    db: Db,
    current: (bool, usize),
    todos: Vec<Stuff>,
    dones: Vec<Stuff>,
    input: Input,
    adding: bool,
    editing: bool,
    editing_id: i64,
    detail: bool,
    exit: bool,
}

impl Todo {
    pub fn new() -> Self {
        let mut todo = Todo {
            db: Db::new(),
            current: (false, 0),
            todos: vec![],
            dones: vec![],
            input: Input::default(),
            adding: false,
            editing: false,
            editing_id: 0,
            detail: false,
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

    pub fn update(&mut self) {
        (self.todos, self.dones) = self.db.get_todos();

        if self.current.0 {
            if self.current.1 >= self.dones.len() {
                self.current.1 = self.dones.len().saturating_sub(1);
            }
        } else {
            if self.current.1 >= self.todos.len() {
                self.current.1 = self.todos.len().saturating_sub(1);
            }
        }

    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        if self.detail || self.adding || self.editing {
            let title_string;
            let content_string;

            if self.adding {
                title_string = String::from(" new todo ");
                content_string = self.input.value().to_string();
            } else {
                let stuff;
                if self.current.0 {
                    stuff = &self.dones[self.current.1];
                } else {
                    stuff = &self.todos[self.current.1];
                }

                if self.editing {
                    title_string = format!(" {} - edit ", stuff.id.to_string());
                    content_string = self.input.value().to_string();
                } else {
                    title_string = format!(" {} ", stuff.id.to_string());
                    content_string = stuff.text.clone();
                }
            }

            let title = Line::from(title_string);
            let block = Block::bordered()
                .title(title.left_aligned())
                .padding(Padding::new(1, 1, 0, 0))
                .border_set(border::ROUNDED);
            let middle = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    if self.detail {
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(60),
                            Constraint::Percentage(20),
                        ]
                    } else {
                        [
                            Constraint::Min(0),
                            Constraint::Length(3),
                            Constraint::Min(0),
                        ]
                    },
                )
                .split(frame.area())[1];
            let middle = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(middle)[1];

            let paragraph = Paragraph::new(content_string)
                .block(block);
            frame.render_widget(Clear, middle);
            frame.render_widget(paragraph, middle);

            if self.adding || self.editing {
                let width = middle.width.max(3) - 5;
                let scroll = self.input.visual_scroll(width as usize);
                frame.set_cursor_position((
                    middle.x + ((self.input.visual_cursor()).max(scroll) - scroll) as u16 + 2,
                    middle.y + 1,
                ))
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self.handle_key_event(key_event),
            _ => {},
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.detail {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Enter | KeyCode::Esc => {
                    self.detail = false;
                },
                _ => {},
            }
        } else if self.adding {
            match key_event.code {
                KeyCode::Enter => {
                    let string = self.input.value();
                    if !string.trim().is_empty() {
                        self.db.add_todo(self.input.value());
                        self.update();
                    }
                    self.input.reset();
                    self.adding = false;
                },
                KeyCode::Esc => {
                    self.input.reset();
                    self.adding = false;
                },
                _ => {
                    self.input.handle_event(&Event::Key(key_event));
                },
            }
        } else if self.editing {
            match key_event.code {
                KeyCode::Enter => {
                    let string = self.input.value();
                    if !string.trim().is_empty() {
                        self.db.edit_todo(self.editing_id, self.input.value());
                        self.update();
                    }
                    self.input.reset();
                    self.editing = false;
                },
                KeyCode::Esc => {
                    self.input.reset();
                    self.editing = false;
                },
                _ => {
                    self.input.handle_event(&Event::Key(key_event));
                },
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('j') => {
                    if self.current.0 {
                        if self.current.1 < self.dones.len().saturating_sub(1) {
                            self.current.1 += 1;
                        }
                    } else {
                        if self.current.1 < self.todos.len().saturating_sub(1) {
                            self.current.1 += 1;
                        }
                    }
                },
                KeyCode::Char('k') => {
                    if self.current.1 > 0 {
                        self.current.1 -= 1;
                    }
                },
                KeyCode::Char('h') | KeyCode::Char('l') => {
                    self.current.0 = !self.current.0;

                    if self.current.0 {
                        if self.current.1 >= self.dones.len() {
                            self.current.1 = self.dones.len().saturating_sub(1);
                        }
                    } else {
                        if self.current.1 >= self.todos.len() {
                            self.current.1 = self.todos.len().saturating_sub(1);
                        }
                    }
                },
                KeyCode::Char('d') => {
                    let done = self.current.0;
                    let index = self.current.1;

                    let stuff;
                    if done {
                        if self.dones.len() > index {
                            stuff = self.dones.get(index).unwrap();
                        } else {
                            return;
                        }
                    } else {
                        if self.todos.len() > index {
                            stuff = self.todos.get(index).unwrap();
                        } else {
                            return;
                        }
                    }

                    self.db.flip(stuff.id);
                    self.update();
                },
                KeyCode::Char('s') => {
                    self.detail = true;
                },
                KeyCode::Char('a') => {
                    self.adding = true;
                },
                KeyCode::Char('e') => {
                    let done = self.current.0;
                    if done {
                        return;
                    }

                    let index = self.current.1;
                    if self.todos.len() <= index {
                        return;
                    }

                    let stuff = self.todos.get(index).unwrap();
                    self.input = Input::new(stuff.text.clone());
                    self.editing = true;
                    self.editing_id = stuff.id;
                },
                _ => (),
            };
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &Todo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let todos: Vec<_> = self.todos.iter()
            .enumerate()
            .map(|(i, stuff)| {
                let string = format!("{} - {}", stuff.id.to_string(), stuff.text.clone());
                if !self.current.0 && self.current.1 == i {
                    Line::from(string.white().on_red())
                } else {
                    Line::from(string)
                }
            })
            .collect();
        let todo_title = Line::from(" todo ");
        let todo_block = Block::bordered()
            .title(todo_title.left_aligned())
            .padding(Padding::new(1, 1, 0, 0))
            .border_set(border::ROUNDED);

        let dones: Vec<_> = self.dones.iter()
            .enumerate()
            .map(|(i, stuff)| {
                let string = format!("{} - {}", stuff.id.to_string(), stuff.text.clone());
                if self.current.0 && self.current.1 == i {
                    Line::from(string.white().on_red())
                } else {
                    Line::from(string)
                }
            })
            .collect();
        let done_title = Line::from(" done ");
        let done_block = Block::bordered()
            .title(done_title.left_aligned())
            .padding(Padding::new(1, 1, 0, 0))
            .border_set(border::ROUNDED);

        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(area);

        Paragraph::new(todos)
            .block(todo_block)
            .render(split[0], buf);
        Paragraph::new(dones)
            .block(done_block)
            .render(split[1], buf);
    }
}

#[derive(Debug)]
pub struct Stuff {
    id: i64,
    done: bool,
    text: String,
}

#[cfg(test)]
mod tests {
    use ratatui::text::Span;

    use super::*;

    #[test]
    fn test_render() {
        let mut todo = Todo::new();
        todo.todos = vec![
            Stuff { id: 1, done: false, text: String::from("todo") },
        ];
        todo.dones = vec![
            Stuff { id: 2, done: true, text: String::from("done") },
        ];

        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 4));

        todo.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            vec![Span::from("╭ todo ────╮╭ done ────╮")],
            vec![Span::from("│"), " 1 - todo ".white().on_red(), Span::from("││ 2 - done │")],
            vec![Span::from("│          ││          │")],
            vec![Span::from("╰──────────╯╰──────────╯")],
        ]);

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_handle_key_event() {
        let mut todo = Todo::new();
        todo.handle_key_event(KeyCode::Char('q').into());
        assert!(todo.exit);
    }
}
