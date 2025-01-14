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
                        if self.position > 0 {
                            self.position -= 1;
                        } else {
                            self.position = 7;
                        }
                    }
                    KeyCode::Char('l') => {
                        if self.position < 7 {
                            self.position += 1;
                        } else {
                            self.position = 0;
                        }
                    }
                    KeyCode::Backspace => {
                        self.position = self.position.saturating_sub(1);
                    }
                    KeyCode::Tab => {
                        if self.position < 4 {
                            self.position = 4;
                        } else if self.position < 6 {
                            self.position = 6;
                        } else {
                            self.position = 0;
                        }
                    }
                    _ => {
                        let int = Self::code_to_int(key_event.code);
                        if int >= 0 {
                            if self.position < 4 {
                                self.year[self.position] = int;
                            } else if self.position < 6 {
                                self.month[self.position - 4] = int;
                            } else if self.position < 8 {
                                self.day[self.position - 6] = int;
                            }

                            if self.position < 7 {
                                self.position += 1;
                            } else {
                                self.position = 0;
                            }
                        }
                    }
                };
                return true;
            }
            _ => (),
        }

        false
    }

    fn code_to_int(code: KeyCode) -> i16 {
        match code {
            KeyCode::Char('0') => 0,
            KeyCode::Char('1') => 1,
            KeyCode::Char('2') => 2,
            KeyCode::Char('3') => 3,
            KeyCode::Char('4') => 4,
            KeyCode::Char('5') => 5,
            KeyCode::Char('6') => 6,
            KeyCode::Char('7') => 7,
            KeyCode::Char('8') => 8,
            KeyCode::Char('9') => 9,
            _ => -1,
        }
    }

    pub fn start(&mut self) {
        self.status = DatePickerStatus::Editing;
    }

    fn done(&mut self) {
        self.date = Some(format!(
            "{}{}{}{}/{}{}/{}{}",
            self.year[0],
            self.year[1],
            self.year[2],
            self.year[3],
            self.month[0],
            self.month[1],
            self.day[0],
            self.day[1],
        ));
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

        let block = Block::new().title(" due ").borders(Borders::ALL);

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
        } else if self.position >= 4 {
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
