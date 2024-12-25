use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Layout, Rect, Constraint, Direction},
    style::{Stylize, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget, Padding, Clear},
    DefaultTerminal, Frame,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use tui_textarea::TextArea;

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

pub struct Todo<'a> {
    db: Db,
    current: (bool, usize),
    todos: Vec<Stuff>,
    dones: Vec<Stuff>,
    input: Input,
    textarea: TextArea<'a>,
    entering: Entering,
    adding: bool,
    editing: bool,
    editing_id: i64,
    detail: bool,
    exit: bool,
}

enum Entering {
    Nothing,
    Title,
    Content,
}

impl Todo<'_> {
    pub fn new() -> Self {
        let mut todo = Todo {
            db: Db::new(),
            current: (false, 0),
            todos: vec![],
            dones: vec![],
            input: Input::default(),
            textarea: TextArea::default(),
            entering: Entering::Nothing,
            adding: false,
            editing: false,
            editing_id: 0,
            detail: false,
            exit: false,
        };

        todo.textarea.set_cursor_line_style(Style::default());
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
            let heading_string;
            let title_string;
            let content_string;

            if self.adding {
                heading_string = String::from(" new ");
                title_string = self.input.value().to_string();
                content_string = String::from("");
            } else {
                let stuff;
                if self.current.0 {
                    stuff = &self.dones[self.current.1];
                } else {
                    stuff = &self.todos[self.current.1];
                }

                if self.editing {
                    heading_string = format!(" {} - edit ", stuff.id.to_string());
                    title_string = self.input.value().to_string();
                    content_string = String::from("");
                } else {
                    heading_string = format!(" {} ", stuff.id.to_string());
                    title_string = stuff.title.clone();
                    content_string = stuff.content.clone();
                }
            }

            let heading = Line::from(heading_string);
            let block = Block::bordered()
                .title(heading.left_aligned())
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

            frame.render_widget(Clear, middle);

            frame.render_widget(block, middle);
            let inner = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Length(2),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ],
            ).split(middle);
            let inner = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ],
            ).split(inner[1]);

            if self.adding || self.editing {
                let paragraph = Paragraph::new(title_string);
                frame.render_widget(paragraph, inner[1]);
                frame.render_widget(&self.textarea, inner[3]);

                if let Entering::Title = self.entering {
                    let width = middle.width.max(3) - 5;
                    let scroll = self.input.visual_scroll(width as usize);
                    frame.set_cursor_position((
                        middle.x + ((self.input.visual_cursor()).max(scroll) - scroll) as u16 + 2,
                        middle.y + 1,
                    ));
                }
            } else {
                let paragraph = Paragraph::new(title_string);
                frame.render_widget(paragraph, inner[1]);
                let content_paragraph = Paragraph::new(content_string);
                frame.render_widget(content_paragraph, inner[3]);
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
                KeyCode::Enter | KeyCode::Esc => {
                    self.detail = false;
                },
                KeyCode::Char('a') => {
                    if self.current.0 {
                        return;
                    }

                    self.detail = false;

                    let index = self.current.1;
                    let stuff = self.todos.get(index).unwrap();
                    self.input = Input::new(stuff.title.clone());
                    self.textarea = TextArea::default();
                    self.textarea.set_cursor_line_style(Style::default());
                    self.textarea.insert_str(stuff.content.clone());
                    self.editing = true;
                    self.editing_id = stuff.id;
                    self.entering = Entering::Title;
                },
                KeyCode::Char('d') => {
                    self.detail = false;

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
                _ => {},
            }
        } else if self.adding || self.editing {
            match key_event.code {
                KeyCode::Esc => {
                    let string = self.input.value();
                    if !string.trim().is_empty() {
                        if self.adding {
                            self.db.add_todo(
                                self.input.value(),
                                &self.textarea.lines().join("\n"),
                            );

                            self.current.0 = false;
                            self.current.1 = self.todos.len() + 1;
                        } else if self.editing {
                            self.db.edit_todo(
                                self.editing_id,
                                self.input.value(),
                                &self.textarea.lines().join("\n"),
                            );
                        }
                        self.update();
                    }
                    self.input.reset();
                    self.adding = false;
                    self.editing = false;
                    self.detail = true;
                },
                KeyCode::Tab => {
                    match self.entering {
                        Entering::Title => self.entering = Entering::Content,
                        Entering::Content => self.entering = Entering::Title,
                        _ => {},
                    };
                },
                KeyCode::Enter => {
                    match self.entering {
                        Entering::Title => self.entering = Entering::Content,
                        Entering::Content => {
                            self.textarea.input(key_event);
                        },
                        _ => {},
                    };
                },
                _ => {
                    match self.entering {
                        Entering::Title => {
                            self.input.handle_event(&Event::Key(key_event));
                        },
                        Entering::Content => {
                            self.textarea.input(key_event);
                        },
                        _ => {},
                    };
                },
            }
        } else {
            match key_event.code {
                KeyCode::Esc => self.exit(),
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
                    self.current.1 = self.current.1.saturating_sub(1);
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
                KeyCode::Enter => {
                    self.detail = if self.current.0 {
                        self.current.1 < self.dones.len()
                    } else {
                        self.current.1 < self.todos.len()
                    };
                },
                KeyCode::Char('a') => {
                    self.adding = true;
                    self.textarea = TextArea::default();
                    self.textarea.set_cursor_line_style(Style::default());
                    self.entering = Entering::Title;
                },
                _ => (),
            };
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &Todo<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let todos: Vec<_> = self.todos.iter()
            .enumerate()
            .map(|(i, stuff)| {
                let string = format!("{} - {}", stuff.id.to_string(), stuff.title.clone());
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
                let string = format!("{} - {}", stuff.id.to_string(), stuff.title.clone());
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
    title: String,
    content: String,
}

#[cfg(test)]
mod tests {
    use ratatui::text::Span;

    use super::*;

    #[test]
    fn test_render() {
        let mut todo = Todo::new();
        todo.todos = vec![
            Stuff { id: 1, title: String::from("todo"), content: String::from("") },
        ];
        todo.dones = vec![
            Stuff { id: 2, title: String::from("done"), content: String::from("") },
        ];

        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 4));

        todo.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            vec![Span::from("╭ todo ────╮╭ done ────╮")],
            vec![Span::from("│ "), "1 - todo".white().on_red(), Span::from(" ││ 2 - done │")],
            vec![Span::from("│          ││          │")],
            vec![Span::from("╰──────────╯╰──────────╯")],
        ]);

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_handle_key_event() {
        let mut todo = Todo::new();
        todo.handle_key_event(KeyCode::Enter.into());
        assert!(todo.exit);
    }
}
