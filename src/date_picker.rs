use chrono::Local;
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
                    KeyCode::F(1) => self.done(),
                    KeyCode::Char('j') => {
                        if self.position < 4 {
                            let mut year = self.year[0] * 1000 + self.year[1] * 100 + self.year[2] * 10 + self.year[3];
                            year -= 10_i16.pow(3 - self.position as u32);
                            while year < 0 {
                                year += 10000;
                            }
                            self.year = Self::split_four(year);
                        } else if self.position < 6 {
                            let mut month = self.month[0] * 10 + self.month[1];
                            month -= 1;
                            if month > 12 {
                                month = 12;
                            }
                            while month < 1 {
                                month += 12;
                            }
                            self.position = 5;
                            self.month = Self::split_two(month);
                        } else if self.position < 8 {
                            let year = self.year[0] * 1000 + self.year[1] * 100 + self.year[2] * 10 + self.year[3];
                            let leap_year = Self::is_leap_year(year);

                            let mut month = self.month[0] * 10 + self.month[1];
                            month = month.clamp(1, 12);

                            let mut day = self.day[0] * 10 + self.day[1];
                            let max_day = Self::get_max_day(month, leap_year);

                            day -=1;
                            if day > max_day {
                                day = max_day;
                            }
                            if day < 1 {
                                day = max_day;
                            }
                            self.position = 7;
                            self.day = Self::split_two(day);
                        }
                    }
                    KeyCode::Char('k') => {
                        if self.position < 4 {
                            let mut year = self.year[0] * 1000 + self.year[1] * 100 + self.year[2] * 10 + self.year[3];
                            year += 10_i16.pow(3 - self.position as u32);
                            year %= 10000;
                            self.year = Self::split_four(year);
                        } else if self.position < 6 {
                            let mut month = self.month[0] * 10 + self.month[1];
                            month %= 12;
                            month += 1;
                            self.position = 5;
                            self.month = Self::split_two(month);
                        } else if self.position < 8 {
                            let year = self.year[0] * 1000 + self.year[1] * 100 + self.year[2] * 10 + self.year[3];
                            let leap_year = Self::is_leap_year(year);

                            let mut month = self.month[0] * 10 + self.month[1];
                            month = month.clamp(1, 12);

                            let mut day = self.day[0] * 10 + self.day[1];
                            let max_day = Self::get_max_day(month, leap_year);

                            if day > max_day {
                                day = 0;
                            }
                            day +=1;
                            self.position = 7;
                            self.day = Self::split_two(day);
                        }
                    }
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

    fn split_four(int: i16) -> [i16; 4] {
        [int / 1000 % 10, int / 100 % 10, int / 10 % 10, int % 10]
    }

    fn split_two(int: i16) -> [i16; 2] {
        let int = int % 100;
        [int / 10 % 10, int % 10]
    }

    fn is_leap_year(year: i16) -> bool {
        if year % 400 == 0 {
            true
        } else if year % 100 == 0 {
            false
        } else if year % 4 == 0 {
            true
        } else {
            false
        }
    }

    fn get_max_day(month: i16, leap_year: bool) -> i16 {
        if month == 2 {
            if leap_year {
                29
            } else {
                28
            }
        } else if matches!(month, 1 | 3 | 5 | 7 | 8 | 10 | 12) {
            31
        } else {
            30
        }
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
        let mut date = due;
        let today = format!("{}", Local::now().format("%Y/%m/%d"));
        if date.len() < 10 {
            date = &today;
        }
        self.set_year(&date[0..=3]);
        self.set_month(&date[5..=6]);
        self.set_day(&date[8..=9]);

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
            None => {
                let year = format!("{}", Local::now().format("%Y"));
                Self::parse_four(&year).unwrap()
            }
        };
    }

    fn set_month(&mut self, string: &str) {
        self.month = match Self::parse_two(string) {
            Some(array) => array,
            None => {
                let month = format!("{}", Local::now().format("%m"));
                Self::parse_two(&month).unwrap()
            }
        };
    }

    fn set_day(&mut self, string: &str) {
        self.day = match Self::parse_two(string) {
            Some(array) => array,
            None => {
                let day = format!("{}", Local::now().format("%d"));
                Self::parse_two(&day).unwrap()
            }
        };
    }

    fn done(&mut self) {
        let year = self.year[0] * 1000 + self.year[1] * 100 + self.year[2] * 10 + self.year[3];
        let leap_year = Self::is_leap_year(year);

        let mut month = self.month[0] * 10 + self.month[1];
        month = month.clamp(1, 12);

        let mut day = self.day[0] * 10 + self.day[1];
        let max_day = Self::get_max_day(month, leap_year);
        day = day.clamp(1, max_day);

        self.date = Some(format!("{year:0>4}/{month:0>2}/{day:0>2}"));
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
            .borders(Borders::ALL)
            .title(" due ");
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
        let line = Line::from(spans).centered();
        Paragraph::new(line).block(block).render(area, buf);
    }
}
