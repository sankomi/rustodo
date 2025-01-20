use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{Block, Borders, Clear, Widget},
};
use tui_textarea::{CursorMove, TextArea};

enum EditorStatus {
    Hiding,
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
    done: bool,
}

impl Editor<'_> {
    pub fn new() -> Self {
        Self {
            status: EditorStatus::Hiding,
            textarea: TextArea::default(),
            content: None,
            done: false,
        }
    }

    pub fn handle_key_press_event(&mut self, key_event: KeyEvent) -> bool {
        match self.status {
            EditorStatus::Editing => {
                match key_event.code {
                    KeyCode::F(1) => self.done(),
                    _ => drop(self.textarea.input(key_event)),
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

    pub fn start(&mut self, subject: &str, body: &str, done: bool) {
        self.done = done;
        let text = if body == "" {
            String::from(subject)
        } else {
            format!("{}\n\n{}", subject, body)
        };
        self.textarea = TextArea::default();
        self.textarea.insert_str(text);
        self.textarea.move_cursor(CursorMove::Top);
        self.textarea.move_cursor(CursorMove::End);

        self.edit();
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
            self.content = None;
            self.hide();
            return;
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

        self.content = Some(Content { subject, body });
        self.hide();
    }

    fn edit(&mut self) {
        if self.done {
            return;
        }

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
            EditorStatus::Editing => " edit ",
            _ => "",
        };

        let block = Block::new()
            .borders(Borders::ALL)
            .title(title);
        let inner = block.inner(area);
        block.render(area, buf);
        self.textarea.render(inner, buf);
    }
}
