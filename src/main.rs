use std::io;
use std::path::Path;

use sqlite::State;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Layout, Rect, Constraint, Direction},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn init_db() {
    if !Path::new("sqlite.db").exists() {
        let connection = sqlite::open("sqlite.db").unwrap();
        let query = "
            CREATE TABLE todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                done BOOLEAN DEFAULT 0,
                text VARCHAR(255) DEFAULT ''
            );
            INSERT INTO todos (text, done)
            VALUES
                ('do this', 0),
                ('be there', 0),
                ('stop that', 0),
                ('see here', 0),
                ('sudo rm -rf /', 1);
        ";
        connection.execute(query).unwrap();
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut todo = Todo::default();
    todo.todos = vec![];
    todo.dones = vec![];

    init_db();
    let connection = sqlite::open("sqlite.db").unwrap();

    let query = "SELECT * FROM todos;";
    let mut stat = connection.prepare(query).unwrap();
    while let Ok(State::Row) = stat.next() {
        let id = stat.read::<i64, _>("id").unwrap();
        let done = stat.read::<i64, _>("done").unwrap() == 1;
        let text = stat.read::<String, _>("text").unwrap();
        let stuff = Stuff { id, text };

        if done {
            todo.dones.push(stuff);
        } else {
            todo.todos.push(stuff);
        }
    }

    let query = "SELECT * FROM todos WHERE id = ?;";
    let stat = connection.prepare(query).unwrap();
    let rows = stat.into_iter().bind((1, 1)).unwrap().map(|row| row.unwrap());
    for _row in rows {
        //println!("{}", row.read::<&str, _>("text"));
    }

    let result = todo.run(terminal);

    ratatui::restore();
    result
}

#[derive(Default)]
pub struct Todo {
    current: (bool, usize),
    todos: Vec<Stuff>,
    dones: Vec<Stuff>,
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
            KeyCode::Char('j') => {
                if self.current.0 {
                    if self.current.1 < self.dones.len() - 1 {
                        self.current.1 += 1;
                    }
                } else {
                    if self.current.1 < self.todos.len() - 1 {
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
                    if self.current.1 >= self.dones.len() {
                        self.current.1 = self.dones.len() - 1;
                    }
                } else {
                    if self.current.1 >= self.todos.len() {
                        self.current.1 = self.todos.len() - 1;
                    }
                }
            },
            KeyCode::Char('l') => {
                self.current.0 = !self.current.0;

                if self.current.0 {
                    if self.current.1 >= self.dones.len() {
                        self.current.1 = self.dones.len() - 1;
                    }
                } else {
                    if self.current.1 >= self.todos.len() {
                        self.current.1 = self.todos.len() - 1;
                    }
                }
            },
            _ => {},
        };
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
                let string = format!(" {} - {} ", stuff.id.to_string(), stuff.text.clone());
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
            .border_set(border::ROUNDED);

        let dones: Vec<_> = self.dones.iter()
            .enumerate()
            .map(|(i, stuff)| {
                let string = format!(" {} - {} ", stuff.id.to_string(), stuff.text.clone());
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
    text: String,
}

#[cfg(test)]
mod tests {
    use ratatui::text::Span;

    use super::*;

    #[test]
    fn test_render() {
        let mut todo = Todo::default();
        todo.todos = vec![
            Stuff { id: 1, text: String::from("todo") },
        ];
        todo.dones = vec![
            Stuff { id: 2, text: String::from("done") },
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
        let mut todo = Todo::default();
        todo.handle_key_event(KeyCode::Char('q').into());
        assert!(todo.exit);
    }
}
