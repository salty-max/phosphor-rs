use phosphor::{
    Application, Color, Command, Event, Frame, KeyCode, Modifier, MouseEvent, Style, run,
    widgets::{Block, Borders, Text},
};

struct MouseDemo {
    click_pos: Option<(u16, u16)>,
    last_action: String,
}

impl Application for MouseDemo {
    type Action = Event;

    fn on_event(&self, event: Event) -> Option<Self::Action> {
        Some(event)
    }

    fn update(&mut self, event: Self::Action) -> Command {
        match event {
            Event::Key(key) => {
                if let KeyCode::Char('q') = key.code {
                    return Command::Quit;
                }
                self.last_action = format!("Key pressed: {:?}", key.code);
            }
            Event::Mouse(MouseEvent { x, y, kind }) => {
                self.click_pos = Some((x, y));
                self.last_action = format!("Mouse {:?} at {},{}", kind, x, y);
            }
            _ => {}
        }
        Command::None
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::new()
            .borders(Borders::ALL)
            .title(" Mouse Demo ")
            .title_style(Style::new().fg(Color::Green).modifier(Modifier::BOLD));

        frame.render_widget(block, area);

        let info = format!(
            "Click anywhere! Press 'q' to quit.\n\nLast Action: {}",
            self.last_action
        );

        frame.render_widget(Text::new(info), Rect::new(2, 2, area.width - 4, 5));

        if let Some((x, y)) = self.click_pos {
            // Draw a target at the click position
            // We need to be careful not to draw outside the frame
            if x < area.width && y < area.height {
                frame.with_style(Style::new().fg(Color::Red).modifier(Modifier::BOLD), |f| {
                    f.write_str(x, y, "X")
                });
            }
        }
    }
}

use phosphor::Rect;

fn main() -> std::io::Result<()> {
    run(MouseDemo {
        click_pos: None,
        last_action: "None".to_string(),
    })
}
