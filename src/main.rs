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
            if self.dones.len() > 0 && self.current.1 >= self.dones.len() {
                self.current.1 = self.dones.len() - 1;
            }
        } else {
            if self.todos.len() > 0 && self.current.1 >= self.todos.len() {
                self.current.1 = self.todos.len() - 1;
            }
        }

    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        if self.detail {
            let stuff;
            if self.current.0 {
                stuff = &self.dones[self.current.1];
            } else {
                stuff = &self.todos[self.current.1];
            }

            let detail_title = Line::from(format!(" {} ", stuff.id.to_string()));
            let detail_block = Block::bordered()
                .title(detail_title.left_aligned())
                .padding(Padding::new(1, 1, 0, 0))
                .border_set(border::ROUNDED);
            let middle = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(frame.area())[1];
            let middle = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(middle)[1];

            let paragraph = Paragraph::new(stuff.text.clone())
                .block(detail_block);
            frame.render_widget(Clear, middle);
            frame.render_widget(paragraph, middle);
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
                KeyCode::Char('s') => {
                    self.detail = false;
                },
                _ => {},
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('j') => {
                    if self.current.0 {
                        if self.dones.len() > 0 && self.current.1 < self.dones.len() - 1 {
                            self.current.1 += 1;
                        }
                    } else {
                        if self.todos.len() > 0 && self.current.1 < self.todos.len() - 1 {
                            self.current.1 += 1;
                        }
                    }
                },
                KeyCode::Char('k') => {
                    if self.current.1 > 0 {
                        self.current.1 -= 1;
                    }
                },
                KeyCode::Char('h') => {
                    self.current.0 = !self.current.0;

                    if self.current.0 {
                        if self.dones.len() > 0 {
                            if self.current.1 >= self.dones.len() {
                                self.current.1 = self.dones.len() - 1;
                            }
                        } else {
                            self.current.1 = 0;
                        }
                    } else {
                        if self.todos.len() > 0 {
                            if self.current.1 >= self.todos.len() {
                                self.current.1 = self.todos.len() - 1;
                            }
                        } else {
                            self.current.1 = 0;
                        }
                    }
                },
                KeyCode::Char('l') => {
                    self.current.0 = !self.current.0;

                    if self.current.0 {
                        if self.dones.len() > 0 {
                            if self.current.1 >= self.dones.len() {
                                self.current.1 = self.dones.len() - 1;
                            }
                        } else {
                            self.current.1 = 0;
                        }
                    } else {
                        if self.todos.len() > 0 {
                            if self.current.1 >= self.todos.len() {
                                self.current.1 = self.todos.len() - 1;
                            }
                        } else {
                            self.current.1 = 0;
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
