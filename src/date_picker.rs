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
                    KeyCode::Enter => self.done(),
                    KeyCode::Esc => self.clear(),
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
                    KeyCode::Char(' ') => {
                        if self.position < 7 {
                            self.position += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        if self.position > 0 {
                            self.position -= 1;
                        }
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
                    KeyCode::BackTab => {
                        if self.position < 4 {
                            self.position = 6;
                        } else if self.position < 6 {
                            self.position = 0;
                        } else {
                            self.position = 4;
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

    pub fn start(&mut self, due: &String) {
        if due.len() >= 10 {
            self.set_year(&due[0..=3]);
            self.set_month(&due[5..=6]);
            self.set_day(&due[8..=9]);
        } else {
            self.year = [2, 0, 2, 5];
            self.month = [0, 1];
            self.day = [0, 1];
        }

        self.position = 0;

        self.status = DatePickerStatus::Editing;
    }

    fn parse_four(string: &str) -> Option<[i16; 4]> {
        let mut not_digit = false;
        let res = string
            .chars()
            .map(|c| {
                if let Some(value) = c.to_digit(10) {
                    value as i16
                } else {
                    not_digit = true;
                    0
                }
            })
            .collect::<Vec<_>>()
            .try_into();

        if not_digit {
            None
        } else if let Ok(array) = res {
            Some(array)
        } else {
            None
        }
    }

    fn parse_two(string: &str) -> Option<[i16; 2]> {
        let mut not_digit = false;
        let res = string
            .chars()
            .map(|c| {
                if let Some(value) = c.to_digit(10) {
                    value as i16
                } else {
                    not_digit = true;
                    0
                }
            })
            .collect::<Vec<_>>()
            .try_into();

        if not_digit {
            None
        } else if let Ok(array) = res {
            Some(array)
        } else {
            None
        }
    }

    fn set_year(&mut self, string: &str) {
        self.year = match Self::parse_four(string) {
            Some(array) => array,
            None => [2, 0, 2, 5],
        };
    }

    fn set_month(&mut self, string: &str) {
        self.month = match Self::parse_two(string) {
            Some(array) => array,
            None => [0, 1],
        };
    }

    fn set_day(&mut self, string: &str) {
        self.day = match Self::parse_two(string) {
            Some(array) => array,
            None => [0, 1],
        };
    }

    fn done(&mut self) {
        let year = self.year[0] * 1000 + self.year[1] * 100 + self.year[2] * 10 + self.year[3];
        let mut month = self.month[0] * 10 + self.month[1];
        let mut day = self.day[0] * 10 + self.day[1];
        let leap_year = {
            if year % 400 == 0 {
                true
            } else if year % 100 == 0 {
                false
            } else if year % 4 == 0 {
                true
            } else {
                false
            }
        };

        month = month.clamp(1, 12);

        let max_day = if month == 2 {
            if leap_year {
                29
            } else {
                28
            }
        } else if matches!(month, 1 | 3 | 5 | 7 | 8 | 10 | 12) {
            31
        } else {
            30
        };
        day = day.clamp(1, max_day);

        self.date = Some(format!("{year:0>4}/{month:0>2}/{day:0>2}"));
        self.hide();
    }

    fn clear(&mut self) {
        self.date = Some(String::new());
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

        let keys = Line::from(vec![
            " ".into(),
            "enter".red(),
            " save | ".into(),
            "esc".red(),
            " clear ".into(),
        ]);
        let block = Block::new()
            .borders(Borders::ALL)
            .title(" due ")
            .title_bottom(keys.right_aligned());
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(26),
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
        let line = Line::from(spans).centered();
        Paragraph::new(line).block(block).render(area, buf);
    }
}
