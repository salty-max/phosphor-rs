use phosphor::{
    Application, Color, Command, Constraint, Direction, Event, Frame, KeyCode, Layout, Modifier,
    Rect, Style, run,
    widgets::{Block, Borders, Text},
};

struct ModalDemo {
    show_modal: bool,
}

impl Application for ModalDemo {
    type Action = bool;

    fn on_event(&self, event: Event) -> Option<Self::Action> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => return Some(true),  // Quit signal
                KeyCode::Char('m') => return Some(false), // Toggle signal
                _ => {}
            }
        }
        None
    }

    fn update(&mut self, quit: bool) -> Command {
        if quit {
            return Command::Quit;
        }
        self.show_modal = !self.show_modal;
        Command::None
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // 1. Draw Background
        let bg_block = Block::new()
            .borders(Borders::ALL)
            .title(" Main Window ")
            .style(Style::new().fg(Color::Blue));
        frame.render_widget(bg_block, area);

        let text = Text::new(
            "Press 'm' to toggle the modal.\nPress 'q' to quit.\n\n".to_string()
                + &"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(10),
        );
        frame.render_widget(text, Rect::new(2, 2, area.width - 4, area.height - 4));

        // 2. Draw Modal (if active)
        if self.show_modal {
            let modal_area = centered_rect(60, 20, area);

            // Clear the modal area (simulate a solid background)
            // We do this by drawing a block with a solid background style?
            // Or just spaces. For now, the Block borders will define it.

            let modal = Block::new()
                .borders(Borders::ALL)
                .title(" Modal ")
                .style(Style::new().fg(Color::Yellow).bg(Color::Black)) // Explicit BG helps
                .title_style(Style::new().fg(Color::Red).modifier(Modifier::BOLD));

            // Clear the area behind the modal to hide the text
            // We can do this by writing spaces manually or adding a Clear widget later.
            // For now, let's just rely on the Block.

            frame.render_widget(modal, modal_area);

            let inner = Rect::new(
                modal_area.x + 2,
                modal_area.y + 2,
                modal_area.width.saturating_sub(4),
                modal_area.height.saturating_sub(4),
            );
            frame.render_widget(
                Text::new(
                    "I am a modal!\nI am floating above the content.\n\nPress 'm' to close me.",
                ),
                inner,
            );

            frame.render_widget(
                Text::new(
                    "I am a modal!\nI am floating above the content.\n\nPress 'm' to close me.",
                ),
                inner,
            );
        }
    }
}

/// Helper function to center a rect using Layout
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::new(
        Direction::Vertical,
        vec![
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ],
    )
    .split(r);

    let vertical_slice = popup_layout[1];

    let horizontal_layout = Layout::new(
        Direction::Horizontal,
        vec![
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ],
    )
    .split(vertical_slice);

    horizontal_layout[1]
}

fn main() -> std::io::Result<()> {
    run(ModalDemo { show_modal: false })
}
