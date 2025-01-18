use ratatui::{
    buffer::Buffer,
    layout::{Direction, Rect},
    symbols,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub struct Preview {
    subject: String,
    body: String,
    direction: Direction,
}

impl Preview {
    pub fn new() -> Self {
        Self {
            subject: String::new(),
            body: String::new(),
            direction: Direction::Horizontal,
        }
    }

    pub fn show(&mut self, subject: &String, body: &String) {
        self.subject = subject.clone();
        self.body = body.clone();
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}

impl Widget for &Preview {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.subject.clone() + "\n\n" + &self.body;

        let corners = match self.direction {
            Direction::Horizontal => {
                symbols::border::Set {
                    top_left: symbols::line::NORMAL.horizontal_down,
                    bottom_left: symbols::line::NORMAL.horizontal_up,
                    ..symbols::border::PLAIN
                }
            }
            Direction::Vertical => {
                symbols::border::Set {
                    top_left: symbols::line::NORMAL.vertical_right,
                    top_right: symbols::line::NORMAL.vertical_left,
                    ..symbols::border::PLAIN
                }
            }
        };
        let block = Block::new()
            .border_set(corners)
            .borders(Borders::ALL);
        Clear.render(area, buf);
        Paragraph::new(text).block(block).render(area, buf);
    }
}
