use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

enum DatePickerStatus {
    Editing,
    Hiding,
}

pub struct DatePicker {
    date: Option<String>,
    status: DatePickerStatus,
}

impl DatePicker {
    pub fn new() -> Self {
        Self {
            date: None,
            status: DatePickerStatus::Hiding,
        }
    }

    pub fn handle_key_press_event(&mut self, key_event: KeyEvent) -> bool {
        match self.status {
            DatePickerStatus::Editing => {
                match key_event.code {
                    KeyCode::Esc => self.done(),
                    _ => (),
                };
                return true;
            }
            _ => (),
        }

        false
    }

    pub fn start(&mut self) {
        self.status = DatePickerStatus::Editing;
    }

    fn done(&mut self) {
        self.date = Some(String::from("2025-01-01"));
        self.hide();
    }

    fn hide(&mut self) {
        self.status = DatePickerStatus::Hiding;
    }

    pub fn get_date(&mut self) -> Option<String> {
        self.date.take()
    }
}

impl Widget for &DatePicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let DatePickerStatus::Hiding = self.status {
            return;
        }

        let block = Block::new()
            .title(" due ")
            .borders(Borders::ALL);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(12),
                Constraint::Min(0),
            ])
            .split(area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(layout[1]);
        let area = layout[1];

        Clear.render(area, buf);
        Paragraph::new("2025/01/01 00:00:00").block(block).render(area, buf);
    }
}
