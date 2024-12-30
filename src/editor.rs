use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::Widget,
};
use tui_textarea::TextArea;

enum EditorStatus {
    Hiding,
    Viewing,
    Editing,
}

pub struct Editor<'a> {
    status: EditorStatus,
    textarea: TextArea<'a>,
    on_done: fn(&str) -> (),
}

impl Editor<'_> {
    pub fn new() -> Self {
        Self {
            status: EditorStatus::Hiding,
            textarea: TextArea::default(),
            on_done: |_| (),
        }
    }

    pub fn handle_key_press_event(&mut self, key_event: KeyEvent) -> bool {
        match self.status {
            EditorStatus::Editing => {
                match key_event.code {
                    KeyCode::Esc => self.view(true),
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

    pub fn start(&mut self, on_done: fn(&str) -> ()) {
        self.on_done = on_done;
        self.view(false);
    }

    fn view(&mut self, done: bool) {
        if done {
            let text = self.textarea.lines().join("\n");
            (self.on_done)(&text);
        }

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

        self.textarea.render(area, buf);
    }
}
