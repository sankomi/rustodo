use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub struct Preview {
    subject: String,
    body: String,
}

impl Preview {
    pub fn new() -> Self {
        Self {
            subject: String::new(),
            body: String::new(),
        }
    }

    pub fn show(&mut self, subject: &String, body: &String) {
        self.subject = subject.clone();
        self.body = body.clone();
    }
}

impl Widget for &Preview {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.subject.clone() + "\n\n" + &self.body;

        let block = Block::new().borders(Borders::ALL);
        Clear.render(area, buf);
        Paragraph::new(text).block(block).render(area, buf);
    }
}
