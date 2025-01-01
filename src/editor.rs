use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{Clear, Widget},
};
use tui_textarea::TextArea;

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
        let text = format!("{}\n\n{}", subject, body);
        self.textarea = TextArea::default();
        self.textarea.insert_str(text);
        self.view();
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
                } else if *line != "" {
                    not_blank += 1;
                    true
                } else {
                    false
                }
            })
            .map(|line| line.clone())
            .collect::<Vec<_>>();
        let subject = lines[0].clone();
        let mut not_blank = false;
        let body = lines[1..]
            .iter()
            .filter(|line| {
                not_blank = not_blank || *line != "";
                not_blank
            })
            .map(|line| line.clone())
            .collect::<Vec<_>>()
            .join("\n")
            .clone();

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
        self.textarea.render(area, buf);
    }
}
