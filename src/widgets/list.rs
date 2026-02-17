use crate::{Style, Widget};

pub struct List {
    items: Vec<String>,
    selected: Option<usize>,
    style: Style,
    highlight_style: Style,
    highlight_symbol: Option<String>,
}

impl List {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: None,
            style: Style::default(),
            highlight_style: Style::default(),
            highlight_symbol: None,
        }
    }

    pub fn selected(&mut self, index: usize) {
        self.selected = Some(index);
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn highlight_symbol(mut self, symbol: String) -> Self {
        self.highlight_symbol = Some(symbol);
        self
    }
}

impl Widget for List {
    fn render(self, area: crate::Rect, frame: &mut crate::Frame) {
        for item in &self.items {}
    }
}
