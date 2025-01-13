use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

enum DatePickerStatus {
    Editing,
    Hiding,
}

pub struct DatePicker {
    date: Option<String>,
    year: [i16; 4],
    month: [i16; 2],
    day: [i16; 2],
    position: usize,
    status: DatePickerStatus,
}

impl DatePicker {
    pub fn new() -> Self {
        Self {
            date: None,
            year: [2, 0, 2, 5],
            month: [0, 1],
            day: [0, 1],
            position: 0,
            status: DatePickerStatus::Hiding,
        }
    }

    pub fn handle_key_press_event(&mut self, key_event: KeyEvent) -> bool {
        match self.status {
            DatePickerStatus::Editing => {
                match key_event.code {
                    KeyCode::Esc => self.done(),
                    KeyCode::Char('h') => {
                        self.position = self.position.saturating_sub(1);
                    }
                    KeyCode::Char('l') => {
                        if self.position < 7 {
                            self.position += 1;
                        }
                    }
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
        let mut spans: Vec<Span> = vec![
            self.year[0].to_string().into(),
            self.year[1].to_string().into(),
            self.year[2].to_string().into(),
            self.year[3].to_string().into(),
            "/".into(),
            self.month[0].to_string().into(),
            self.month[1].to_string().into(),
            "/".into(),
            self.day[0].to_string().into(),
            self.day[1].to_string().into(),
        ];
        let position = if self.position >= 6 {
            self.position + 2
        } else if self.position >= 4{
            self.position + 1
        } else {
            self.position
        };
        if let Some(span) = spans.get_mut(position) {
            *span = span.clone().on_red();
        }
        let line = Line::from(spans);
        Paragraph::new(line).block(block).render(area, buf);
    }
}
