use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Clear, Widget},
};
use tui_textarea::{CursorMove, TextArea};

enum EditorStatus {
    Hiding,
    Viewing,
    Editing,
}

pub struct Content {
    pub subject: String,
    pub body: String,
}

pub struct Editor<'a> {
    status: EditorStatus,
    textarea: TextArea<'a>,
    content: Option<Content>,
}

impl Editor<'_> {
    pub fn new() -> Self {
        Self {
            status: EditorStatus::Hiding,
            textarea: TextArea::default(),
            content: None,
        }
    }

    pub fn handle_key_press_event(&mut self, key_event: KeyEvent) -> bool {
        match self.status {
            EditorStatus::Editing => {
                match key_event.code {
                    KeyCode::Esc => self.done(),
                    _ => drop(self.textarea.input(key_event)),
                };
                return true;
            }
            EditorStatus::Viewing => {
                match key_event.code {
                    KeyCode::Enter => self.edit(),
                    KeyCode::Esc => self.hide(),
                    _ => (),
                };
                return true;
            }
            _ => (),
        }

        false
    }

    pub fn get_content(&mut self) -> Option<Content> {
        self.content.take()
    }

    pub fn start(&mut self, subject: &str, body: &str) {
        let text = if body == "" {
            String::from(subject)
        } else {
            format!("{}\n\n{}", subject, body)
        };
        self.textarea = TextArea::default();
        self.textarea.insert_str(text);
        self.textarea.move_cursor(CursorMove::Top);
        self.textarea.move_cursor(CursorMove::End);

        if subject.len() == 0 {
            self.edit();
        } else {
            self.view();
        }
    }

    fn done(&mut self) {
        let mut not_blank = 0;
        let lines = self
            .textarea
            .lines()
            .iter()
            .filter(|line| {
                if not_blank > 1 {
                    true
                } else if !line.trim().is_empty() {
                    not_blank += 1;
                    true
                } else {
                    false
                }
            })
            .map(|line| line.clone())
            .collect::<Vec<_>>();

        let subject = if let Some(line) = lines.get(0) {
            line.clone()
        } else {
            String::from("(no subject)")
        };

        let body = if let Some(lines) = lines.get(1..) {
            lines
                .iter()
                .map(|line| line.clone())
                .collect::<Vec<_>>()
                .join("\n")
                .clone()
        } else {
            String::from("")
        };

        self.start(&subject, &body);
        self.content = Some(Content { subject, body });
    }

    fn view(&mut self) {
        self.status = EditorStatus::Viewing;
    }

    fn edit(&mut self) {
        self.status = EditorStatus::Editing;
    }

    fn hide(&mut self) {
        self.status = EditorStatus::Hiding;
    }
}

impl Widget for &Editor<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let EditorStatus::Hiding = self.status {
            return;
        }

        Clear.render(area, buf);

        let title = match self.status {
            EditorStatus::Viewing => " view ",
            EditorStatus::Editing => " edit ",
            _ => "",
        };
        let keys = match self.status {
            EditorStatus::Viewing => {
                Line::from(vec![
                    " ".into(),
                    "enter".red().into(),
                    " edit | ".into(),
                    "esc".red().into(),
                    " back ".into(),
                ])
            }
            EditorStatus::Editing => {
                Line::from(vec![
                    " ".into(),
                    "esc".red().into(),
                    " save ".into(),
                ])
            }
            _ => Line::from(""),
        };

        let block = Block::new()
            .borders(Borders::ALL)
            .title(title)
            .title_bottom(keys.right_aligned());
        let inner = block.inner(area);
        block.render(area, buf);
        self.textarea.render(inner, buf);
    }
}
